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

//! This crate provides the [op_build_mod](macro.op_build_mod.html) macro to load the application build
//! netadata defined by [oysterpack_app_metadata](https://crates.io/crates/oysterpack_app_metadata).
//!
//! It works in conjuction with [oysterpack_built](https://crates.io/crates/oysterpack_built), which
//! is used to generate the application build netadata.

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_app_metadata_macros/0.1.0")]

pub extern crate oysterpack_app_metadata;
pub extern crate semver;
pub extern crate chrono;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;
#[cfg(test)]
extern crate serde_json;

#[macro_use]
mod macros;

#[cfg(test)]
op_tests_mod!();
