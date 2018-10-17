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

//! Provides support for universally unique identifiers that confirm to the [ULID spec]((https://github.com/ulid/spec)).
//!
//! This crate provides to ways to work with ULIDs:
//!
//! ## via the `ulid()` functions
//!
//! ```
//! use oysterpack_uid::{
//!     ulid,
//!     ulid_u128,
//!     into_ulid_string,
//!     into_ulid_u128
//! };
//!
//! // generates a new ULID as a string
//! let id_str = ulid();
//! // generates a new ULID as u128
//! let id_u128 = ulid_u128();
//!
//! // conversions between string and u128 ULIDs
//! assert_eq!(into_ulid_u128(&into_ulid_string(id_u128)).unwrap(), id_u128);
//! ```
//! ## via Uid<T> :
//!
//! ### Defining a Uid for a struct
//! ```rust
//! use oysterpack_uid::Uid;
//! struct Domain;
//! type DomainId = Uid<Domain>;
//! let id = DomainId::new();
//! ```
//! ### Defining a Uid for a trait
//! ```rust
//! use oysterpack_uid::Uid;
//! trait Foo{}
//! // traits are not Send. Send is added to the type def in order to satisfy Uid type constraints
//! // in order to be able to send the Uid across threads
//! type FooId = Uid<dyn Foo + Send + Sync>;
//! let id = FooId::new();
//! ```

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.1.0")]

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
pub use uid::{into_ulid_string, into_ulid_u128, ulid, ulid_u128};

#[cfg(test)]
mod tests;
