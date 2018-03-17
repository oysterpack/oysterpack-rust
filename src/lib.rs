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

//! # OysterPack Rust Reactive Platform
//!
//! The mission is to provide a platform to build [reactive](https://www.reactivemanifesto.org/)
//! systems in Rust.
//!
//! ## Why Rust ?
//! Because [Rust](https://www.rust-lang.org) is the best systems programming language for building
//! production grade reactive systems today. It's main competitive advantages are:
//!
//! 1. Bare metal performance
//! 2. Memory safety features guarantees with out the need for a garbage collector
//!    - garbage collectors add resource overhead and add to bloat
//! 3. Small memory footprint
//!    - no language runtime required
//!    - no garbage collector
//! 4. Rust compiler
//!    - prevents segfaults
//!    - prevents data races
//!    - smartest compiler in the market that detects runtime issues at compile time
//!    - compiler is designed to help you figure out what went wrong and provides suggestions
//! 5. Cargo tooling and Rust ecosystem
//!    - simple to learn and use
//!    - promotes collaboration and productivity
//!    - [crates.io](https://crates.io/)
//!    - makes it easy to build and publish crates
//!

pub mod messages;

#[cfg(test)]
mod tests {

    #[test]
    fn quick_test() {
    }
}
