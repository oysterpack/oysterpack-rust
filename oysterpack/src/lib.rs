// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate defines the public framework API for the OysterPack platform.
//! This crate curates the OysterPack modules in a central location.
//!
//! ## Getting started
//!
//! ```rust
//! #[macro_use]
//! extern crate oysterpack;
//!
//! use oysterpack::log;
//!
//! // gathers build time info for this crate
//! op_build_mod!();
//!
//! fn main() {
//!     // get the app's build info that was gathered during compilation
//!     let app_build = build::get();
//!
//!     // initialize the log system
//!     log::init(log_config(),&app_build);
//!
//!     // The LogConfig used to initialize the log system can be retrieved.
//!     // This enables the LogConfig to be inspected.
//!     let log_config = log::config().unwrap();
//!
//!     run();
//!
//!     // shutdown the logging system
//!     log::shutdown();
//! }
//!
//! /// This should be loaded from the app's configuration.
//! /// For this simple example, we are simply using the default LogConfig.
//! /// The default LogConfig sets the root log level to Warn and logs to stdout.
//! fn log_config() -> log::LogConfig {
//!     Default::default()
//! }
//!
//! fn run() {
//!     // the log macros were re-exported by oysterpack
//!     info!("running ...");
//! }
//! ```

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack/0.2.2")]

pub extern crate oysterpack_app_metadata as app_metadata;
pub extern crate oysterpack_app_metadata_macros;
pub extern crate oysterpack_log as log;
pub extern crate oysterpack_macros;
pub extern crate oysterpack_uid as uid;

pub extern crate chrono;
pub extern crate semver;

pub extern crate serde;
#[allow(unused_imports)]
#[macro_use]
pub extern crate serde_derive;

/// re-export log macros
pub use log::log::{debug, error, info, log, log_enabled, trace, warn};
/// re-exports macros
pub use oysterpack_app_metadata_macros::op_build_mod;
pub use oysterpack_macros::{
    op_newtype, op_tt_as_expr, op_tt_as_item, op_tt_as_pat, op_tt_as_stmt,
};
pub use serde_derive::*;
