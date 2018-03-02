// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#[macro_use]
extern crate error_chain;

#[macro_use(kw, var)]
extern crate mentat;

extern crate libc;
extern crate mentat_core;
extern crate mentat_ffi;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use mentat::{
    InProgress,
    IntoResult,
    Queryable,
    QueryExecutionResult,
    QueryInputs,
    Store,
    TypedValue,
    ValueType,
};

use mentat_ffi::utils::log;

use mentat_core::{
    KnownEntid,
};

use mentat::entity_builder::{
    BuildTerms,
};

use mentat::vocabulary::{
    AttributeBuilder,
    Definition,
    VersionedStore,
};

use mentat::vocabulary::attribute::{
    Unique
};

use mentat::sync::Syncable;

pub use time::Timespec;
pub use mentat::Uuid;

pub mod labels;
pub mod items;
pub mod errors;

mod utils;

use errors::{
    ErrorKind,
    Result,
};

use std::os::raw::{
    c_char,
};

use mentat_ffi::utils::strings::{
    c_char_to_string,
    string_to_c_char,
};

pub use items::{
    Item,
    Items,
};

pub use labels::{
    Label,
};

use utils::{
    ToInner,
    ToTypedValue,
};

fn transact_items_vocabulary(in_progress: &mut InProgress) -> Result<()> {
    in_progress.ensure_vocabulary(&Definition {
            name: kw!(:toodle/items),
            version: 1,
            attributes: vec![
                (kw!(:item/uuid),
                 AttributeBuilder::new()
                    .value_type(ValueType::Uuid)
                    .multival(false)
                    .unique(Unique::Value)
                    .index(true)
                    .build()),
                (kw!(:item/name),
                 AttributeBuilder::new()
                    .value_type(ValueType::String)
                    .multival(false)
                    .build()),
                (kw!(:item/due_date),
                 AttributeBuilder::new()
                    .value_type(ValueType::Instant)
                    .multival(false)
                    .build()),
                (kw!(:item/completion_date),
                 AttributeBuilder::new()
                    .value_type(ValueType::Instant)
                    .multival(false)
                    .build()),
                (kw!(:item/label),
                 AttributeBuilder::new()
                    .value_type(ValueType::Ref)
                    .multival(true)
                    .build()),
            ],
        })
        .map_err(|e| e.into())
        .and(Ok(()))
}

fn transact_labels_vocabulary(in_progress: &mut InProgress) -> Result<()> {
    in_progress.ensure_vocabulary(&Definition {
            name: kw!(:toodle/labels),
            version: 1,
            attributes: vec![
                (kw!(:label/name),
                 AttributeBuilder::new()
                    .value_type(ValueType::String)
                    .multival(false)
                    .unique(Unique::Identity)
                    .build()),
                (kw!(:label/color),
                 AttributeBuilder::new()
                    .value_type(ValueType::String)
                    .multival(false)
                    .build()),
            ],
        })
        .map_err(|e| e.into())
        .and(Ok(()))
}

fn create_uuid() -> Uuid {
    uuid::Uuid::new_v4()
}

fn return_date_field(results: QueryExecutionResult) -> Result<Option<Timespec>> {
    results.into_scalar_result()
           .map(|o| o.and_then(|ts| ts.to_inner()))
           .map_err(|e| e.into())
}

pub trait Toodle {
    fn item_row_to_item(&mut self, row: Vec<TypedValue>) -> Item;
    fn fetch_completion_date_for_item(&mut self, item_id: &Uuid) -> Result<Option<Timespec>>;
    fn fetch_due_date_for_item(&mut self, item_id: &Uuid) -> Result<Option<Timespec>>;

    fn initialize(&mut self) -> Result<()>;
    fn create_label(&mut self, name: String, color: String) -> Result<Option<Label>>;
    fn fetch_label(&mut self, name: &String) -> Result<Option<Label>>;
    fn fetch_labels(&mut self) -> Result<Vec<Label>>;
    fn fetch_labels_for_item(&mut self, item_uuid: &Uuid) -> Result<Vec<Label>>;
    fn fetch_items_with_label(&mut self, label: &Label) -> Result<Vec<Item>>;
    fn fetch_items(&mut self) -> Result<Items>;
    fn fetch_item(&mut self, uuid: &Uuid) -> Result<Option<Item>>;
    fn create_item(&mut self, item: &Item) -> Result<Uuid>;
    fn create_and_fetch_item(&mut self, item: &Item) -> Result<Option<Item>>;
    fn update_item_by_uuid(&mut self,
                               uuid_string: &str,
                               name: Option<String>,
                               due_date: Option<Timespec>,
                               completion_date: Option<Timespec>)
                               -> Result<Item>;
    fn update_item(&mut self,
                       item: &Item, name: Option<String>,
                       due_date: Option<Timespec>,
                       completion_date: Option<Timespec>,
                       labels: Option<&Vec<Label>>) -> Result<()>;

    fn do_sync(&mut self, server_uri: &String, user_uuid: &String) -> Result<()>;
}

pub struct ResultC {
    pub error: *const c_char
}

impl From<std::result::Result<(), errors::Error>> for ResultC {
    fn from(result: std::result::Result<(), errors::Error>) -> Self {
        match result {
            Ok(_) => {
                ResultC {
                    error: std::ptr::null(),
                }
            },
            Err(e) => {
                ResultC {
                    error: string_to_c_char(e.description().into())
                }
            }
        }
    }
}

impl Toodle for Store {

    fn initialize(&mut self) -> Result<()> {
        let mut in_progress = self.begin_transaction()?;
        {
            transact_items_vocabulary(&mut in_progress)?;
        }
        {
            transact_labels_vocabulary(&mut in_progress)?;
        }
        in_progress.commit()?;
        Ok(())
    }

    fn item_row_to_item(&mut self, row: Vec<TypedValue>) -> Item {
        let uuid = row[1].clone().to_inner();
        let item;
        {
            item = Item {
                id: row[0].clone().to_inner(),
                uuid: uuid,
                name: row[2].clone().to_inner(),
                due_date: self.fetch_due_date_for_item(&uuid).unwrap_or(None),
                completion_date: self.fetch_completion_date_for_item(&uuid).unwrap_or(None),
                labels: self.fetch_labels_for_item(&uuid).unwrap_or(vec![]),
            }
        }
        item
    }

    fn create_label(&mut self, name: String, color: String) -> Result<Option<Label>> {
        {
            let in_progress = self.begin_transaction()?;
            let mut builder = in_progress.builder().describe_tempid("label");

            builder.add_kw(&kw!(:label/name), TypedValue::typed_string(&name))?;
            builder.add_kw(&kw!(:label/color), TypedValue::typed_string(&color))?;

            builder.commit()?;
        }
        self.fetch_label(&name)
    }

    fn fetch_label(&mut self, name: &String) -> Result<Option<Label>> {
        let query = r#"[:find [?eid ?name ?color]
                        :in ?name
                        :where
                        [?eid :label/name ?name]
                        [?eid :label/color ?color]
        ]"#;
        let in_progress_read = self.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?name), name.to_typed_value())]);
        in_progress_read
            .q_once(query, args)
            .into_tuple_result()
            .map(|o| o.as_ref().and_then(Label::from_row))
            .map_err(|e| e.into())
    }

    fn fetch_labels(&mut self) -> Result<Vec<Label>> {
        let query = r#"[:find ?eid ?name ?color
                        :where
                        [?eid :label/name ?name]
                        [?eid :label/color ?color]
        ]"#;
        let in_progress_read = self.begin_read()?;
        in_progress_read
            .q_once(query, None)
            .into_rel_result()
            .map(|rows| rows.iter().filter_map(|row| Label::from_row(&row)).collect())
            .map_err(|e| e.into())
    }

    fn fetch_labels_for_item(&mut self, item_uuid: &Uuid) -> Result<Vec<Label>> {
        let query = r#"[:find ?l ?name ?color
                        :in ?item_uuid
                        :where
                        [?i :item/uuid ?item_uuid]
                        [?i :item/label ?l]
                        [?l :label/name ?name]
                        [?l :label/color ?color]
        ]"#;
        let in_progress_read = self.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?item_uuid), item_uuid.to_typed_value())]);
        in_progress_read
            .q_once(query, args)
            .into_rel_result()
            .map(|rows| rows.iter().filter_map(|row| Label::from_row(&row)).collect())
            .map_err(|e| e.into())
    }


    fn fetch_items_with_label(&mut self, label: &Label) -> Result<Vec<Item>> {
        let query = r#"[:find ?eid ?uuid ?name
                        :in ?label
                        :where
                        [?l :label/name ?label]
                        [?eid :item/label ?l]
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;
        let rows;
        {
            let in_progress_read = self.begin_read()?;
            let args = QueryInputs::with_value_sequence(vec![(var!(?label), label.name.to_typed_value())]);
            rows = in_progress_read
                .q_once(query, args)
                .into_rel_result()
                .map_err(|e| e.into());
        }
        rows.map(|rows| rows.into_iter().map(|r| self.item_row_to_item(r)).collect())
    }

    fn fetch_items(&mut self) -> Result<Items> {
        let query = r#"[:find ?eid ?uuid ?name
                        :where
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;

        let rows;
        {
            let in_progress_read = self.begin_read()?;
            rows = in_progress_read
                .q_once(query, None)
                .into_rel_result()
                .map_err(|e| e.into());
        }
        rows.map(|rows| Items::new(rows.into_iter().map(|r| self.item_row_to_item(r)).collect()))
    }

    fn fetch_item(&mut self, uuid: &Uuid) -> Result<Option<Item>> {
        let query = r#"[:find [?eid ?uuid ?name]
                        :in ?uuid
                        :where
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;
        let rows;
        {
            let in_progress_read = self.begin_read()?;
            let args = QueryInputs::with_value_sequence(vec![(var!(?uuid), uuid.to_typed_value())]);
            rows = in_progress_read
                .q_once(query, args)
                .into_tuple_result()
                .map(|o| o.map(|r| r))
                .map_err(|e| e.into());
        }

        rows.map(|row| row.map(|r| self.item_row_to_item(r)))
    }

    fn fetch_completion_date_for_item(&mut self, item_id: &Uuid) -> Result<Option<Timespec>> {
        let query = r#"[:find ?date .
            :in ?uuid
            :where
            [?eid :item/uuid ?uuid]
            [?eid :item/completion_date ?date]
        ]"#;

        let in_progress_read = self.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?uuid), item_id.to_typed_value())]);
        return_date_field(
            in_progress_read
            .q_once(query, args))
    }

    fn fetch_due_date_for_item(&mut self, item_id: &Uuid) -> Result<Option<Timespec>> {
        let query = r#"[:find ?date .
            :in ?uuid
            :where
            [?eid :item/uuid ?uuid]
            [?eid :item/due_date ?date]
        ]"#;
        let in_progress_read = self.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?uuid), item_id.to_typed_value())]);
        let date = return_date_field(
            in_progress_read
            .q_once(query, args));
        date
    }

    fn create_item(&mut self, item: &Item) -> Result<Uuid> {
        let item_uuid = create_uuid();
        log::d(&format!("create_item item_uuid: {:?}", item_uuid));
        {
            let in_progress = self.begin_transaction()?;
            log::d(&format!("create_item in_progress"));
            let mut builder = in_progress.builder().describe_tempid("item");
            log::d(&format!("create_item builder"));
            builder.add_kw(&kw!(:item/uuid), TypedValue::Uuid(item_uuid))?;
            log::d(&format!("create_item builder uuid"));
            builder.add_kw(&kw!(:item/name), TypedValue::typed_string(&item.name))?;
            log::d(&format!("create_item builder name"));
            if let Some(due_date) = item.due_date {
                builder.add_kw(&kw!(:item/due_date), due_date.to_typed_value())?;
                log::d(&format!("create_item builder due_date"));
            }
            if let Some(completion_date) = item.completion_date {
                builder.add_kw(&kw!(:item/completion_date), completion_date.to_typed_value())?;
                log::d(&format!("create_item builder completion_date"));
            }
            
            log::d(&format!("create_item builder pre commit"));
            builder.commit()?;
            log::d(&format!("create_item builder post commit"));
        }
        Ok(item_uuid)
    }

    fn create_and_fetch_item(&mut self, item: &Item) -> Result<Option<Item>> {
        log::d(&format!("create_and_fetch_item item: {:?}", item));
        let item_uuid_res = self.create_item(&item);
        match item_uuid_res {
            Ok(item_uuid) => {
                log::d(&format!("create_and_fetch_item item_uuid: {:?}", item_uuid));
                let item = self.fetch_item(&item_uuid);
                log::d(&format!("create_and_fetch_item fetch_item: {:?}", item));
                item
            },
            Err(e) => {
                log::d(&format!("create_and_fetch_item error: {:?}", e));
                Err(e)
            }
        }
    }

    fn update_item_by_uuid(&mut self,
                               uuid_string: &str,
                               name: Option<String>,
                               due_date: Option<Timespec>,
                               completion_date: Option<Timespec>)
                               -> Result<Item> {
        let uuid = Uuid::parse_str(&uuid_string)?;
        let item =
            self.fetch_item(&uuid)
                .ok()
                .unwrap_or_default()
                .ok_or_else(|| ErrorKind::ItemNotFound(uuid_string.to_string()))?;

        let new_item =
            self.update_item(&item, name, due_date, completion_date, None)
                .and_then(|_| self.fetch_item(&uuid))
                .unwrap_or_default()
                .ok_or_else(|| ErrorKind::ItemNotFound(uuid_string.to_string()))?;

        Ok(new_item)
    }

    fn update_item(&mut self,
                       item: &Item, name: Option<String>,
                       due_date: Option<Timespec>,
                       completion_date: Option<Timespec>,
                       labels: Option<&Vec<Label>>) -> Result<()> {
        let entid = KnownEntid(item.id.to_owned().ok_or_else(|| ErrorKind::ItemNotFound(item.uuid.hyphenated().to_string()))?.id);
        let existing_labels = self.fetch_labels_for_item(&(item.uuid)).unwrap_or(vec![]);
        let in_progress = self.begin_transaction()?;
        let mut builder = in_progress.builder().describe(entid);

        if let Some(name) = name {
            if item.name != name {
                builder.add_kw(&kw!(:item/name), TypedValue::typed_string(&name))?;
            }
        }

        if item.due_date != due_date {
            let due_date_kw = kw!(:item/due_date);
            if let Some(date) = due_date {
                builder.add_kw(&due_date_kw, date.to_typed_value())?;
            } else if let Some(date) = item.due_date {
                builder.retract_kw(&due_date_kw, date.to_typed_value())?;
            }
        }

        if item.completion_date != completion_date {
            let completion_date_kw = kw!(:item/completion_date);
            if let Some(date) = completion_date {
                builder.add_kw(&completion_date_kw, date.to_typed_value())?;
            } else if let Some(date) = item.completion_date {
                builder.retract_kw(&completion_date_kw, date.to_typed_value())?;
            }
        }

        if let Some(new_labels) = labels {
            let item_labels_kw = kw!(:item/label);
            for label in new_labels {
                builder.add_kw(&item_labels_kw, TypedValue::Ref(label.id.clone().unwrap().id))?;
            }
            for label in existing_labels {
                if !new_labels.contains(&label) && label.id.is_some() {
                    builder.retract_kw(&item_labels_kw, TypedValue::Ref(label.id.clone().unwrap().id))?;
                }
            }
        }
        builder.commit()
               .map_err(|e| e.into())
               .and(Ok(()))
    }

    fn do_sync(&mut self, server_uri: &String, user_uuid: &String) -> Result<()> {
        self.sync(server_uri, user_uuid);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{
        Toodle,
        Label,
        Item,
        create_uuid,
    };

    use time::{
        now_utc,
    };

    use mentat::{
        Uuid,
    };
    use mentat::edn;

    fn toodle() -> Store {
        Store::open(String::new()).expect("Expected a Toodle")
    }

    fn assert_ident_present(edn: edn::Value, namespace: &str, name: &str) -> bool {
        match edn {
            edn::Value::Vector(v) => {
                let mut found = false;
                for val in v.iter() {
                    found = assert_ident_present(val.clone(), namespace, name);
                    if found {
                        break;
                    }
                }
                found
            },
            edn::Value::Map(m) => {
                let mut found = false;
                for (key, val) in &m {
                    if let edn::Value::NamespacedKeyword(ref kw) = *key {
                        if kw.namespace == "db" && kw.name == "ident" {
                            found = assert_ident_present(val.clone(), namespace, name);
                            if found { break; }
                        } else {
                            continue
                        }
                    }
                }
                found
            },
            edn::Value::NamespacedKeyword(kw) => kw.namespace == namespace && kw.name == name,
            _ => false
        }
    }

    #[test]
    fn test_new_toodle() {
        let manager = toodle();
        let conn = manager.connection.conn();
        let schema = conn.current_schema().to_edn_value();
        assert_ident_present(schema.clone(), "label", "name");
        assert_ident_present(schema, "list", "name");
    }

    #[test]
    fn test_create_label() {
        let mut manager = toodle();
        let name = "test".to_string();
        let color = "#000000".to_string();

        let label = manager.create_label(name.clone(), color.clone()).expect("expected a label option");
        assert!(label.is_some());
        let label = label.unwrap();
        assert!(label.id.is_some());
        assert_eq!(label.name, name);
        assert_eq!(label.color, color);
    }

    #[test]
    fn test_fetch_label() {
        let mut manager = toodle();
        let created_label = manager.create_label("test".to_string(), "#000000".to_string()).expect("expected a label option").expect("Expected a label");
        let fetched_label = manager.fetch_label(&created_label.name).expect("expected a label option").expect("expected a label");
        assert_eq!(fetched_label, created_label);

        let fetched_label = manager.fetch_label(&"doesn't exist".to_string()).expect("expected a label option");
        assert_eq!(fetched_label, None);
    }

    #[test]
    fn test_fetch_labels() {
        let mut manager = toodle();

        let labels = ["label1".to_string(), "label2".to_string(), "label3".to_string()];
        for label in labels.iter() {
            let _  = manager.create_label(label.clone(), "#000000".to_string()).expect("expected a label option");
        }
        let fetched_labels = manager.fetch_labels().expect("expected a vector of labels");
        assert_eq!(fetched_labels.len(), labels.len());
        for label in fetched_labels.iter() {
            assert!(labels.contains(&label.name));
        }
    }

    #[test]
    fn test_create_item() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: Some(date.clone()),
            completion_date: Some(date.clone()),
            labels: vec![label, label2]
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        let due_date = item.due_date.expect("expecting a due date");
        assert_eq!(due_date.sec, date.sec);
        let completion_date = item.completion_date.expect("expecting a completion date");
        assert_eq!(completion_date.sec, date.sec);
        assert_eq!(item.labels, i.labels);
    }

    #[test]
    fn test_create_item_no_due_date() {
        let mut manager = toodle();
        let l = Label {
            id: None,
            name: "label1".to_string(),
            color: "#000000".to_string()
        };
        let label = manager.create_label(l.name.clone(), l.color.clone()).expect("expected a label option").unwrap();

        let l2 = Label {
            id: None,
            name: "label2".to_string(),
            color: "#000000".to_string()
        };
        let label2 = manager.create_label(l2.name.clone(), l2.color.clone()).expect("expected an item option").unwrap();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: None,
            completion_date: Some(date.clone()),
            labels: vec![label, label2]
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        assert_eq!(item.due_date, i.due_date);
        let completion_date = item.completion_date.expect("expecting a completion date");
        assert_eq!(completion_date.sec, date.sec);
        assert_eq!(item.labels, i.labels);
    }

    #[test]
    fn test_create_item_no_completion_date() {
        let mut manager = toodle();
        let l = Label {
            id: None,
            name: "label1".to_string(),
            color: "#000000".to_string()
        };
        let label = manager.create_label(l.name.clone(), l.color.clone()).expect("expected a label option").unwrap();

        let l2 = Label {
            id: None,
            name: "label2".to_string(),
            color: "#000000".to_string()
        };
        let label2 = manager.create_label(l2.name.clone(), l2.color.clone()).expect("expected a label option").unwrap();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: Some(date.clone()),
            completion_date: None,
            labels: vec![label, label2]
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        let due_date = item.due_date.expect("expecting a due date");
        assert_eq!(due_date.sec, date.sec);
        assert_eq!(item.completion_date, i.completion_date);
        assert_eq!(item.labels, i.labels);
    }

    #[test]
    fn test_fetch_item() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let mut created_item = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label]
        };

        created_item.uuid = manager.create_item(&created_item).expect("expected a uuid");
        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item.uuid, created_item.uuid);
        assert_eq!(fetched_item.name, created_item.name);
        assert_eq!(fetched_item.due_date, created_item.due_date);
        assert_eq!(fetched_item.completion_date, created_item.completion_date);
        assert_eq!(fetched_item.labels, created_item.labels);

        let tmp_uuid = create_uuid().hyphenated().to_string();
        let item_uuid = Uuid::parse_str(&tmp_uuid).unwrap();
        let fetched_item = manager.fetch_item(&item_uuid).expect("expected an item option");
        assert_eq!(fetched_item, None);
    }

    #[test]
    fn test_fetch_labels_for_item() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let mut item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label, label2, label3]
        };

        item1.uuid = manager.create_item(&item1).expect("expected a uuid");

        let fetched_labels = manager.fetch_labels_for_item(&item1.uuid).expect("expected a vector of labels");
        assert_eq!(fetched_labels, item1.labels);
    }

    #[test]
    fn test_fetch_items_with_label() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label.clone()]
        };
        let item2 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 2".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label.clone()]
        };
        let item3 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 3".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label.clone(), label2.clone()]
        };

        let item4 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 4".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label2.clone()]
        };

        let item1 = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected item1");
        let item2 = manager.create_and_fetch_item(&item2).expect("expected an item option").expect("expected item2");
        let item3 = manager.create_and_fetch_item(&item3).expect("expected an item option").expect("expected item3");
        let item4 = manager.create_and_fetch_item(&item4).expect("expected an item option").expect("expected item4");

        let fetched_label1_items = manager.fetch_items_with_label(&label).expect("expected a vector of items");
        assert_eq!(fetched_label1_items, vec![item1, item2, item3.clone()]);
        let fetched_label2_items = manager.fetch_items_with_label(&label2).expect("expected a vector of items");
        assert_eq!(fetched_label2_items, vec![item3, item4]);
    }

    #[test]
    fn test_update_item_add_label() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a labeloption").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label, label2]
        };

        let mut created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        let mut new_labels = item1.labels.clone();
        new_labels.push(label3);

        match manager.update_item(&created_item, None, None, None, Some(&new_labels)) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("e {:?}", e);
                assert!(false)
            }
        }

        created_item.labels = new_labels;

        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item, created_item);
    }

    #[test]
    fn test_update_item_remove_label() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label, label2, label3]
        };

        let mut created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        let mut new_labels = created_item.labels.clone();
        new_labels.remove(2);

        match manager.update_item(&created_item, None, None, None, Some(&new_labels)) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("e {:?}", e);
                assert!(false)
            }
        }

        created_item.labels = new_labels;

        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item, created_item);
    }

    #[test]
    fn test_update_item_add_due_date() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected alabel option").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label, label2, label3]
        };

        let created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");

        match manager.update_item(&created_item, None, Some(date), None, None) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("e {:?}", e);
                assert!(false)
            }
        }

        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        let due_date = fetched_item.due_date.expect("expected a due date");
        assert_eq!(due_date.sec, date.sec);
    }

    #[test]
    fn test_update_item_change_name() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: Some(date),
            completion_date: None,
            labels: vec![label, label2, label3]
        };

        let mut created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, Some("new name".to_string()), None, None, None) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("e {:?}", e);
                assert!(false)
            }
        }

        created_item.name = "new name".to_string();

        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item.name, created_item.name);
    }

    #[test]
    fn test_update_item_complete_item() {
        let mut manager = toodle();
        let label = manager.create_label("label1".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label2 = manager.create_label("label2".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();
        let label3 = manager.create_label("label3".to_string(), "#000000".to_string()).expect("expected a label option").unwrap();

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
            labels: vec![label, label2, label3]
        };

        let created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, None, None, Some(date), None) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("e {:?}", e);
                assert!(false)
            }
        }

        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        let completion_date = fetched_item.completion_date.expect("expected a completion_date");
        assert_eq!(completion_date.sec, date.sec);
    }
}
