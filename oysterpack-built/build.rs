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

//! Gathers build time information for the crate - see https://crates.io/crates/built

extern crate built;

use std::{env, io, path};

fn main() {
    write_built_file(false).expect("Failed to acquire build-time information");
}

/// Parsing Cargo.lock and writing lists of dependencies and their versions.
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
fn write_built_file(dependencies: bool) -> io::Result<()> {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    let mut options = built::Options::default();
    options.set_dependencies(dependencies);
    built::write_built_file_with_opts(&options, &src, &dst)?;
    Ok(())
}
