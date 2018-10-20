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

//! Defines the application metadata domain model.
//!
//! It is meant to be used in conjunction with [oysterpack_built](https://crates.io/crates/oysterpack_built).
//! [oysterpack_built](https://crates.io/crates/oysterpack_built) generates the application build
//! metadata source file. This crate is used to load the application build netadata via the
//! [op_build_mod](macro.op_build_mod.html) macro.
//!
//! ## Application Build Metadata Domain Model
//! ![uml](ml.svg)

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_app_metadata_macros/0.1.0")]

pub extern crate chrono;
pub extern crate oysterpack_app_metadata;
pub extern crate semver;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate serde_json;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;
