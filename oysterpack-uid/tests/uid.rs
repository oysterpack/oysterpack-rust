// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// uid integration tests

extern crate oysterpack_uid;
extern crate serde_json;

use oysterpack_uid::Uid;

pub struct Foo;

#[test]
fn uid_json() {
    let id = Uid::<Foo>::new();
    let id_json = serde_json::to_string(&id).unwrap();
    let id2 = serde_json::from_str(&id_json).unwrap();
    assert_eq!(id, id2);
}
