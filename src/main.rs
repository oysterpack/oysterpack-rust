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

extern crate oysterpack_errors;

use oysterpack_errors::{Panic, PanicId};

const PANIC_1: Panic = Panic(PanicId(1));

fn main() {
    let mut _mutable_integer = 7i32;

    {
        // Borrow `_mutable_integer`
        let _large_integer = &mut _mutable_integer;

        // Error! `_mutable_integer` is frozen in this scope
        *_large_integer = 50;
        // FIXME ^ Comment out this line

        // `_large_integer` goes out of scope
    }

    // Ok! `_mutable_integer` is not frozen in this scope
    _mutable_integer = 3;
}

