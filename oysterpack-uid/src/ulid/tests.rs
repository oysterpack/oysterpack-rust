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

use super::*;
use serde_json;
use std::{cmp::Ordering, str::FromStr};
use tests::run_test;

#[derive(Debug)]
struct User;

impl HasDomain for User {
    const DOMAIN: Domain = Domain("User");
}

type UserId = TypedULID<User>;

trait Foo {}

type FooId = TypedULID<dyn Foo + Send + Sync>;

op_test!(uid_hash_uniqueness {
    test_uid_hash_uniqueness();
});

// New Ids should be unique

fn test_uid_hash_uniqueness() {
    use std::collections::HashSet;
    let count = 100000;
    info!("id_hash_uniqueness: {}", count);

    let mut hashes = HashSet::new();
    for _ in 0..count {
        assert!(hashes.insert(UserId::generate()))
    }
}

#[test]
fn uid_str() {
    run_test("uid_str", || {
        let id = FooId::generate();
        let id_str = id.to_string();
        info!("uid_str: {}", id_str);
        let id2 = FooId::from_str(&id_str).unwrap();
        assert_eq!(id, id2);
    });
}

#[test]
fn uid_eq() {
    let id = FooId::generate();
    let id_u128: u128 = id.into();
    assert_eq!(id, TypedULID::from(id_u128));
}

#[test]
fn uid_ordered() {
    use std::thread;
    let mut id = FooId::generate();
    for _ in 0..10 {
        thread::sleep_ms(1);
        let temp = FooId::generate();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_next() {
    let mut id = FooId::generate();
    for _ in 0..1000 {
        let temp = id.clone().increment().unwrap();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_is_thread_safe() {
    use std::thread;

    let id = FooId::generate();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);

    let id = UserId::generate();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);
}

#[test]
fn uid_serde() {
    pub struct Foo;

    run_test("uid_serde", || {
        let id = TypedULID::<Foo>::generate();
        let id_json = serde_json::to_string(&id).unwrap();
        info!("uid_serde(): id json: {}", id_json);
        let id2 = serde_json::from_str(&id_json).unwrap();
        assert_eq!(id, id2);
    });
}

#[test]
fn ulid_functions() {
    run_test("ulid_functions", || {
        use std::collections::HashSet;
        let count = 100000;

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(ulid_str()))
        }

        for uid in hashes {
            let uid_u128 = ulid_str_into_u128(&uid).unwrap();
            let uid2 = ulid_u128_into_string(uid_u128);
            assert_eq!(uid, uid2);
        }

        assert!(ulid_str_into_u128("INVALID").is_err());
    });
}

//[12:13:25.222][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:158] benchmark_new_ulid(): TypedULID::new() : 1.012130242s
//[12:13:26.232][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:167] benchmark_new_ulid(): ulid_u128() : 1.009717453s
//[12:13:27.293][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:174] benchmark_new_ulid(): id.increment().unwrap() : 1.06046932s
//[12:13:29.548][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:183] benchmark_new_ulid(): ulid_str() : 2.254963995s
//[12:13:30.510][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:190] benchmark_new_ulid(): uuid::Uuid::new_v4() : 962.218743ms
//[12:13:36.603][INFO][oysterpack_uid::ulid::tests][oysterpack_uid::ulid::tests:200] benchmark_new_ulid(): uuid::Uuid::new_v4().to_string() : 6.093115497s
#[test]
#[ignore]
fn benchmark_new_ulid() {
    use std::time::Instant;

    run_test("benchmark_new_ulid", || {
        struct Foo;
        type FooId = TypedULID<Foo>;;
        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = FooId::generate();
        }
        info!(
            "benchmark_new_ulid(): TypedULID::generate() : {:?}",
            now.elapsed()
        );

        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = ULID::generate();
        }
        info!("benchmark_new_ulid(): ULID::generate() : {:?}", now.elapsed());

        let id = FooId::generate();
        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = id.increment().unwrap();
        }
        info!(
            "benchmark_new_ulid(): TypedULID.increment().unwrap() : {:?}",
            now.elapsed()
        );

        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = ULID::generate().to_string();
        }
        info!("benchmark_new_ulid(): ULID::generate().to_string() : {:?}", now.elapsed());

        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = FooId::generate().to_string();
        }
        info!("benchmark_new_ulid(): TypedULID::generate().to_string() : {:?}", now.elapsed());

        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = ulid_str();
        }
        info!("benchmark_new_ulid(): ulid_str() : {:?}", now.elapsed());

        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = ulid_u128();
        }
        info!("benchmark_new_ulid(): ulid_u128() : {:?}", now.elapsed());

        use uuid;
        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = uuid::Uuid::new_v4();
        }
        info!(
            "benchmark_new_ulid(): uuid::Uuid::new_v4() : {:?}",
            now.elapsed()
        );

        info!("UUID: {}", uuid::Uuid::new_v4());
        let now = Instant::now();
        for _ in 0..1000000 {
            let _ = uuid::Uuid::new_v4().to_string();
        }
        info!(
            "benchmark_new_ulid(): uuid::Uuid::new_v4().to_string() : {:?}",
            now.elapsed()
        );
    })
}

#[test]
fn domain_uid() {
    run_test("domain_uid", || {
        let id: DomainULID = UserId::generate().into();
        assert_eq!(id.domain(), User::DOMAIN.name());
        info!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
        info!("{:?} => {}", id, id);
        const DOMAIN_FOO: Domain = Domain("Foo");
        let id = FooId::generate();
        let id: DomainULID = DomainULID::from_ulid(&DOMAIN_FOO, id.ulid());
        info!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
        let id = DomainULID::from_ulid(&User::DOMAIN, ULID::generate());
    })
}

#[test]
fn ulid_default() {
    run_test("ULID", || {
        let id_1: ULID = ULID::generate();
        let id_2: ULID = ULID::generate();
        assert!(id_1 != id_2);

        let id_json = serde_json::to_string_pretty(&id_2).unwrap();
        info!("ULID as json: {}", id_json);
        let id_3: ULID = serde_json::from_str(&id_json).unwrap();
        assert_eq!(id_3, id_2);

        let foo_id = FooId::generate();
        let foo_ulid: ULID = foo_id.into();
        assert_eq!(foo_id.ulid(), foo_ulid);
    });
}
