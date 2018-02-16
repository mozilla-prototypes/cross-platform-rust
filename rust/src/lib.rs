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

extern crate mentat_core;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use mentat::{
    InProgress,
    IntoResult,
    Queryable,
    QueryInputs,
    Store,
    TypedValue,
    ValueType,
};

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

pub use time::Timespec;
pub use mentat::Uuid;

pub mod items;
pub mod errors;
mod utils;

use errors::{
    ErrorKind,
    Result,
};

pub use items::{
    Item,
    Items,
};

use utils::{
    ToInner,
    ToTypedValue,
    create_uuid,
    return_date_field,
};

// Creates items as:
// [
//  {   :db/ident       :item/uuid
//      :db/valueType   :db.type/uuid
//      :db/cardinality :db.cardinality/one
//      :db/unique      :db.unique/value
//      :db/index true                        },
//  {   :db/ident       :item/name
//      :db/valueType   :db.type/string
//      :db/cardinality :db.cardinality/one
//      :db/index       true
//      :db/fulltext    true                  },
//  {   :db/ident       :item/completion_date
//      :db/valueType   :db.type/instant
//      :db/cardinality :db.cardinality/one   }
// ]
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
                    .fulltext(true)
                    .build()),
                (kw!(:item/completion_date),
                 AttributeBuilder::new()
                    .value_type(ValueType::Instant)
                    .multival(false)
                    .build()),
            ],
        })
        .map_err(|e| e.into())
        .and(Ok(()))
}

#[repr(C)]
pub struct Toodle {
    connection: Store,
}

impl Toodle {
    pub fn new<T>(uri: T) -> Result<Toodle>  where T: Into<Option<String>> {
        let uri_string = uri.into().unwrap_or(String::new());
        let mut store_result = Store::open(&uri_string)?;
        {
            // TODO proper error handling at the FFI boundary
            let mut in_progress = store_result.begin_transaction()?;
            in_progress.verify_core_schema()?;
            transact_items_vocabulary(&mut in_progress)?;
            in_progress.commit()?;
        }

        let toodle = Toodle {
            connection: store_result,
        };

        Ok(toodle)
    }
}

impl Toodle {

    pub fn create_item(&mut self, item: &Item) -> Result<Uuid> {
        let item_uuid = create_uuid();
        {
            let in_progress = self.connection.begin_transaction()?;
            let mut builder = in_progress.builder().describe_tempid("item");

            builder.add_kw(&kw!(:item/uuid), TypedValue::Uuid(item_uuid))?;
            builder.add_kw(&kw!(:item/name), TypedValue::typed_string(&item.name))?;

            if let Some(completion_date) = item.completion_date {
                builder.add_kw(&kw!(:item/completion_date), completion_date.to_typed_value())?;
            }

            builder.commit()?;
        }
        Ok(item_uuid)
    }

    pub fn update_item_by_uuid(&mut self,
                               uuid_string: &str,
                               name: Option<String>,
                               completion_date: Option<Timespec>)
                               -> Result<Item> {
        let uuid = Uuid::parse_str(&uuid_string)?;
        let item =
            self.fetch_item(&uuid)
                .ok()
                .unwrap_or_default()
                .ok_or_else(|| ErrorKind::ItemNotFound(uuid_string.to_string()))?;

        let new_item =
            self.update_item(&item, name, completion_date)
                .and_then(|_| self.fetch_item(&uuid))
                .unwrap_or_default()
                .ok_or_else(|| ErrorKind::ItemNotFound(uuid_string.to_string()))?;

        Ok(new_item)
    }

    pub fn update_item(&mut self,
                       item: &Item, name: Option<String>,
                       completion_date: Option<Timespec>) -> Result<()> {
        let entid = KnownEntid(item.id.to_owned().ok_or_else(|| ErrorKind::ItemNotFound(item.uuid.hyphenated().to_string()))?.id);
        let in_progress = self.connection.begin_transaction()?;
        let mut builder = in_progress.builder().describe(entid);

        if let Some(name) = name {
            if item.name != name {
                builder.add_kw(&kw!(:item/name), TypedValue::typed_string(&name))?;
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

        builder.commit()
               .map_err(|e| e.into())
               .and(Ok(()))
    }

    pub fn fetch_items(&mut self) -> Result<Items> {
        let query = r#"[:find ?eid ?uuid ?name
                        :where
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;

        let rows;
        {
            let in_progress_read = self.connection.begin_read()?;
            rows = in_progress_read
                .q_once(query, None)
                .into_rel_result()
                .map_err(|e| e.into());
        }
        rows.map(|rows| Items::new(rows.into_iter().map(|r| self.item_row_to_item(r)).collect()))
    }

    pub fn fetch_item(&mut self, uuid: &Uuid) -> Result<Option<Item>> {
        let query = r#"[:find [?eid ?uuid ?name]
                        :in ?uuid
                        :where
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;
        let rows;
        {
            let in_progress_read = self.connection.begin_read()?;
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

        let in_progress_read = self.connection.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?uuid), item_id.to_typed_value())]);
        return_date_field(
            in_progress_read
            .q_once(query, args))
    }

    fn item_row_to_item(&mut self, row: Vec<TypedValue>) -> Item {
        let uuid = row[1].clone().to_inner();
        let item;
        {
            item = Item {
                id: row[0].clone().to_inner(),
                uuid: uuid,
                name: row[2].clone().to_inner(),
                completion_date: self.fetch_completion_date_for_item(&uuid).unwrap_or(None),
            }
        }
        item
    }

    pub fn create_and_fetch_item(&mut self, item: &Item) -> Result<Option<Item>> {
        let item_uuid = self.create_item(&item)?;
        let item = self.fetch_item(&item_uuid);
        item
    }
}

#[cfg(test)]
mod test {
    use super::{
        Toodle,
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

    fn toodle() -> Toodle {
        Toodle::new(String::new()).expect("Expected a Toodle")
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
        assert!(assert_ident_present(schema, "item", "name"));
    }

    #[test]
    fn test_create_item() {
        let mut manager = toodle();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            completion_date: Some(date.clone())
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        let completion_date = item.completion_date.expect("expecting a completion date");
        assert_eq!(completion_date.sec, date.sec);
    }

    #[test]
    fn test_create_item_no_completion_date() {
        let mut manager = toodle();

        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            completion_date: None
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        assert_eq!(item.completion_date, i.completion_date);
    }

    #[test]
    fn test_fetch_item() {
        let mut manager = toodle();
        let mut created_item = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            completion_date: None
        };

        created_item.uuid = manager.create_item(&created_item).expect("expected a uuid");
        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item.uuid, created_item.uuid);
        assert_eq!(fetched_item.name, created_item.name);
        assert_eq!(fetched_item.completion_date, created_item.completion_date);

        let tmp_uuid = create_uuid().hyphenated().to_string();
        let item_uuid = Uuid::parse_str(&tmp_uuid).unwrap();
        let fetched_item = manager.fetch_item(&item_uuid).expect("expected an item option");
        assert_eq!(fetched_item, None);
    }

    #[test]
    fn test_update_item_change_name() {
        let mut manager = toodle();

        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            completion_date: None,
        };

        let mut created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, Some("new name".to_string()), None) {
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

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            completion_date: None,
        };

        let created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, None, Some(date)) {
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
