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

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.1.1")]

extern crate chrono;
extern crate rusty_ulid;
#[cfg(any(test, feature = "serde"))]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate serde_json;

pub mod uid;

pub use uid::Uid;
pub use uid::{ulid, ulid_str_into_u128, ulid_u128, ulid_u128_into_string};

#[cfg(test)]
mod tests;
