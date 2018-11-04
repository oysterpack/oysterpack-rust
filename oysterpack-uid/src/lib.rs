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
//! ## Features
//! - ULID generation via [ULID](ulid/struct.ULID.html)
//! - ULIDs can be associated with a domain. Example domains are user ids, request ids, application ids, service ids, etc.
//!   - [TypedULID&lt;T&gt;](ulid/struct.TypedULID.html)
//!     - the domain is defined by the type system
//!     - business logic should be working with this strongly typed domain ULID
//!   - [DomainULID&lt;T&gt;](ulid/struct.DomainULID.html)
//!     - the domain is defined explicitly on the struct
//!     - meant to be used when multiple types of ULIDs need to be handled in a generic fashion, e.g.,
//!      event tagging, storing to a database, etc
//! - ULIDs are serializable via [serde](https://crates.io/crates/serde)
//!
//! ### Generating ULIDs
//! ```rust
//! # use oysterpack_uid::*;
//! let id = ULID::generate();
//! ```
//!
//! ### Generating TypedULID&lt;T&gt; where T is a struct
//! ```rust
//! use oysterpack_uid::TypedULID;
//! struct User;
//! type UserId = TypedULID<User>;
//! let id = UserId::generate();
//! ```
//!
//! ### TypedULID&lt;T&gt; where T is a trait
//! ```rust
//! use oysterpack_uid::TypedULID;
//! trait Foo{}
//! // Send + Sync are added to the type def in order to satisfy TypedULID type constraints for thread safety,
//! // i.e., in order to be able to send the TypedULID across threads.
//! type FooId = TypedULID<dyn Foo + Send + Sync>;
//! let id = FooId::generate();
//! ```
//!
//! ### Generating DomainULIDs
//! ```rust
//! # use oysterpack_uid::*;
//! const DOMAIN: Domain = Domain("Foo");
//! let id = DomainULID::generate(&DOMAIN);
//! ```
//!
//! ### ULID vs [UUID](https://crates.io/crates/uuid) Performance
//! - below are the times to generate 1 million ULIDs are on my machine (Intel(R) Core(TM) i7-3770K CPU @ 3.50GHz):
//!
//! |Description|Test|Duration (ms)|
//! |-----------|----|-------------|
//! |TypedULID generation|[TypedULID::generate()](struct.TypedULID.html#method.new)|995|
//! |ULID generation|[ulid_str()](ulid/fn.ulid_str.html)|980|
//! |V4 UUID generation|`uuid::Uuid::new_v4()`|966|
//! |TypedULID encoded as String|`TypedULID::generate().to_string()`|4113|
//! |ULID encoded as String|`ULID::generate().to_string()`|3271|
//! |V4 UUID encoded as String|`uuid::Uuid::new_v4().to_string()`|6051|
//!
//! #### Performance Test Summary
//! - in terms of raw id generation performance, it's a draw between ULID and UUID
//! - ULID is the clear winner in terms of encoding the identifier as a String

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.2.0")]

extern crate chrono;
extern crate rusty_ulid;
#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate uuid;

pub mod ulid;

pub use ulid::{Domain, DomainULID, HasDomain, TypedULID, ULID};

#[cfg(test)]
op_tests_mod!();
