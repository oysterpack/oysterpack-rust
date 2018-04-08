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
//! Because [Rust](https://www.rust-lang.org) is the best systems programming language for building production grade reactive systems.
//!
//! Rust's focus on **safety**, **speed**, and **concurrency** delivers the performance and control of a low-level language, but with the powerful abstractions of a high-level language.
//!

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/oysterpack/0.1.1")]

#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate lazy_static;

pub mod platform;
pub mod utils;

pub use utils::id::Id;