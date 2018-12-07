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

//! This crate standardizes events for the OysterPack platform.
//!
//! ![uml](ml.svg)

#![deny(missing_docs, missing_debug_implementations)]
#![allow(clippy::unreadable_literal)]
#![doc(html_root_url = "https://docs.rs/oysterpack_events/0.1.0")]

#[macro_use]
extern crate oysterpack_macros;
#[allow(unused_imports)]
#[macro_use]
extern crate oysterpack_log;

#[macro_use]
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate failure;

#[macro_use]
mod macros;
pub mod event;

pub use crate::event::{Event, Eventful, Id, InstanceId, Level};
pub use crate::macros::*;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;

#[cfg(test)]
op_tests_mod!();
