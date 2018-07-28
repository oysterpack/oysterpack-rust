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

#[macro_use]
extern crate oysterpack_core;

/// this module needs to be in scope for the uid! macro to work
use oysterpack_core::uid;

uid!{
    /// EventId comments can be specified.
    EventId
}

#[test]
fn event_id() {
    let id = EventId::new();
    println!("{:?}", id);
}

uid!{
    Id
}

#[test]
fn id() {
    let id = Id::new();
    println!("{:?}", id);
}

uid_const! {
    /// Unique Command ID
    CommandId
}

#[test]
fn command_id() {
    let id = CommandId(1);
    println!("{:?}", id);
}
