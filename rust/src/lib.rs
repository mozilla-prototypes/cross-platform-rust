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

#[macro_use(kw)]
extern crate mentat;

extern crate ffi_utils;
extern crate libc;
extern crate rusqlite;
extern crate time;
extern crate uuid;

extern crate store;

use std::ffi::CString;
use std::os::raw::c_char;
use std::str::FromStr;
use std::error::Error;

use ffi_utils::log;
use ffi_utils::strings::{
    c_char_to_string,
    string_to_c_char,
    optional_timespec,
};

use libc::{
    c_int,
    time_t,
};

use mentat::{
    InProgress,
    IntoResult,
    Queryable,
    QueryExecutionResult,
    QueryInputs,
    TypedValue,
    ValueType,
    Variable,
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

pub mod labels;
pub mod items;
pub mod errors;
pub mod ctypes;

use errors::{
    ErrorKind,
    Result,
};

use labels::Label;

use items::{
    Item,
    Items,
};

use ctypes::{
    ItemC,
    ItemsC,
    ItemCList
};

use store::{
    new_store,
    Store,
    ToInner,
    ToTypedValue,
};

// TODO this is pretty horrible and rather crafty, but I couldn't get this to live
// inside a Toodle struct and be able to mutate it...
static mut CHANGED_CALLBACK: Option<extern fn()> = None;

fn transact_items_vocabulary(in_progress: &mut InProgress) -> Result<()> {
    in_progress.ensure_vocabulary(&Definition {
            name: kw!(:example/links),
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
            name: kw!(:example/links),
            version: 1,
            attributes: vec![
                (kw!(:label/name),
                 AttributeBuilder::new()
                    .value_type(ValueType::String)
                    .multival(false)
                    .unique(Unique::Identity)
                    .fulltext(true)
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

#[repr(C)]
pub struct Toodle {
    connection: Store,
}

impl Toodle {
    pub fn new(uri: String) -> Result<Toodle> {
        let mut store_result = new_store(uri)?;
        {
            // TODO proper error handling at the FFI boundary
            let mut in_progress = store_result.begin_transaction()?;
            in_progress.verify_core_schema()?;
            transact_labels_vocabulary(&mut in_progress)?;
            transact_items_vocabulary(&mut in_progress)?;
            in_progress.commit()?;
        }

        let toodle = Toodle {
            connection: store_result,
        };

        Ok(toodle)
    }
}

fn create_uuid() -> Uuid {
    uuid::Uuid::new_v4()
}

fn return_date_field(results: QueryExecutionResult) -> Result<Option<Timespec>> {
    results.into_scalar_result()
           .map(|o| o.and_then(|ts| ts.to_inner()))
           .map_err(|e| e.into())
}

impl Toodle {
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

    pub fn create_label(&mut self, name: String, color: String) -> Result<Option<Label>> {
        // TODO: better transact API.
        let query = format!("[{{ :label/name \"{0}\" :label/color \"{1}\" }}]", &name, &color);
        {
            let mut in_progress = self.connection.begin_transaction()?;
            in_progress.transact(&query)?;
            in_progress.commit()?;
        }
        self.fetch_label(&name)
    }

    pub fn fetch_label(&mut self, name: &String) -> Result<Option<Label>> {
        let query = r#"[:find [?eid ?name ?color]
                        :in ?name
                        :where
                        [?eid :label/name ?name]
                        [?eid :label/color ?color]
        ]"#;
        let in_progress_read = self.connection.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?name"), name.to_typed_value())]);
        in_progress_read
            .q_once(query, args)
            .into_tuple_result()
            .map(|o| o.as_ref().and_then(Label::from_row))
            .map_err(|e| e.into())
    }

    pub fn fetch_labels(&mut self) -> Result<Vec<Label>> {
        let query = r#"[:find ?eid ?name ?color
                        :where
                        [?eid :label/name ?name]
                        [?eid :label/color ?color]
        ]"#;
        let in_progress_read = self.connection.begin_read()?;
        in_progress_read
            .q_once(query, None)
            .into_rel_result()
            .map(|rows| rows.iter().filter_map(|row| Label::from_row(&row)).collect())
            .map_err(|e| e.into())
    }

    pub fn fetch_labels_for_item(&mut self, item_uuid: &Uuid) -> Result<Vec<Label>> {
        let query = r#"[:find ?l ?name ?color
                        :in ?item_uuid
                        :where
                        [?i :item/uuid ?item_uuid]
                        [?i :item/label ?l]
                        [?l :label/name ?name]
                        [?l :label/color ?color]
        ]"#;
        let in_progress_read = self.connection.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?item_uuid"), item_uuid.to_typed_value())]);
        in_progress_read
            .q_once(query, args)
            .into_rel_result()
            .map(|rows| rows.iter().filter_map(|row| Label::from_row(&row)).collect())
            .map_err(|e| e.into())
    }


    pub fn fetch_items_with_label(&mut self, label: &Label) -> Result<Vec<Item>> {
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
            let in_progress_read = self.connection.begin_read()?;
            let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?label"), label.name.to_typed_value())]);
            rows = in_progress_read
                .q_once(query, args)
                .into_rel_result()
                .map_err(|e| e.into());
        }
        rows.map(|rows| rows.into_iter().map(|r| self.item_row_to_item(r)).collect())
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

    pub fn fetch_item(&mut self, uuid: &Uuid) -> Result<Option<Item>>{
        let query = r#"[:find [?eid ?uuid ?name]
                        :in ?uuid
                        :where
                        [?eid :item/uuid ?uuid]
                        [?eid :item/name ?name]
        ]"#;
        let rows;
        {
            let in_progress_read = self.connection.begin_read()?;
            let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?uuid"), uuid.to_typed_value())]);
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
        let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?uuid"), item_id.to_typed_value())]);
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
        let in_progress_read = self.connection.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(Variable::from_valid_name("?uuid"), item_id.to_typed_value())]);
        let date = return_date_field(
            in_progress_read
            .q_once(query, args));
        date
    }

    pub fn create_item(&mut self, item: &Item) -> Result<Uuid> {
        // TODO: make this mapping better!
        let label_str = item.labels
                            .iter()
                            .filter(|label| label.id.is_some() )
                            .map(|label|  format!("{}", label.id.clone().map::<i64, _>(|e| e.into()).unwrap()) )
                            .collect::<Vec<String>>()
                            .join(", ");
        let item_uuid = create_uuid();
        let uuid_string = item_uuid.hyphenated().to_string();
        let mut query = format!(r#"[{{
            :item/uuid #uuid {:?}
            :item/name {:?}
            "#, &uuid_string, &(item.name));
        if let Some(due_date) = item.due_date {
            let micro_seconds = due_date.sec * 1000000;
            query = format!(r#"{}:item/due_date #instmicros {}
                "#, &query, &micro_seconds);
        }
        if let Some(completion_date) = item.completion_date {
            let micro_seconds = completion_date.sec * 1000000;
            query = format!(r#"{}:item/completion_date #instmicros {}
                "#, &query, &micro_seconds);
        }
        if !label_str.is_empty() {
            query = format!(r#"{0}:item/label [{1}]
                "#, &query, &label_str);
        }
        query = format!("{0}}}]", &query);
        let mut in_progress = self.connection.begin_transaction()?;
        in_progress.transact(&query)?;
        in_progress.commit()?;
        Ok(item_uuid)
    }

    pub fn create_and_fetch_item(&mut self, item: &Item) -> Result<Option<Item>> {
        let item_uuid = self.create_item(&item)?;
        self.fetch_item(&item_uuid)
    }

    pub fn update_item_by_uuid(&mut self,
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

    pub fn update_item(&mut self,
                       item: &Item, name: Option<String>,
                       due_date: Option<Timespec>,
                       completion_date: Option<Timespec>,
                       labels: Option<&Vec<Label>>) -> Result<()> {
        let item_id = item.id.to_owned().expect("item must have ID to be updated");
        let mut transaction = vec![];

        if let Some(name) = name {
            if item.name != name {
                transaction.push(format!("[:db/add {0} :item/name \"{1}\"]", &item_id.id, name));
            }
        }
        if item.due_date != due_date {
            if let Some(date) = due_date {
                let micro_seconds = date.sec * 1000000;
                transaction.push(format!("[:db/add {:?} :item/due_date #instmicros {}]", &item_id.id, &micro_seconds));
            } else {
                let micro_seconds = item.due_date.unwrap().sec * 1000000;
                transaction.push(format!("[:db/retract {:?} :item/due_date #instmicros {}]", &item_id.id, &micro_seconds));
            }
        }

        if item.completion_date != completion_date {
            if let Some(date) = completion_date {
                let micro_seconds = date.sec * 1000000;
                transaction.push(format!("[:db/add {:?} :item/completion_date #instmicros {}]", &item_id.id, &micro_seconds));
            } else {
                let micro_seconds = item.completion_date.unwrap().sec * 1000000;
                transaction.push(format!("[:db/retract {:?} :item/completion_date #instmicros {}]", &item_id.id, &micro_seconds));
            }
        }

        if let Some(new_labels) = labels {
            let existing_labels = self.fetch_labels_for_item(&(item.uuid)).unwrap_or(vec![]);
            let labels_to_add = new_labels.iter()
                                        .filter(|label| !existing_labels.contains(label) && label.id.is_some() )
                                        .map(|label|  format!("{}", label.id.clone().map::<i64, _>(|e| e.into()).unwrap()) )
                                        .collect::<Vec<String>>()
                                        .join(", ");
            if !labels_to_add.is_empty() {
                transaction.push(format!("[:db/add {0} :item/label [{1}]]", &item_id.id, labels_to_add));
            }
            let labels_to_remove = existing_labels.iter()
                                        .filter(|label| !new_labels.contains(label) && label.id.is_some() )
                                        .map(|label|  format!("{}", label.id.clone().map::<i64, _>(|e| e.into()).unwrap()) )
                                        .collect::<Vec<String>>()
                                        .join(", ");
            if !labels_to_remove.is_empty() {
                transaction.push(format!("[:db/retract {0} :item/label [{1}]]", &item_id.id, labels_to_remove));
            }
        }

        // TODO: better transact API.
        let query = format!("[{0}]", transaction.join(""));

        let mut in_progress = self.connection.begin_transaction()?;
        in_progress.transact(&query)?;
        in_progress.commit()
                   .map_err(|e| e.into())
                   .and(Ok(()))
    }

    pub fn sync(&mut self, user_uuid: &Uuid) -> Result<()> {
        // TODO this feels like a natural way to expose sync, but we'll see what mentat does.
        // self.connection.sync(user_uuid)
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn new_toodle(uri: *const c_char) -> *mut Toodle {
    let uri = c_char_to_string(uri);
    let toodle = Toodle::new(uri).expect("expected a toodle");
    Box::into_raw(Box::new(toodle))
}

#[no_mangle]
pub unsafe extern "C" fn toodle_destroy(toodle: *mut Toodle) {
    let _ = Box::from_raw(toodle);
}

#[no_mangle]
pub unsafe extern "C" fn toodle_get_all_labels(manager: *mut Toodle) -> *mut Vec<Label> {
    let manager = &mut*manager;
    let label_list = Box::new(manager.fetch_labels().unwrap_or(vec![]));
    Box::into_raw(label_list)
}

#[no_mangle]
pub unsafe extern "C" fn toodle_create_item(manager: *mut Toodle, name: *const c_char, due_date: *const time_t) -> *mut ItemC {
    let name = c_char_to_string(name);
    log::d(&format!("Creating item: {:?}, {:?}, {:?}", name, due_date, manager)[..]);

    let manager = &mut*manager;
    let mut item = Item::default();

    item.name = name;
    let due: Option<Timespec>;
    if !due_date.is_null() {
        let due_date = *due_date as i64;
        due = Some(Timespec::new(due_date, 0));
    } else {
        due = None;
    }
    item.due_date = due;
    let item = manager.create_and_fetch_item(&item).expect("expected an item");
    if let Some(callback) = CHANGED_CALLBACK {
        callback();
    }
    if let Some(i) = item {
        return Box::into_raw(Box::new(i.into()));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn toodle_on_items_changed(callback: extern fn()) {
    CHANGED_CALLBACK = Some(callback);
    callback();
}

// TODO: figure out callbacks in swift such that we can use `toodle_all_items` instead.
#[no_mangle]
pub unsafe extern "C" fn toodle_get_all_items(manager: *mut Toodle) -> *mut ItemCList {
    let manager = &mut *manager;
    let items: ItemsC = manager.fetch_items().map(|item| item.into()).expect("all items");
    let count = items.vec.len();
    let item_list = ItemCList {
        items: items.vec.into_boxed_slice(),
        len: count,
    };

    Box::into_raw(Box::new(item_list))
}

#[no_mangle]
pub unsafe extern "C" fn item_list_entry_at(item_c_list: *mut ItemCList, index: c_int) -> *const ItemC {
    let item_c_list = &*item_c_list;
    let index = index as usize;
    let item = Box::new(item_c_list.items[index].clone());
    Box::into_raw(item)
}

#[no_mangle]
pub unsafe extern "C" fn item_list_count(item_list: *mut ItemCList) -> c_int {
    let item_list = &*item_list;
    item_list.len as c_int
}

#[no_mangle]
pub unsafe extern "C" fn toodle_all_items(manager: *mut Toodle, callback: extern "C" fn(Option<&ItemCList>)) {
    let manager = &mut*manager;
    let items: ItemsC = manager.fetch_items().map(|item| item.into()).expect("all items");

    // TODO there's bound to be a better way. Ideally this should just return an empty set,
    // but I ran into problems while doing that.
    let count = items.vec.len();

    let set = ItemCList {
        items: items.vec.into_boxed_slice(),
        len: count,
    };

    let res = match count > 0 {
        // NB: we're lending a set, it will be cleaned up automatically once 'callback' returns
        true => Some(&set),
        false => None
    };

    callback(res);
}


// TODO this is pretty crafty... Currently this setup means that ItemJNA could only be used
// together with something like toodle_all_items - a function that will clear up ItemJNA itself.
#[no_mangle]
pub unsafe extern "C" fn item_c_destroy(item: *mut ItemC) -> *mut ItemC {
    let item = Box::from_raw(item);

    // Reclaim our strings and let Rust clear up their memory.
    let _ = CString::from_raw(item.uuid);
    let _ = CString::from_raw(item.name);

    // Prevent Rust from clearing out item itself. It's already managed by toodle_all_items.
    // If we'll let Rust clean up entirely here, we'll get an NPE in toodle_all_items.
    Box::into_raw(item)
}

#[no_mangle]
pub unsafe extern "C" fn toodle_item_for_uuid(manager: *mut Toodle, uuid: *const c_char) -> *mut ItemC {
    let uuid_string = c_char_to_string(uuid);
    let uuid = Uuid::parse_str(&uuid_string).unwrap();
    eprintln!("fetching item {:?}", uuid);
    let manager = &mut*manager;

    if let Ok(Some(i)) = manager.fetch_item(&uuid) {
        eprintln!("returning item with uuid {:?}", i.uuid);
        let c_item: ItemC = i.into();
        return Box::into_raw(Box::new(c_item));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn toodle_update_item(manager: *mut Toodle, item: *const Item, name: *const c_char, due_date: *const time_t, completion_date: *const time_t, labels: *const Vec<Label>) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    let item = &*item;
    let labels = &*labels;
    let _ = manager.update_item(
        &item,
        Some(name),
        optional_timespec(due_date),
        optional_timespec(completion_date),
        Some(&labels)
    );
}

#[no_mangle]
pub unsafe extern "C" fn toodle_update_item_by_uuid(manager: *mut Toodle, uuid: *const c_char, name: *const c_char, due_date: *const time_t, completion_date: *const time_t) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    // TODO proper error handling, see https://github.com/mozilla-prototypes/sync-storage-prototype/pull/6
    let _ = manager.update_item_by_uuid(c_char_to_string(uuid).as_str(),
                                        Some(name),
                                        optional_timespec(due_date),
                                        optional_timespec(completion_date));

    if let Some(callback) = CHANGED_CALLBACK {
        callback();
    }
}

#[no_mangle]
pub unsafe extern "C" fn toodle_create_label(manager: *mut Toodle, name: *const c_char, color: *const c_char) -> *mut Option<Label> {
    let manager = &mut*manager;
    let name = c_char_to_string(name);
    let color = c_char_to_string(color);
    let label = Box::new(manager.create_label(name, color).unwrap_or(None));
    Box::into_raw(label)
}

#[no_mangle]
pub unsafe extern "C" fn toodle_sync(manager: *mut Toodle, user_uuid: *const c_char) -> *mut ctypes::ResultC {
    let manager = &mut*manager;
    let user_uuid = c_char_to_string(user_uuid);
    match Uuid::from_str(&user_uuid) {
        Ok(uuid) => Box::into_raw(Box::new(manager.sync(&uuid).into())),
        Err(e) => Box::into_raw(Box::new(ctypes::ResultC {
            error: string_to_c_char(e.description().into())
        }))
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
