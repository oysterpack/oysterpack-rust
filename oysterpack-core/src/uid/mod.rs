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

//! Provides support to generate universally unique identifiers.

use rusty_ulid::{new_ulid_string, Ulid};

#[macro_use]
mod macros;

/// Returns a universally unique ddentifier
pub fn uid() -> u128 {
    Ulid::new().into()
}

/// Returns a universally unique lexicographically sortable identifier :
/// - 128-bit compatibility with UUID
/// - 1.21e+24 unique ULIDs per millisecond
/// - Lexicographically sortable!
/// - Canonically encoded as a 26 character string, as opposed to the 36 character UUID
/// - Uses Crockford's base32 for better efficiency and readability (5 bits per character)
/// - Case insensitive
/// - No special characters (URL safe)
pub fn ulid() -> String {
    new_ulid_string()
}
