// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

extern crate time;

extern crate mentat;

use std::fmt::{
    Display,
    Formatter,
    Result
};

use time::Timespec;

use mentat::{
    NamespacedKeyword,
    Entid,
    TypedValue,
    Uuid,
};

pub trait ToTypedValue {
    fn to_typed_value(&self) -> TypedValue;
}

impl ToTypedValue for String {
    fn to_typed_value(&self) -> TypedValue {
        self.clone().into()
    }
}

impl<'a> ToTypedValue for &'a str {
    fn to_typed_value(&self) -> TypedValue {
        self.to_string().into()
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

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
        (*self).into()
    }
}

impl ToTypedValue for Timespec {
    fn to_typed_value(&self) -> TypedValue {
        let micro_seconds = (self.sec * 1_000_000) + i64::from((self.nsec / 1_000));
        TypedValue::instant(micro_seconds)
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

#[cfg(test)]
mod test {
    use super::{
        ToTypedValue,
        TypedValue,
        Timespec,
    };

    #[test]
    fn test_timespec_to_typed_value() {
        let timespec = Timespec {
            sec: 1518434618,
            nsec: 740993537,
        };
        let typed_value: TypedValue = timespec.to_typed_value();
        assert_eq!(typed_value, TypedValue::instant(1518434618740993));
    }
}
