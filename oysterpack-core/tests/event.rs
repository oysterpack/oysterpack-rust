// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate oysterpack_core;
#[macro_use]
extern crate oysterpack_log;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use oysterpack_core::event::{self, Event};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Foo(String);

// BOILERPLATE THAT CAN BE GENERATED //
pub type FooEvent = Event<Foo>;

pub const FOO_EVENT_ID: event::Id = event::Id(1);

/// Constructor
pub fn new_foo_event(data: Foo) -> FooEvent {
    Event::new(FOO_EVENT_ID, data)
}

// BOILERPLATE THAT CAN BE GENERATED //

#[test]
fn foo_event() {
    let foo_event = new_foo_event(Foo("foo data".into()));
    println!(
        "foo_event: {}",
        serde_json::to_string_pretty(&foo_event).unwrap()
    );
    assert_eq!(foo_event.id(),FOO_EVENT_ID);
    assert_eq!(*foo_event.data(),Foo("foo data".into()));
}
