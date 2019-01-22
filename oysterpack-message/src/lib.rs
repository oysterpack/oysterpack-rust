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

//! OysterPack message features:
//! 1. messages are secured via public-key encryption
//!    - messages encrypted by the sender can only be decrypted by the recipient
//! 2. public keys are used as message addresses
//! 3. [nng](https://nanomsg.github.io/nng/index.html) message conversion

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![allow(clippy::unreadable_literal)]
#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_message/0.1.0")]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;


pub mod envelope;
pub mod errors;
pub mod marshal;
pub mod message;
pub mod op_nng;
pub mod op_thread;
pub mod security;
