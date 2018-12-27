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
//! - ULIDs can be associated with a domain. Example domains are user ids, request ids, application ids, service ids, etc.
//!   - [DomainULID](ulid/struct.DomainULID.html) and [Domain](ulid/struct.Domain.html)
//!     - domain is defined by code, i.e., [Domain](ulid/struct.Domain.html) is used to define domain names as constants
//!     - [DomainULID](ulid/struct.DomainULID.html) scopes [ULID](ulid/struct.ULID.html)(s) to a [Domain](ulid/struct.Domain.html)
//!   - [DomainId](ulid/struct.DomainId.html) can be used to define constants, which can then be converted into DomainULID
//!   - u128 or ULID tuple structs marked with a `#[ulid]` attribute
//! - ULIDs are thread safe, i.e., they can be sent across threads
//! - ULIDs are lightweight and require no heap allocation
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
//! # #[macro_use]
//! # extern crate serde;
//! # use oysterpack_uid::*;
//! #[oysterpack_uid::macros::ulid]
//! pub struct FooId(u128);
//!
//! const FOO_ID: FooId = FooId(1866910953065622895350834727020862173);
//! # fn main() {}
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

/// re-exporting because it is required by op_ulid!
pub extern crate rusty_ulid;

/// macros
pub mod macros {
    pub use oysterpack_uid_macros::{
        ulid, domain
    };
}
pub mod ulid;

pub use crate::ulid::{
    ulid_str, ulid_str_into_u128, ulid_u128, ulid_u128_into_string, DecodingError, ULID,
};

pub use crate::ulid::domain::{Domain, DomainId, DomainULID, HasDomain};
