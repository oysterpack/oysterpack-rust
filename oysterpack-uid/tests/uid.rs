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

// ulid integration tests

use oysterpack_uid::macros::ulid;

// is required because Serialize and Deserialize are derived for Foo via `#[ulid]`
use serde::{Deserialize, Serialize};

#[ulid]
pub struct Foo(u128);

#[test]
fn uid_json() {
    let id = Foo::generate();
    let id_json = serde_json::to_string(&id).unwrap();
    let id2 = serde_json::from_str(&id_json).unwrap();
    assert_eq!(id, id2);
}
