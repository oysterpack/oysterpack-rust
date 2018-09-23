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

//! provides build info gathered from cargo - see https://crates.io/crates/built

use built;
use std::{env, io, path};

include!(concat!(env!("OUT_DIR"), "/built.rs"));

/// Gathers build information and generates code to make it available at runtime.
///
/// If `dependencies` = true, then Cargo.lock is parsed to get this crate's dependencies and their versions.
///
/// For this to work, Cargo.lock needs to actually be there; this is (usually) only true for executables
/// and not for libraries. Cargo will only create a Cargo.lock for the top-level crate in a dependency-tree.
/// In case of a library, the top-level crate will decide which crate/version combination to compile
/// and there will be no Cargo.lock while the library gets compiled as a dependency.
///
/// Parsing Cargo.lock instead of Cargo.toml allows us to serialize the precise versions Cargo chose
/// to compile. One can't, however, distinguish build-dependencies, dev-dependencies and dependencies.
/// Furthermore, some dependencies never show up if Cargo had not been forced to actually use them
/// (e.g. dev-dependencies with cargo test never having been executed).
pub fn write_built_file(dependencies: bool) -> io::Result<()> {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    let mut options = built::Options::default();
    options.set_dependencies(dependencies);
    built::write_built_file_with_opts(&options, &src, &dst)?;
    Ok(())
}

/// Gathers library build information and generates code to make it available at runtime.
///
/// Dependencies are not available for library crates
///
/// # Panics
/// - if build-time information failed to be acquired
pub fn write_library_built_file() {
    write_built_file(false).expect("Failed to acquire build-time information")
}

/// Gathers application binary build information and generates code to make it available at runtime.
///
/// Dependencies are available for binary crates
///
/// # Panics
/// - if build-time information failed to be acquired
pub fn write_app_built_file() {
    write_built_file(true).expect("Failed to acquire build-time information")
}
