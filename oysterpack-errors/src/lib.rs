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

//! Errors must be treated as a core architectural concern. This standardizes Errors on the OysterPack
//! platform. Errors have the following properties:
//!
//! - Errors are assigned a ULID - think of it as the Error type id
//! - Error instances are assigned a ULID
//!   - this enables specific errors to be looked up
//!   - the error instance create timestamp is embedded within the ULID
//! - Errors are assigned a severity level
//! - The source code location where the Error was created is captured
//! - Error causes can be linked to the error
//! - Errors can be converted into oysterpack_events::Event
//! - Errors implement [failure::Fail](https://docs.rs/failure/latest/failure/trait.Fail.html)
//! - Errors are threadsafe, i.e., they can be sent across threads
//!
//! ## How to Define Errors
//! ```rust
//! #[macro_use]
//! extern crate oysterpack_errors;
//!
//! use oysterpack_errors::{
//!  Error,
//!  oysterpack_events::{ Event, Eventful }
//! };
//!
//! // centralize your errors in a dedicated error module
//! mod errs {
//!   use oysterpack_errors::{Level, Id, IsError};
//!
//!   pub const FOO_ERR: (Id, Level) = (Id(1863702216415833425137248269790651577), Level::Error);
//!   pub const BAR_ERR: (Id, Level) = (Id(1863710724424640971018467695308814281), Level::Alert);
//!
//!   pub struct InvalidCredentials;
//!
//!   impl InvalidCredentials {
//!     pub const ERR_ID: Id = Id(1865548837704866157621294180822811573);
//!     pub const ERR_LEVEL: Level = Level::Error;
//!   }
//!
//!   impl IsError for InvalidCredentials {
//!     fn error_id(&self) -> Id { Self::ERR_ID }
//!     fn error_level(&self) -> Level { Self::ERR_LEVEL }
//!   }
//!
//!   impl std::fmt::Display for InvalidCredentials {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!        f.write_str("invalid credentials")
//!     }
//!   }
//! }
//!
//! fn main() {
//!     let cause = op_error!(errs::BAR_ERR, "CAUSE".to_string());
//!     let err = op_error!(errs::FOO_ERR, "BOOM!!".to_string());
//!     let err = err.with_cause(cause);
//!     let event: Event<Error> = op_error_event!(err);
//!     event.log();
//!
//!     let err = op_error!(errs::InvalidCredentials);
//! }
//! ```

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_errors/0.1.2")]

#[allow(unused_imports)]
#[macro_use]
extern crate oysterpack_log;
#[allow(unused_imports)]
#[macro_use]
pub extern crate oysterpack_events;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;

#[macro_use]
#[cfg(test)]
extern crate oysterpack_testing;

#[macro_use]
pub mod error;

pub use crate::error::{Error, ErrorMessage, Id, IsError, Level};

#[cfg(test)]
op_tests_mod!();
