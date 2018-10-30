// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate oysterpack_events;
#[macro_use]
extern crate oysterpack_log;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use oysterpack_events::event::{self, Event, Eventful};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Foo(String);

impl Foo {
    /// Event ID
    const FOO_EVENT_ID: event::Id = event::Id(1);
}

// BOILERPLATE THAT CAN BE GENERATED //
impl Eventful for Foo {
    fn event_id() -> event::Id {
        Foo::FOO_EVENT_ID
    }

    fn event_severity_level() -> event::SeverityLevel {
        event::SeverityLevel::Info
    }

    fn new_event(data: Foo) -> Event<Foo> {
        Event::new(Foo::FOO_EVENT_ID, data)
    }
}

#[test]
fn foo_event() {
    let foo_event = Foo::new_event(Foo("foo data".into()));
    println!(
        "foo_event: {}",
        serde_json::to_string_pretty(&foo_event).unwrap()
    );
    assert_eq!(foo_event.id(), Foo::FOO_EVENT_ID);
    assert_eq!(*foo_event.data(), Foo("foo data".into()));
}
