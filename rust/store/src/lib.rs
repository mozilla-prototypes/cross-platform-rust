// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#[macro_use] extern crate error_chain;

extern crate mentat;
extern crate edn;
extern crate mentat_query;
extern crate mentat_core;
extern crate mentat_db;
extern crate ordered_float;
extern crate rusqlite;
extern crate time;
extern crate ffi_utils;

use std::fmt;
use std::os::raw::{
    c_char
};
use std::rc::Rc;
use std::sync::{
    Arc,
    RwLock,
};

use edn::{
    DateTime,
    FromMicros,
    NamespacedKeyword,
    Utc,
};

use mentat::{
    new_connection,
};

use mentat::conn::Conn;

use mentat_core::{
    Entid,
    TypedValue,
    Uuid,
};

use mentat_db::types::TxReport;

use mentat::query::{
    QueryInputs,
    Variable,
};

use ordered_float::OrderedFloat;

use rusqlite::{
    Connection
};

use time::Timespec;

pub mod errors;

use ffi_utils::strings::c_char_to_string;
use errors as store_errors;

pub trait ToTypedValue {
    fn to_typed_value(&self) -> TypedValue;
}

impl ToTypedValue for String {
    fn to_typed_value(&self) -> TypedValue {
        TypedValue::String(Rc::new(self.clone()))
    }
}

impl<'a> ToTypedValue for &'a str {
    fn to_typed_value(&self) -> TypedValue {
        TypedValue::String(Rc::new(self.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: Entid
}

impl Entity {
    pub fn new(id: Entid) -> Entity {
        Entity { id: id}
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl ToTypedValue for Entity {
    fn to_typed_value(&self) -> TypedValue {
        TypedValue::Ref(self.id.clone())
    }
}

impl Into<Entid> for Entity {
    fn into(self) -> Entid {
        self.id
    }
}

impl ToTypedValue for NamespacedKeyword {
    fn to_typed_value(&self) -> TypedValue {
        self.clone().into()
    }
}

impl ToTypedValue for bool {
    fn to_typed_value(&self) -> TypedValue {
        (*self).into()
    }
}

impl ToTypedValue for i64 {
    fn to_typed_value(&self) -> TypedValue {
        TypedValue::Long(*self)
    }
}

impl ToTypedValue for f64 {
    fn to_typed_value(&self) -> TypedValue {
        TypedValue::Double(OrderedFloat(*self))
    }
}

impl ToTypedValue for Timespec {
    fn to_typed_value(&self) -> TypedValue {
        let micro_seconds = (self.sec * 1000000) + i64::from((self.nsec * 1000));
        TypedValue::Instant(DateTime::<Utc>::from_micros(micro_seconds))
    }
}

impl ToTypedValue for Uuid {
    fn to_typed_value(&self) -> TypedValue {
        self.clone().into()
    }
}

pub trait ToInner<T> {
    fn to_inner(self) -> T;
}

impl ToInner<Option<Entity>> for TypedValue {
    fn to_inner(self) -> Option<Entity> {
        match self {
            TypedValue::Ref(r) => Some(Entity::new(r.clone())),
            _ => None,
        }
    }
}

impl ToInner<Option<i64>> for TypedValue {
    fn to_inner(self) -> Option<i64> {
        match self {
            TypedValue::Long(v) => Some(v),
            _ => None,
        }
    }
}

impl ToInner<String> for TypedValue {
    fn to_inner(self) -> String {
        match self {
            TypedValue::String(s) => s.to_string(),
            _ => String::new(),
        }
    }
}

impl ToInner<Uuid> for TypedValue {
    fn to_inner(self) -> Uuid {
        match self {
            TypedValue::Uuid(u) => u,
            _ => Uuid::nil(),
        }
    }
}

impl ToInner<Option<Timespec>> for TypedValue {
    fn to_inner(self) -> Option<Timespec> {
        match self {
            TypedValue::Instant(v) => {
                let timestamp = v.timestamp();
                Some(Timespec::new(timestamp, 0))
            },
            _ => None,
        }
    }
}

impl<'a> ToInner<Option<Timespec>> for Option<&'a TypedValue> {
    fn to_inner(self) -> Option<Timespec> {
        match self {
            Some(&TypedValue::Instant(v)) => {
                let timestamp = v.timestamp();
                Some(Timespec::new(timestamp, 0))
            },
            _ => None,
        }
    }
}


impl<'a> ToInner<Uuid> for &'a TypedValue {
    fn to_inner(self) -> Uuid {
        match self {
            &TypedValue::Uuid(u) => u,
            _ => Uuid::nil(),
        }
    }
}

#[derive(Debug)]
pub struct StoreConnection {
    pub handle: Connection,
    pub store: Store,
}

impl StoreConnection {
    pub fn query(&self, query: &str) -> mentat::query::QueryExecutionResult {
        self.store.conn.read().unwrap().q_once(&self.handle, query, None)
    }

    pub fn query_args(&self, query: &str, inputs: Vec<(Variable, TypedValue)>) -> mentat::query::QueryExecutionResult {
        let i = QueryInputs::with_value_sequence(inputs);
        self.store.conn.read().unwrap().q_once(&self.handle, query, i)
    }

    pub fn transact(&mut self, transaction: &str) -> Result<TxReport, store_errors::Error> {
        Ok(self.store.conn.write().unwrap().transact(&mut self.handle, transaction)?)
    }

    pub fn fetch_schema(&self) -> edn::Value {
        self.store.conn.read().unwrap().current_schema().to_edn_value()
    }

    pub fn new_connection(&self) -> store_errors::Result<StoreConnection> {
        Ok(StoreConnection {
            handle: new_connection(&self.store.uri)?,
            store: self.store.clone(),
        })
    }
}

/// Store containing a SQLite connection
#[derive(Clone)]
pub struct Store {
    conn: Arc<RwLock<Conn>>,
    uri: String,
}

impl Drop for Store {
    fn drop(&mut self) {
        println!("{:?} is being deallocated", self);
    }
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Store at {:?}", self.uri)
    }
}

impl Store {
    pub fn new_store<T>(uri: T) -> Result<StoreConnection, store_errors::Error>
        where T: Into<Option<String>> {
        let uri_string = uri.into().unwrap_or(String::new());
        let mut connection = new_connection(&uri_string)?;
        let store = Store::new(uri_string, &mut connection)?;
        Ok(StoreConnection {
            handle: connection,
            store: store,
        })
    }

    fn new(uri: String,  connection: &mut Connection) -> Result<Self, store_errors::Error> {
        let c = Conn::connect(connection)?;
        Ok(Store {
            conn:Arc::new(RwLock::new(c)),
            uri: uri,
        })
    }
}
