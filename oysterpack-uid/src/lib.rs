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

//! Provides generic type safe Universally Unique Lexicographically Sortable Identifiers ([ulid](https://github.com/ulid/spec)):
//! - ulids are associated with a type
//! - are serializable, i.e., [Serde](https://docs.rs/serde) compatible
//! - are threadsafe

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_uid/0.1.0")]

extern crate rusty_ulid;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;

pub mod uid;

pub use uid::Uid;

#[cfg(test)]
mod tests;
