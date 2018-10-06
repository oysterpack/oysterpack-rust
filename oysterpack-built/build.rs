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

use std::{env, io, path, fs::OpenOptions, io::{prelude::*}};

fn main() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&built::Options::default(), &src, &dst).expect("Failed to acquire build-time information");
    let mut built_file = OpenOptions::new()
        .append(true)
        .open(&dst).expect("Failed to open file in append mode");


    built_file.write_all(b"/// An array of effective dependencies as a JSON array.\n").unwrap();
    writeln!(
        built_file,
        "pub const DEPENDENCIES_JSON: &str = \"[]\";",
    ).unwrap();
}
