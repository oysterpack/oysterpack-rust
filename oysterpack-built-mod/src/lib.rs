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

//! This module provides a macro that will generate a public module that contains build-time info
//! that was generated via [oysterpack_built](https://crates.io/crates/oysterpack_built)

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built_mod/0.1.0")]

/// Generate a public module named `build` which includes build-time info generated via
/// [oysterpack_built](https://crates.io/crates/oysterpack_built)
#[macro_export]
macro_rules! op_build_mod {
    () => {
        /// provides build-time information
        pub mod build {
            // The file has been placed there by the build script.
            include!(concat!(env!("OUT_DIR"), "/built.rs"));
        }
    };
}

#[cfg(test)]
mod tests;
