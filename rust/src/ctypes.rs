// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use ffi_utils::strings::{
    string_to_c_char,
    c_char_to_string,
};
use std::os::raw::c_char;
use std::ptr;

use mentat_core::Uuid;
use time::Timespec;

use items::{
    Item, Items
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ItemC {
    pub uuid: *mut c_char,
    pub name: *mut c_char,
    pub due_date: *mut i64,
    pub completion_date: *mut i64,
}

impl From<Item> for ItemC {
    fn from(item: Item) -> Self {
        let due = match item.due_date {
            Some(date) => {
                Box::into_raw(Box::new(date.sec))
            },
            None => {
                ptr::null_mut()
            }
        };
        let completion = match item.completion_date {
            Some(date) => {
                Box::into_raw(Box::new(date.sec))
            },
            None => {
                ptr::null_mut()
            }
        };
        ItemC {
            uuid: string_to_c_char(item.uuid.hyphenated().to_string()),
            name: string_to_c_char(item.name.clone()),
            due_date: due,
            completion_date: completion,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemsC {
    pub vec: Vec<ItemC>,
    pub len: usize,
}

impl ItemsC {
    pub fn new(vec: Vec<ItemC>) -> ItemsC {
        let len = vec.len();
        ItemsC {
            vec: vec,
            len: len
        }
    }
}

impl From<Items> for ItemsC {
    fn from(items: Items) -> Self {
        ItemsC::new(items.vec.into_iter().map(|item| item.into()).collect())
    }
}

impl From<Vec<Item>> for ItemsC {
    fn from(items: Vec<Item>) -> Self {
        ItemsC::new(items.into_iter().map(|item| item.into()).collect())
    }
}

impl From<ItemC> for Item {
    fn from(item_c: ItemC) -> Self {
        let uuid = Uuid::parse_str(&c_char_to_string(item_c.uuid)).unwrap_or(Uuid::default());
        let due: Option<Timespec>;
        if !item_c.due_date.is_null() {
            due = Some(Timespec::new(item_c.due_date as i64, 0));
        } else {
            due = None;
        }
        let completion: Option<Timespec>;
        if !item_c.completion_date.is_null() {
            completion = Some(Timespec::new(item_c.completion_date as i64, 0));
        } else {
            completion = None;
        }
        Item {
            id: None,
            uuid: uuid,
            name: c_char_to_string(item_c.name),
            due_date: due,
            completion_date: completion,
            labels: vec![]
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ItemCList {
    pub items: Box<[ItemC]>,
    pub len: usize
}
