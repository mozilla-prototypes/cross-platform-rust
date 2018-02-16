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
extern crate time;
extern crate toodle;

#[doc(hidden)]
pub mod android_ffi;
mod ctypes;
mod utils;

use libc::{ c_int, size_t, time_t };
use mentat::Uuid;
use std::ffi::CString;
use std::os::raw::{
    c_char,
};
use time::Timespec;

use toodle::{
    Item,
    Toodle,
};
use ctypes::{
    ItemC,
    ItemsC,
    ItemCList
};
use utils::strings::{
    c_char_to_string,
    optional_timespec,
};
use utils::log;


// TODO this is pretty horrible and rather crafty, but I couldn't get this to live
// inside a Toodle struct and be able to mutate it...
static mut CHANGED_CALLBACK: Option<extern fn()> = None;

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
pub unsafe extern "C" fn toodle_create_item(manager: *mut Toodle, name: *const c_char) -> *mut ItemC {
    let name = c_char_to_string(name);
    log::d(&format!("Creating item: {:?}, {:?}", name, manager)[..]);

    let manager = &mut*manager;
    let mut item = Item::default();
    item.name = name;

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
pub unsafe extern "C" fn toodle_update_item(manager: *mut Toodle, item: *const Item, name: *const c_char, completion_date: *const time_t) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    let item = &*item;
    let _ = manager.update_item(
        &item,
        Some(name),
        optional_timespec(completion_date)
    );
}

#[no_mangle]
pub unsafe extern "C" fn toodle_update_item_by_uuid(manager: *mut Toodle, uuid: *const c_char, name: *const c_char, completion_date: *const time_t) {
    let name = c_char_to_string(name);
    let manager = &mut*manager;
    // TODO proper error handling, see https://github.com/mozilla-prototypes/sync-storage-prototype/pull/6
    let _ = manager.update_item_by_uuid(c_char_to_string(uuid).as_str(),
                                        Some(name),
                                        optional_timespec(completion_date));

    if let Some(callback) = CHANGED_CALLBACK {
        callback();
    }
}

#[no_mangle]
pub unsafe extern "C" fn item_set_name(item: *mut Item, name: *const c_char) {
    let item = &mut*item;
    item.name = c_char_to_string(name);
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
