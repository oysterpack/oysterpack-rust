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
        use std;

        // Suffixed literals, their types are known at initialization
        let x = 1u8;
        let y = 2u32;
        let z = 3f32;

        // Unsuffixed literal, their types depend on how they are used
        let i = 1;
        let f = 1.0;

        // `size_of_val` returns the size of a variable in bytes
        println!("size of `x` in bytes: {}", std::mem::size_of_val(&x));
        println!("size of `y` in bytes: {}", std::mem::size_of_val(&y));
        println!("size of `z` in bytes: {}", std::mem::size_of_val(&z));
        println!("size of `i` in bytes: {}", std::mem::size_of_val(&i));
        println!("size of `f` in bytes: {}", std::mem::size_of_val(&f));
    }
}
