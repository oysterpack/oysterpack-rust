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

//! Provides support for universally unique identifiers that confirm to the [ULID spec](https://github.com/ulid/spec).
//!
//! You can generate ULIDs as String or u128.
//! You can convert ULIDs between String and u128.
//!
//! ```
//! use oysterpack_uid::{
//!     ulid,
//!     ulid_u128,
//!     ulid_u128_into_string,
//!     ulid_str_into_u128
//! };
//!
//! // generates a new ULID as a string
//! let id_str = ulid();
//! // generates a new ULID as u128
//! let id_u128 = ulid_u128();
//!
//! // conversions between string and u128 ULIDs
//! let ulid_str = ulid_u128_into_string(id_u128);
//! assert_eq!(ulid_str_into_u128(&ulid_str).unwrap(), id_u128);
//! ```
//!
//! You can define type safe ULID based unique identifiers ([Uid](uid/struct.Uid.html)):
//!
//! ### Uid for structs
//! ```rust
//! use oysterpack_uid::Uid;
//! struct User;
//! type UserId = Uid<User>;
//! let id = UserId::new();
//! ```
//!
//! ### Uid for traits
//! ```rust
//! use oysterpack_uid::Uid;
//! trait Foo{}
//! // Send + Sync are added to the type def in order to satisfy Uid type constraints for thread safety,
//! // i.e., in order to be able to send the Uid across threads.
//! type FooId = Uid<dyn Foo + Send + Sync>;
//! let id = FooId::new();
//! ```
//! By default, Uid<T> is serializable via serde. If serialization is not needed then you can opt out by
//! including the dependency with default features disabled : `default-features = false`.
//!
//! ### ULID vs [UUID](https://crates.io/crates/uuid) Performance
//! - below are the times to generate 1 million ULIDs are on my machine (Intel(R) Core(TM) i7-3770K CPU @ 3.50GHz):
//!
//! |Description|Test|Duration (ms)|
//! |-----------|----|-------------|
//! |new ULID encoded as u128|[ulid_128()](uid/fn.ulid_u128.html)|954|
//! |new ULID as Uid|[Uid::new()](struct.Uid.html#method.new)|953|
//! |new ULID encoded as String|[ulid()](uid/fn.ulid.html)|2172|
//! |new V4 UUID|`uuid::Uuid::new_v4()`|1007|
//! |new V4 UUID encoded as String|`uuid::Uuid::new_v4().to_string()`|6233|
//!
//! #### Performance Test Summary
//! - in terms of raw performance, ULID is slightly faster than UUID, but on par
//! - ULID is the clear winner in terms of encoding the identifier as a String
//!   - encoding ULIDs as a string is roughly 2.28x slower
//!   - encoding UUIDs as a string is roughly 6.19x slower

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.1.2")]

extern crate chrono;
extern crate rusty_ulid;
#[cfg(any(test, feature = "serde"))]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate uuid;

pub mod uid;

pub use uid::Uid;
pub use uid::{ulid, ulid_str_into_u128, ulid_u128, ulid_u128_into_string};

#[cfg(test)]
op_tests_mod!();
