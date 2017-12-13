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
use uuid::Uuid;
use items::{
    Item, Items
};

#[repr(C)]
#[derive(Debug)]
pub struct ItemC {
    pub name: *mut c_char
}

impl From<Item> for ItemC {
    fn from(item: Item) -> Self {
        ItemC {
            name: string_to_c_char(item.name.clone())
        }
    }
}

pub struct ItemsC {
    pub vec: Vec<ItemC>
}

impl ItemsC {
    pub fn new(vec: Vec<ItemC>) -> ItemsC {
        ItemsC {
            vec: vec
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
        Item {
            id: None,
            uuid: Uuid::nil(),
            name: c_char_to_string(item_c.name),
            due_date: None,
            completion_date: None,
            labels: vec![]
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ItemCList {
    pub items: Box<[ItemC]>
}
