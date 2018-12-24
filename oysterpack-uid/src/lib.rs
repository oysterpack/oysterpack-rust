/*
 * Copyright 2018 OysterPack Inc.
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

//! Provides support for universally unique identifiers that confirm to the [ULID spec](https://github.com/ulid/spec).
//!
//! ## Features
//! - ULID generation via [ULID](ulid/struct.ULID.html)
//! - ULIDs can be associated with a domain manner. Example domains are user ids, request ids, application ids, service ids, etc.
//!   - [TypedULID&lt;T&gt;](ulid/struct.TypedULID.html)
//!     - domain is encoded into the type system
//!   - [DomainULID](ulid/struct.DomainULID.html) and [Domain](ulid/struct.Domain.html)
//!     - domain is defined by code, i.e., [Domain](ulid/struct.Domain.html) is used to define domain names as constants
//!     - [DomainULID](ulid/struct.DomainULID.html) scopes [ULID](ulid/struct.ULID.html)(s) to a [Domain](ulid/struct.Domain.html)
//! - ULIDs are thread safe, i.e., they can be sent across threads
//! - ULIDs are lightweight and require no heap allocation//!
//! - ULIDs are serializable via [serde](https://crates.io/crates/serde)
//!
//! ### Generating ULIDs
//! ```rust
//! # use oysterpack_uid::*;
//! let id = ULID::generate();
//! ```
//! ### Generating ULID constants
//! ```rust
//! # #[macro_use]
//! # extern crate oysterpack_uid;
//! # use oysterpack_uid::*;
//! op_ulid!{
//!     /// Foo Id
//!     FooId
//! }
//!
//! const FOO_ID: FooId = FooId(1866910953065622895350834727020862173);
//! # fn main() {}
//! ```
//!
//! ### Generating TypedULID&lt;T&gt; where T is a struct
//! ```rust
//! # use oysterpack_uid::TypedULID;
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
//! let id = DomainULID::generate(DOMAIN);
//! ```
//!
//! ### Generating DomainULID constants via DomainId
//! ```rust
//! # use oysterpack_uid::*;
//! pub const FOO_EVENT_ID: DomainId = DomainId(Domain("Foo"), 1866921270748045466739527680884502485);
//! let domain_ulid = FOO_EVENT_ID.as_domain_ulid();
//! ```

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.2.3")]

#[allow(unused_imports)]
#[macro_use]
pub extern crate oysterpack_macros;
/// re-exporting because it is required by op_ulid!
pub extern crate rusty_ulid;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;

#[macro_use]
mod macros;
pub mod ulid;

pub use crate::ulid::{
    ulid_str, ulid_str_into_u128, ulid_u128, ulid_u128_into_string, DecodingError, ULID,
};

pub use crate::ulid::domain::{Domain, DomainId, DomainULID, HasDomain};
pub use crate::ulid::typed::TypedULID;

// re-exported because it is used internally by op_ulid!
// this makes it easier for clients to use op_ulid! without using oysterpack_macros directly, i.e.,
// it is used behind the scenes by this crate.
pub use oysterpack_macros::op_tt_as_item;

#[cfg(test)]
op_tests_mod!();
