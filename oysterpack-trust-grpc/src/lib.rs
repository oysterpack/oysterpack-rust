/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! **oysterpack-trust-nng** provides support for building robust scalable distributed applications
//! leveraging proven [nng](https://nanomsg.github.io/nng/) messaging technology.
//!
//! nng provides the advanced messaging protocol-specific processing. Rust provides the power of
//! fearless concurrency and futures for message processing.

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
//#![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_trust_grpc/0.1.0")]

#[allow(unused_imports)]
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[allow(missing_debug_implementations)]
pub mod protos;

#[cfg(test)]
fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info)
        .target_level(
            oysterpack_log::Target::from(env!("CARGO_PKG_NAME")),
            oysterpack_log::Level::Debug,
        )
        .build()
}

#[cfg(test)]
fn configure_logging() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
}
