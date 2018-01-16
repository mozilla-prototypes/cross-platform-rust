extern crate ffi_utils;
extern crate libc;
extern crate mentat_core;
extern crate time;
extern crate toodle;

mod ctypes;

use libc::{ c_int, size_t, time_t };
use mentat_core::Uuid;
use std::ffi::CString;
use std::os::raw::{
    c_char,
};
use std::ptr;
use std::str::FromStr;
use time::Timespec;

use toodle::{
    Item,
    Label,
    Toodle,
};
use ctypes::{
    ItemC,
    ItemsC,
    ItemCList
};
use ffi_utils::strings::{
    c_char_to_string,
    optional_timespec,
    string_to_c_char,
};
use ffi_utils::log;


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
pub unsafe extern "C" fn toodle_get_all_labels(manager: *const Toodle) -> *mut Vec<Label> {
    let manager = &*manager;
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
    let manager = &*manager;
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
pub unsafe extern "C" fn toodle_update_item(manager: *mut Toodle, item: *const Item, name: *const c_char, due_date: *const time_t, completion_date: *const time_t, labels: *const Vec<Label>) {
    let manager = &mut*manager;
    let item = &*item;
    let labels = &*labels;
    let _ = manager.update_item(
        &item,
        Some(c_char_to_string(name)),
        optional_timespec(due_date),
        optional_timespec(completion_date),
        Some(&labels)
    );
}

#[no_mangle]
pub unsafe extern "C" fn toodle_update_item_by_uuid(manager: *mut Toodle, uuid: *const c_char, name: *const c_char, due_date: *const time_t, completion_date: *const time_t) {
    let manager = &mut*manager;
    // TODO proper error handling, see https://github.com/mozilla-prototypes/sync-storage-prototype/pull/6
    let item = manager.fetch_item(
        &Uuid::from_str(c_char_to_string(uuid).as_str()).expect("parsed uuid")
    ).expect("item from uuid").unwrap();
    let _ = manager.update_item(
        &item,
        Some(c_char_to_string(name)),
        optional_timespec(due_date),
        optional_timespec(completion_date),
        Some(&item.labels)
    );

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
pub unsafe extern "C" fn label_destroy(label: *mut Label) {
    let _ = Box::from_raw(label);
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
pub extern "C" fn item_new() -> *mut Item {
    let item = Item::default();
    let boxed_item = Box::new(item);
    Box::into_raw(boxed_item)
}

#[no_mangle]
pub unsafe extern "C" fn item_destroy(item: *mut Item) {
    let _ = Box::from_raw(item);
}

#[no_mangle]
pub unsafe extern "C" fn item_get_name(item: *const Item) -> *mut c_char {
    let item = &*item;
    string_to_c_char(item.name.clone())
}

#[no_mangle]
pub unsafe extern "C" fn item_set_name(item: *mut Item, name: *const c_char) {
    let item = &mut*item;
    item.name = c_char_to_string(name);
}

#[no_mangle]
pub unsafe extern "C" fn item_get_due_date(item: *const Item) -> *mut i64 {
    let item = &*item;
    match item.due_date {
        Some(date) => {
            Box::into_raw(Box::new(date.sec))
        },
        None => {
            ptr::null_mut()
        }
    }

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
pub unsafe extern "C" fn item_get_completion_date(item: *const Item) -> *mut i64 {
    let item = &*item;
    match item.completion_date {
        Some(date) => {
            Box::into_raw(Box::new(date.sec))
        },
        None => {
            ptr::null_mut()
        }
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