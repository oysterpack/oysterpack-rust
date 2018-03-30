// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # OysterPack Rust Reactive Platform
//!
//! The mission is to provide a platform to build [reactive](https://www.reactivemanifesto.org/) systems in Rust.
//!
//! ## Why Rust ?
//! Because [Rust](https://www.rust-lang.org) is the best systems programming language for building production grade reactive systems today.
//! It's main competitive advantages are:
//!
//! 1. Bare metal performance
//! 2. Memory safety features guarantees with out the need for a garbage collector
//!    - Rust enforces [RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization), which shields against memory resource leak bugs
//!    - garbage collectors add resource overhead and add to bloat
//! 3. Small memory footprint
//!    - no language runtime required
//!    - no garbage collector
//! 4. Rust compiler
//!    - prevents segfaults
//!    - prevents data races
//!    - prevents memory leaks
//!    - smartest compiler in the market that detects runtime issues at compile time
//!    - compiler is designed to help you figure out what went wrong and provides suggestions
//! 5. Cargo tooling and Rust ecosystem
//!    - simple to learn and use
//!    - promotes collaboration and productivity
//!    - [crates.io](https://crates.io/)
//!    - makes it easy to build and publish crates

#[cfg(test)]
mod tests {
    #[test]
    fn quick_test() {
    }
}
