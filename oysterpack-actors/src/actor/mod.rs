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

//! The actor module provides the standard for OysterPack Actors:
//!
//! 1. All Actor instances will be assigned a unique ActorId

extern crate actix;
extern crate chrono;
extern crate futures;
extern crate oysterpack_id;

#[cfg(test)]
mod tests;

pub mod service;

use self::actix::prelude::*;
use self::futures::prelude::*;

///// Actor SettingS
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Setting {
    /// Actor mailbox capacity
    MailboxCapacity(usize),
}
