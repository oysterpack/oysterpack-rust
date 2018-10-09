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

use std::{env, fs::OpenOptions, io::prelude::*, path};

fn main() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&built::Options::default(), &src, &dst)
        .expect("Failed to acquire build-time information");
    let mut built_file = OpenOptions::new()
        .append(true)
        .open(&dst)
        .expect("Failed to open file in append mode");

    built_file
        .write_all(b"/// An array of effective dependencies as a JSON array.\n")
        .unwrap();

    let dependencies = r#"
    digraph {
    0 [label="oysterpack_app_template=0.1.0"]
    1 [label="log=0.4.5"]
    2 [label="serde=1.0.79"]
    3 [label="oysterpack_app_metadata=0.1.0"]
    4 [label="serde_derive=1.0.79"]
    5 [label="fern=0.5.6"]
    6 [label="semver=0.9.0"]
    7 [label="chrono=0.4.6"]
    8 [label="serde_json=1.0.31"]
    9 [label="ryu=0.2.6"]
    10 [label="itoa=0.4.3"]
    11 [label="num-integer=0.1.39"]
    12 [label="time=0.1.40"]
    13 [label="num-traits=0.2.6"]
    14 [label="libc=0.2.43"]
    15 [label="semver-parser=0.7.0"]
    16 [label="proc-macro2=0.4.19"]
    17 [label="syn=0.15.6"]
    18 [label="quote=0.6.8"]
    19 [label="unicode-xid=0.1.0"]
    20 [label="cfg-if=0.1.5"]
    0 -> 1
    0 -> 2
    0 -> 3
    0 -> 4
    0 -> 5
    0 -> 6
    0 -> 7
    0 -> 8
    8 -> 2
    8 -> 9
    8 -> 10
    7 -> 11
    7 -> 2
    7 -> 12
    7 -> 13
    12 -> 14
    11 -> 13
    6 -> 15
    6 -> 2
    5 -> 1
    4 -> 16
    4 -> 17
    4 -> 18
    18 -> 16
    17 -> 19
    17 -> 18
    17 -> 16
    16 -> 19
    3 -> 2
    3 -> 7
    3 -> 6
    3 -> 4
    1 -> 20
}"#;
    writeln!(built_file, "pub const DEPENDENCIES_GRAPHVIZ_DOT: &str = r#\"{}\"#;",dependencies).unwrap();
}
