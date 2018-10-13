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

extern crate mml;
extern crate oysterpack_built;

fn main() {
    oysterpack_built::run();
    generate_uml_graphviz();
}

/// Generates UML class diagrams for the metadata module in Graphviz DOT and SVG format.
/// The generated files will be located within
fn generate_uml_graphviz() {
    use std::{env, path};

    let src = path::Path::new("src").join("metadata");
    let dest = path::Path::new(&env::var("OUT_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("doc")
        .join(env!("CARGO_PKG_NAME"));
    if let Err(err) = mml::src2both(src.as_path(), dest.as_path()) {
        eprintln!("Failed to generated UML Graphviz diagrams: {}", err);
    }
}
