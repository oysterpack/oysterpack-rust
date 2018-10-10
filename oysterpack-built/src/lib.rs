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
//! `oysterpack_built` builds upon [built](https://crates.io/crates/built). In addition, `oysterpack_built`
//! gathers the crate's compile dependencies at build time and makes them available at runtime.
//!
//! Its main purpose is to standardize the build-time metadata integration for OysterPack apps.
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
//!    oysterpack_app_metadata = "0.1"
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
//!    - [oysterpack_app_metadata](https://crates.io/crates/oysterpack_app_metadata) is the companion dependency
//!      that provides the `op_build_mod!()` macro
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
//!    It can be picked up and compiled via the [op_build_mod!()](https://docs.rs/oysterpack_built/latest/oysterpack_app_metadata/macro.op_build_mod.html) macro,
//!    The `op_build_mod!()` will create a public module named *build*, which will contain the build-time
//!    information.
//!
//! The generated `build` module will consist of:
//! - constants for each piece of build metadata
//!
//! Constant | Type | Description
//! -------- | ---- | -----------
//! BUILT_TIME_UTC|&str|The built-time in RFC822, UTC
//! CFG_ENDIAN|&str|The endianness, given by cfg!(target_endian).
//! CFG_ENV|&str|The toolchain-environment, given by cfg!(target_env).
//! CFG_FAMILY|&str|The OS-family, given by cfg!(target_family).
//! CFG_OS|&str|The operating system, given by cfg!(target_os).
//! CFG_POINTER_WIDTH|u8|The pointer width, given by cfg!(target_pointer_width).
//! CFG_TARGET_ARCH|&str|The target architecture, given by cfg!(target_arch).
//! CI_PLATFORM|Option<&str>|The Continuous Integration platform detected during compilation.
//! DEBUG|bool|Value of DEBUG for the profile used during compilation.
//! FEATURES|\[&str; N\]|The features that were enabled during compilation.
//! FEATURES_STR|&str|The features as a comma-separated string.
//! GIT_VERSION|Option<&str>|If the crate was compiled from within a git-repository, GIT_VERSION contains HEAD's tag. The short commit id is used if HEAD is not tagged.
//! HOST|&str|The host triple of the rust compiler.
//! NUM_JOBS|u32|The parallelism that was specified during compilation.
//! OPT_LEVEL|&str|Value of OPT_LEVEL for the profile used during compilation.
//! PKG_AUTHORS|&str|A colon-separated list of authors.
//! PKG_DESCRIPTION|&str|The description.
//! PKG_HOMEPAGE|&str|The homepage.
//! PKG_NAME|&str|The name of the package.
//! PKG_VERSION|&str|The full version.
//! PKG_VERSION_MAJOR|&str|The major version.
//! PKG_VERSION_MINOR|&str|The minor version.
//! PKG_VERSION_PATCH|&str|The patch version.
//! PKG_VERSION_PRE|&str|The pre-release version.
//! PROFILE|&str|release for release builds, debug for other builds.
//! RUSTC|&str|The compiler that cargo resolved to use.
//! RUSTC_VERSION|&str|The output of rustc -V
//! RUSTDOC|&str|The documentation generator that cargo resolved to use.
//! RUSTDOC_VERSION|&str|The output of rustdoc -V
//! DEPENDENCIES_GRAPHVIZ_DOT|&str|graphviz .dot format for the effective dependency graph
//!
//! - `fn get() -> Build`
//!     - [Build](struct.Build.html) provides a consolidated view of the build-time metadata.
//!       This makes it easier to work with the build-time metadata in a typesafe manner.
//!     - [Build](struct.Build.html) supports serialization via [serde](https://crates.io/crates/serde)
//!
//! **NOTE:** The `op_build_mod!()` depends on the following dependencies in order to compile:
//! - [semver](https://crates.io/crates/semver)
//! - [chrono](https://crates.io/crates/chrono)
//! - [serde](https://crates.io/crates/serde)

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built/0.3.0")]

#[macro_use]
extern crate log;
extern crate chrono;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;

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
#[cfg(test)]
extern crate serde_json;

#[cfg(feature = "build-time")]
pub mod build_time;

#[cfg(feature = "build-time")]
pub use build_time::run;

#[cfg(test)]
pub(crate) mod tests;
