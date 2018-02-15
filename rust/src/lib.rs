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

extern crate ffi_utils;
extern crate libc;
extern crate mentat_core;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use std::ffi::CString;
use std::os::raw::c_char;

use ffi_utils::log;
use ffi_utils::strings::{
    c_char_to_string,
    optional_timespec,
};

use libc::{
    c_int,
    time_t,
};

use mentat::{
    HasSchema,
    InProgress,
    IntoResult,
    Queryable,
    QueryExecutionResult,
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
    TermBuilder,
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
pub mod ctypes;
mod utils;

use errors::{
    ErrorKind,
    Result,
};

use items::{
    Item,
    Items,
};

use ctypes::{
    ItemC,
    ItemsC,
    ItemCList
};

use utils::{
    ToInner,
    ToTypedValue,
};

// TODO this is pretty horrible and rather crafty, but I couldn't get this to live
// inside a Toodle struct and be able to mutate it...
static mut CHANGED_CALLBACK: Option<extern fn()> = None;

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
            }
        }
        item
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

    fn fetch_due_date_for_item(&mut self, item_id: &Uuid) -> Result<Option<Timespec>> {
        let query = r#"[:find ?date .
            :in ?uuid
            :where
            [?eid :item/uuid ?uuid]
            [?eid :item/due_date ?date]
        ]"#;
        let in_progress_read = self.connection.begin_read()?;
        let args = QueryInputs::with_value_sequence(vec![(var!(?uuid), item_id.to_typed_value())]);
        let date = return_date_field(
            in_progress_read
            .q_once(query, args));
        date
    }

    pub fn create_item(&mut self, item: &Item) -> Result<Uuid> {
        let item_uuid = create_uuid();
        {
            let in_progress = self.connection.begin_transaction()?;
            let mut builder = in_progress.builder().describe_tempid("item");

            builder.add_kw(&kw!(:item/uuid), TypedValue::Uuid(item_uuid))?;
            builder.add_kw(&kw!(:item/name), TypedValue::typed_string(&item.name))?;

            if let Some(due_date) = item.due_date {
                builder.add_kw(&kw!(:item/due_date), due_date.to_typed_value())?;
            }
            if let Some(completion_date) = item.completion_date {
                builder.add_kw(&kw!(:item/completion_date), completion_date.to_typed_value())?;
            }

            builder.commit()?;
        }
        Ok(item_uuid)
    }

    pub fn create_and_fetch_item(&mut self, item: &Item) -> Result<Option<Item>> {
        let item_uuid = self.create_item(&item)?;
        let item = self.fetch_item(&item_uuid);
        item
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
            self.update_item(&item, name, due_date, completion_date)
                .and_then(|_| self.fetch_item(&uuid))
                .unwrap_or_default()
                .ok_or_else(|| ErrorKind::ItemNotFound(uuid_string.to_string()))?;

        Ok(new_item)
    }

    pub fn update_item(&mut self,
                       item: &Item, name: Option<String>,
                       due_date: Option<Timespec>,
                       completion_date: Option<Timespec>) -> Result<()> {
        let entid = KnownEntid(item.id.to_owned().ok_or_else(|| ErrorKind::ItemNotFound(item.uuid.hyphenated().to_string()))?.id);
        let in_progress = self.connection.begin_transaction()?;
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
        builder.commit()
               .map_err(|e| e.into())
               .and(Ok(()))
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
    let manager = &mut*manager;

    if let Ok(Some(i)) = manager.fetch_item(&uuid) {
        let c_item: ItemC = i.into();
        return Box::into_raw(Box::new(c_item));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn toodle_update_item(manager: *mut Toodle, item: *const Item, name: *const c_char, due_date: *const time_t, completion_date: *const time_t) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    let item = &*item;
    let _ = manager.update_item(
        &item,
        Some(name),
        optional_timespec(due_date),
        optional_timespec(completion_date)
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
            due_date: Some(date.clone()),
            completion_date: Some(date.clone())
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        let due_date = item.due_date.expect("expecting a due date");
        assert_eq!(due_date.sec, date.sec);
        let completion_date = item.completion_date.expect("expecting a completion date");
        assert_eq!(completion_date.sec, date.sec);
    }

    #[test]
    fn test_create_item_no_due_date() {
        let mut manager = toodle();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: None,
            completion_date: Some(date.clone()),
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        assert_eq!(item.due_date, i.due_date);
        let completion_date = item.completion_date.expect("expecting a completion date");
        assert_eq!(completion_date.sec, date.sec);
    }

    #[test]
    fn test_create_item_no_completion_date() {
        let mut manager = toodle();

        let date = now_utc().to_timespec();
        let i = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: Some(date.clone()),
            completion_date: None
        };

        let item = manager.create_and_fetch_item(&i).expect("expected an item option").expect("expected an item");
        assert!(!item.uuid.is_nil());
        assert_eq!(item.name, i.name);
        let due_date = item.due_date.expect("expecting a due date");
        assert_eq!(due_date.sec, date.sec);
        assert_eq!(item.completion_date, i.completion_date);
    }

    #[test]
    fn test_fetch_item() {
        let mut manager = toodle();
        let mut created_item = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item".to_string(),
            due_date: None,
            completion_date: None
        };

        created_item.uuid = manager.create_item(&created_item).expect("expected a uuid");
        let fetched_item = manager.fetch_item(&created_item.uuid).expect("expected an item option").expect("expected an item");
        assert_eq!(fetched_item.uuid, created_item.uuid);
        assert_eq!(fetched_item.name, created_item.name);
        assert_eq!(fetched_item.due_date, created_item.due_date);
        assert_eq!(fetched_item.completion_date, created_item.completion_date);

        let tmp_uuid = create_uuid().hyphenated().to_string();
        let item_uuid = Uuid::parse_str(&tmp_uuid).unwrap();
        let fetched_item = manager.fetch_item(&item_uuid).expect("expected an item option");
        assert_eq!(fetched_item, None);
    }

    #[test]
    fn test_update_item_add_due_date() {
        let mut manager = toodle();

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: None,
            completion_date: None,
        };

        let created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");

        match manager.update_item(&created_item, None, Some(date), None) {
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

        let date = now_utc().to_timespec();
        let item1 = Item {
            id: None,
            uuid: Uuid::nil(),
            name: "test item 1".to_string(),
            due_date: Some(date),
            completion_date: None,
        };

        let mut created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, Some("new name".to_string()), None, None) {
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
            due_date: None,
            completion_date: None,
        };

        let created_item = manager.create_and_fetch_item(&item1).expect("expected an item option").expect("expected an item");
        match manager.update_item(&created_item, None, None, Some(date)) {
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
