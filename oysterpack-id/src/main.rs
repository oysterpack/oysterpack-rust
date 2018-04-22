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

//! Prints a new unique random number in lower case hex format

#![deny(missing_docs, missing_debug_implementations, warnings)]

extern crate oysterpack_id;

use oysterpack_id::Id;

struct Unique;
type Uid = Id<Unique>;

fn main() {
    println!("{}", Uid::new())
}
