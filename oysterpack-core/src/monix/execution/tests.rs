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

use rusty_ulid::Ulid;
use std::convert::Into;

#[test]
fn quick_test() {
    let id = Ulid::new();
    println!("{}", id);
    let id: u128 = id.into();
    println!("{}", id);
    let id = Ulid::from(id);
    println!("{}", id);
}
