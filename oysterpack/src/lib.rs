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
//! ## Logging
//! The [log](https://crates.io/crates/log) crate is used as the logging framework. This crate is not
//! curated because re-exporting the log macros requires re-exporting the whole crate - which pollutes
//! the OysterPack public API. When the rust [2018 edition](https://rust-lang-nursery.github.io/edition-guide/rust-2018/index.html)
//! becomes available, then we will be able to curate the log crate and export the log macros in a
//! clean manner - see [macro changes](https://rust-lang-nursery.github.io/edition-guide/rust-2018/macros/macro-changes.html).

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack/0.2.1")]

pub extern crate oysterpack_app_metadata_macros;
pub extern crate oysterpack_app_metadata as app_metadata;
pub extern crate oysterpack_uid as uid;

pub extern crate semver;
pub extern crate chrono;

pub extern crate serde;
#[allow(unused_imports)]
#[macro_use]
pub extern crate serde_derive;

/// re-exports the macros
pub use oysterpack_app_metadata_macros::*;
pub use serde_derive::*;
