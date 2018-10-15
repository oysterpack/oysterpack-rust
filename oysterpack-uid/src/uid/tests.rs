// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! unit tests
#![allow(warnings)]

use super::Uid;
use serde_json;
use std::{cmp::Ordering, str::FromStr};
use tests::run_test;

struct O;

type Oid = Uid<O>;

trait Foo {}

type FooId = Uid<dyn Foo + Send + Sync>;

// New Ids should be unique
#[test]
fn uid_hash_uniqueness() {
    run_test(|| {
        use std::collections::HashSet;
        let count = 100000;
        info!("id_hash_uniqueness: {}", count);

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(Oid::new()))
        }
    });
}

#[test]
fn uid_str() {
    run_test(|| {
        let id = FooId::new();
        let id_str = id.to_string();
        info!("uid_str: {}", id_str);
        let id2 = FooId::from_str(&id_str).unwrap();
        assert_eq!(id, id2);
    });
}

#[test]
fn uid_eq() {
    let id = FooId::new();
    let id_u128: u128 = id.into();
    assert_eq!(id, Uid::from(id_u128));
}

#[test]
fn uid_ordered() {
    use std::thread;
    let mut id = FooId::new();
    for _ in 0..10 {
        thread::sleep_ms(1);
        let temp = FooId::new();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_next() {
    let mut id = FooId::new();
    for _ in 0..1000 {
        let temp = id.clone().increment().unwrap();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_is_thread_safe() {
    use std::thread;

    let id = FooId::new();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);

    let id = Oid::new();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);
}

#[test]
fn uid_serde() {
    pub struct Foo;

    run_test(|| {
        let id = Uid::<Foo>::new();
        let id_json = serde_json::to_string(&id).unwrap();
        info!("id json: {}", id_json);
        let id2 = serde_json::from_str(&id_json).unwrap();
        assert_eq!(id, id2);
    });
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id(pub u128);

impl Id {
    /// returns the id
    pub fn id(&self) -> u128 {
        self.0
    }
}

impl ::std::fmt::Display for Id {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use std::fmt;

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u128(self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("i128")
            }

            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > 0 {
                    Ok(Id(value as u128))
                } else {
                    Err(E::custom(format!("u128 must be >= 0: {}", value)))
                }
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > 0 {
                    Ok(Id(value as u128))
                } else {
                    Err(E::custom(format!("u128 must be >= 0: {}", value)))
                }
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > 0 {
                    Ok(Id(value as u128))
                } else {
                    Err(E::custom(format!("u128 must be >= 0: {}", value)))
                }
            }

            #[inline]
            fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > 0 {
                    Ok(Id(value as u128))
                } else {
                    Err(E::custom(format!("u128 must be >= 0: {}", value)))
                }
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id(u128::from(value)))
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id(u128::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id(u128::from(value)))
            }

            #[inline]
            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id(value))
            }
        }

        deserializer.deserialize_i128(IdVisitor)
    }
}
