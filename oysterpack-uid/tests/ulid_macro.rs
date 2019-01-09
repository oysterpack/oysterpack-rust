/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use oysterpack_uid::{macros::*, Domain, HasDomain, ULID};

use serde::{Deserialize, Serialize};

#[ulid]
pub struct UserId(pub u128);

#[test]
fn from_ulid() {
    let ulid = oysterpack_uid::ULID::generate();
    let user_id = UserId::from(ulid);
    let ulid_u128: u128 = ulid.into();
    assert_eq!(user_id.0, ulid_u128);

    println!("{:?} {}", user_id, user_id);
}

#[test]
fn const_ulid() {
    #[ulid]
    struct FooId(u128);

    const FOO_ID: FooId = FooId(1866907549525959787301297812082244355);
    let id = FOO_ID.to_string();
    let ulid: ULID = id.parse().unwrap();
    assert_eq!(ulid, FOO_ID.ulid());
    let foo_ulid: ULID = FOO_ID.into();
    assert_eq!(ulid, foo_ulid);
    println!("{}", serde_json::to_string(&FOO_ID).unwrap());
    assert_ne!(FooId::generate(), FooId::generate());
}

#[test]
fn ulid_newtype() {
    #[ulid]
    struct FooId(oysterpack_uid::ULID);

    let foo_id: FooId = FooId::generate();
    let id = foo_id.to_string();
    let ulid: ULID = id.parse().unwrap();
    assert_eq!(ulid, foo_id.ulid());
    let foo_ulid: ULID = foo_id.into();
    assert_eq!(ulid, foo_ulid);
    println!("{}", serde_json::to_string(&foo_id).unwrap());
    assert_ne!(FooId::generate(), FooId::generate());
}

#[test]
fn domain_attribute() {
    #[domain(Foo)]
    #[ulid]
    struct FooId(oysterpack_uid::ULID);

    let foo_id: FooId = FooId::generate();
    let id = foo_id.to_string();
    let ulid: ULID = id.parse().unwrap();
    assert_eq!(ulid, foo_id.ulid());
    let foo_ulid: ULID = foo_id.into();
    assert_eq!(ulid, foo_ulid);
    println!("{}", serde_json::to_string(&foo_id).unwrap());
    assert_ne!(FooId::generate(), FooId::generate());

    println!("FooId::DOMAIN = {}", FooId::DOMAIN);
    assert_eq!(FooId::DOMAIN, Domain("Foo"));
}
