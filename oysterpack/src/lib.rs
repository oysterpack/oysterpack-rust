// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate defines the public framework API for the OysterPack platform.
//! This crate curates the OysterPack modules into a single package.
//!
//! # Features
//!

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack/0.1.1")]

pub extern crate oysterpack_app_metadata;
extern crate oysterpack_uid;

pub use oysterpack_uid::uid as ulid;
pub use oysterpack_app_metadata::{
    semver,
    chrono
};


