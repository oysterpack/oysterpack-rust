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
//! into the final crate. To see what build-time information is gathered, see this crate's
//! [build](https://docs.rs/oysterpack_built/0.2.2/oysterpack_built/build/) module.
//!
//! ## What is the Motivation?
//! From a DevOps perspective, it is critical to know exactly what is deployed.
//!
//! All OysterPack application binaries must provide build time info. This module standardizes the
//! approach, leveraging [built](https://crates.io/crates/built).
//!
//! ## How to integrate within your project
//!
//! 1. Add the following to **Cargo.toml**:
//!    ```toml
//!    [package]
//!    build = "build.rs"
//!
//!    [build-dependencies]
//!    oysterpack_built = "0.2"
//!    ```
//!    - `oysterpack_built` is added as a build dependency
//!    - `build.rs` is the name of the cargo build script to use
//!       - NOTE: By default Cargo looks up for "build.rs" file in a package root (even if you do
//!         not specify a value for build - see [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)).
//! 2. Include the following in **build.rs**:
//!
//!    ```no_run
//!    extern crate oysterpack_built;
//!
//!    fn main() {
//!       oysterpack_built::write_built_file();
//!    }
//!    ```
//! 3. The build script will by default write a file named **built.rs** into Cargo's output directory.
//!    It can be picked up and compiled like this:
//!    ```no_run
//!    // Use of a mod or pub mod is not actually necessary.
//!    pub mod build {
//!       // The file has been placed there by the build script.
//!       include!(concat!(env!("OUT_DIR"), "/built.rs"));
//!    }
//!    ```
//!    - `OUT_DIR` [environment variable is set by Cargo for build scripts](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
//!    - [oysterpack_built_mod](https://crates.io/crates/oysterpack_built_mod) provides a macro to
//!      eliminate the boilerplate
//!

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built/0.2.2")]

extern crate built;

use std::{env, io, path};

/// Gathers build information and generates code to make it available at runtime.
///
/// # Panics
/// If build-time information failed to be gathered.
pub fn write_built_file()  {
    built::write_built_file().expect("Failed to acquire build-time information");
}

#[cfg(test)]
mod tests;
