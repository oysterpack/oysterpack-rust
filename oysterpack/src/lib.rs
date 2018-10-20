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
//! # Features
//!

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack/0.1.1")]

pub extern crate oysterpack_app_metadata_macros;
pub extern crate oysterpack_app_metadata as app_metadata;
pub extern crate oysterpack_uid as uid;

pub extern crate semver;
pub extern crate chrono;

/// re-exports the macros
pub use oysterpack_app_metadata_macros::*;
