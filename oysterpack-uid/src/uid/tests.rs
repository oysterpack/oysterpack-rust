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

use super::{ulid, ulid_str_into_u128, ulid_u128, ulid_u128_into_string, Uid};
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
        info!("uid_serde(): id json: {}", id_json);
        let id2 = serde_json::from_str(&id_json).unwrap();
        assert_eq!(id, id2);
    });
}

#[test]
fn ulid_functions() {
    run_test(|| {
        use std::collections::HashSet;
        let count = 100000;

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(ulid()))
        }

        for uid in hashes {
            let uid_u128 = ulid_str_into_u128(&uid).unwrap();
            let uid2 = ulid_u128_into_string(uid_u128);
            assert_eq!(uid, uid2);
        }

        assert!(ulid_str_into_u128("INVALID").is_err());
    });
}
