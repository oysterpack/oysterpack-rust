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

//! OysterPack Actors

//#![deny(missing_debug_implementations, missing_docs, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_actors/0.1.0")]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate slog;

pub mod registry;
pub mod actor;
pub mod logging;

#[cfg(test)]
mod tests;

//
//extern crate oysterpack_platform as platform;
//
//use std::collections::HashSet;
//
///// Actor Descriptor
//pub struct Descriptor {
//    /// ActorId
//    id: platform::ServiceId,
//    /// Actor settings
//    settings: HashSet<Setting>,
//}
//
///// Actor SettingS
//#[derive(Debug, Hash, Eq, PartialEq, Clone)]
//pub enum Setting {
//    /// Actor mailbox capacity
//    MailboxCapacity(usize),
//}
