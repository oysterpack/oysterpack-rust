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

//! Errors must be treated as a core architectural concern. This standardizes Errors on the OysterPack
//! platform. Errors will have the following properties :
//!
//! - Errors are assigned a ULID - think of it as the Error type id
//! - Error instances are assigned a ULID
//!   - this enables specific errors to be looked up
//!   - the error instance creat timestamp is embedded within the ULID
//! - Errors are assigned a severity level
//! - The source code location where the Error was created is captured
//! - Errors can be converted into oysterpack_events::Event
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
//!   use oysterpack_errors::{Level, Id};
//!
//!   pub const FOO_ERR: (Id, Level) = (Id(1863702216415833425137248269790651577), Level::Error);
//!   pub const BAR_ERR: (Id, Level) = (Id(1863710724424640971018467695308814281), Level::Alert);
//! }
//!
//! fn main() {
//!     let err = op_error!(errs::FOO_ERR, "BOOM!!".to_string());
//!     let event: Event<Error> = op_error_event!(err);
//!     event.log();
//! }
//!
//! ```

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_errors/0.1.0")]

#[macro_use]
extern crate oysterpack_log;
#[macro_use]
pub extern crate oysterpack_events;
#[macro_use]
extern crate oysterpack_macros;
extern crate oysterpack_uid;

#[macro_use]
extern crate serde;
extern crate chrono;
#[macro_use]
extern crate failure;

#[macro_use]
#[cfg(test)]
extern crate oysterpack_testing;

#[macro_use]
pub mod error;

pub use error::{Error, Id, Level};

#[cfg(test)]
op_tests_mod!();
