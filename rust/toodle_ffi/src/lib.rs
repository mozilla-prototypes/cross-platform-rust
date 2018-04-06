// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

extern crate libc;
extern crate mentat;
extern crate mentat_ffi;
extern crate time;
extern crate toodle;

mod ctypes;
mod utils;

use libc::{ c_int, size_t, time_t };
use std::ffi::CString;
use std::os::raw::{
    c_char,
};

use time::Timespec;

pub use mentat::{
    Store,
    Uuid,
};

pub use mentat_ffi::{
    store_entid_for_attribute,
    store_register_observer,
    store_unregister_observer,
};
use mentat_ffi::utils::log;
use mentat_ffi::utils::strings::{
    c_char_to_string,
    string_to_c_char,
};

use toodle::{
    Item,
    Label,
    Toodle,
};
use ctypes::{
    ItemC,
    ItemsC,
    ItemCList,
};
use utils::time::{
    optional_timespec,
};

#[no_mangle]
pub extern "C" fn new_toodle(uri: *const c_char) -> *mut Store {
    let uri = c_char_to_string(uri);
    log::d(&format!("db uri: {:?}", uri));
    let mut store = Store::open(&uri).expect("expected a store");
    store.initialize().expect("Expected store to initialize");
    log::d(&format!("init the store, schema: {:?}", store.conn().current_schema()));
    Box::into_raw(Box::new(store))
}

#[no_mangle]
pub unsafe extern "C" fn toodle_get_all_labels(manager: *mut Store) -> *mut Vec<Label> {
    let manager = &mut*manager;
    let label_list = Box::new(manager.fetch_labels().unwrap_or(vec![]));
    Box::into_raw(label_list)
}

#[no_mangle]
pub unsafe extern "C" fn toodle_create_item(manager: *mut Store, name: *const c_char, due_date: *const time_t) -> *mut ItemC {
    let name = c_char_to_string(name);
    log::d(&format!("Creating item: {:?}, {:?}, {:?}", name, due_date, manager)[..]);

    let manager = &mut*manager;
    let mut item = Item::default();

    log::d(&format!("toodle_create_item default item: {:?}", item));

    item.name = name;
    let due: Option<Timespec>;
    if !due_date.is_null() {
        let due_date = *due_date as i64;
        due = Some(Timespec::new(due_date, 0));
    } else {
        due = None;
    }
    item.due_date = due;
    log::d(&format!("toodle_create_item due item: {:?}", item));
    let item = manager.create_and_fetch_item(&item).expect("expected an item");
    log::d(&format!("toodle_create_item create_and_fetch_item: {:?}", item));
    if let Some(i) = item {
        return Box::into_raw(Box::new(i.into()));
    }
    return std::ptr::null_mut();
}

// TODO: figure out callbacks in swift such that we can use `toodle_all_items` instead.
#[no_mangle]
pub unsafe extern "C" fn toodle_get_all_items(manager: *mut Store) -> *mut ItemCList {
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
pub unsafe extern "C" fn toodle_all_items(manager: *mut Store, callback: extern "C" fn(Option<&ItemCList>)) {
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

#[no_mangle]
pub unsafe extern "C" fn toodle_item_for_uuid(manager: *mut Store, uuid: *const c_char) -> *mut ItemC {
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
pub unsafe extern "C" fn toodle_update_item(manager: *mut Store, item: *const Item, name: *const c_char, due_date: *const time_t, completion_date: *const time_t, labels: *const Vec<Label>) {
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
pub unsafe extern "C" fn toodle_update_item_by_uuid(manager: *mut Store, uuid: *const c_char, name: *const c_char, due_date: *const time_t, completion_date: *const time_t) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    // TODO proper error handling, see https://github.com/mozilla-prototypes/sync-storage-prototype/pull/6
    let _ = manager.update_item_by_uuid(c_char_to_string(uuid).as_str(),
                                        Some(name),
                                        optional_timespec(due_date),
                                        optional_timespec(completion_date));

    // if let Some(callback) = CHANGED_CALLBACK {
    //     callback();
    // }
}

#[no_mangle]
pub unsafe extern "C" fn toodle_create_label(manager: *mut Store, name: *const c_char, color: *const c_char) -> *mut Option<Label> {
    let manager = &mut*manager;
    let name = c_char_to_string(name);
    let color = c_char_to_string(color);
    let label = Box::new(manager.create_label(name, color).unwrap_or(None));
    Box::into_raw(label)
}

#[no_mangle]
pub unsafe extern "C" fn label_get_name(label: *const Label) -> *mut c_char {
    let label = &*label;
    string_to_c_char(label.name.clone())
}

#[no_mangle]
pub unsafe extern "C" fn label_get_color(label: *const Label) -> *mut c_char {
    let label = &*label;
    string_to_c_char(label.color.clone())
}

#[no_mangle]
pub unsafe extern "C" fn label_set_color(label: *mut Label, color: *const c_char) {
    let label = &mut*label;
    label.color = c_char_to_string(color);
}

#[no_mangle]
pub unsafe extern "C" fn item_set_name(item: *mut Item, name: *const c_char) {
    let item = &mut*item;
    item.name = c_char_to_string(name);
}

#[no_mangle]
pub unsafe extern "C" fn item_set_due_date(item: *mut Item, due_date: *const size_t) {
    let item = &mut*item;
    if !due_date.is_null() {
        item.due_date = Some(Timespec::new(due_date as i64, 0));
    } else {
        item.due_date = None;
    }
}

#[no_mangle]
pub unsafe extern "C" fn item_set_completion_date(item: *mut Item, completion_date: *const size_t) {
    let item = &mut*item;
    if !completion_date.is_null() {
        item.completion_date = Some(Timespec::new(completion_date as i64, 0));
    } else {
        item.completion_date = None;
    }
}

#[no_mangle]
pub unsafe extern "C" fn item_get_labels(item: *const Item) -> *mut Vec<Label> {
    let item = &*item;
    let boxed_labels = Box::new(item.labels.clone());
    Box::into_raw(boxed_labels)
}

#[no_mangle]
pub unsafe extern "C" fn item_labels_count(item: *const Item) -> c_int {
    let item = &*item;
    item.labels.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn item_label_at(label_list: *const Vec<Label>, index: size_t) -> *const Label {
    let label_list = &*label_list;
    let index = index as usize;
    let label = Box::new(label_list[index].clone());
    Box::into_raw(label)
}

