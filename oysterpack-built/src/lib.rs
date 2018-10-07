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

//! `oysterpack_built` is used as a build-time dependency to gather information about the cargo build
//! environment. It serializes the build-time information into Rust-code, which can then be compiled
//! into the final crate.
//!
//! ## What is the Motivation?
//! From a DevOps perspective, it is critical to know exactly what is deployed.
//!
//! `oysterpack_built` provides the same functionality as [built](https://crates.io/crates/built).
//! Its main purpose is to standardize the integration for OysterPack apps.
//!
//! ## How to integrate within your project
//!
//! 1. Add the following to **Cargo.toml**:
//!
//!    ```toml
//!    [package]
//!    build = "build.rs"
//!
//!    [dependencies]
//!    oysterpack_built = {path = "../oysterpack-built", version="0.3"}
//!    semver = {version = "0.9", features = ["serde"]}
//!    chrono = { version = "0.4", features = ["serde", "time"] }
//!    serde = "1"
//!    serde_derive = "1"
//!    serde_json = "1"
//!
//!    [build-dependencies]
//!    oysterpack_built = {path = "../oysterpack-built", version="0.3", features = ["build-time"]}
//!    ```
//!    - `oysterpack_built` is added as a build dependency
//!    - `build.rs` is the name of the cargo build script to use
//!       - NOTE: By default Cargo looks up for "build.rs" file in a package root (even if you do
//!         not specify a value for build - see [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)).
//!
//! 2. Include the following in **build.rs**:
//!
//!    ```ignore
//!    extern crate oysterpack_built;
//!
//!    fn main() {
//!       oysterpack_built::run();
//!    }
//!    ```
//!
//! 3. The build script will by default write a file named **built.rs** into Cargo's output directory.
//!    It can be picked up and compiled via the `op_build_mod!()` macro.
//!    The `op_build_mod!()` will create a public module named *build*, which will contain the build-time
//!    information.

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built/0.3.0")]

#[macro_use]
extern crate log;
extern crate semver;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
#[macro_use]
extern crate oysterpack_app_metadata;

#[cfg(feature = "build-time")]
extern crate built;
#[cfg(feature = "build-time")]
extern crate cargo;
#[cfg(feature = "build-time")]
extern crate failure;
#[cfg(feature = "build-time")]
extern crate petgraph;

#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate fern;

#[cfg(feature = "build-time")]
pub mod build_time;

#[cfg(feature = "build-time")]
pub use build_time::run;

#[cfg(test)]
pub(crate) mod tests;
