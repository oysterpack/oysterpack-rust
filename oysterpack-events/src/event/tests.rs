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

use super::{Event, Eventful, Id, Level, ModuleSource};
use tests::run_test;
use std::fmt::{
    self, Display, Formatter
};
use oysterpack_uid::{
    IntoGenericUid, Type
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Foo(String);

impl Foo {}

impl Display for Foo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

op_newtype! {
    /// App Id
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
    pub AppId(pub u128)
}

impl IntoGenericUid for AppId {
    const TYPE: Type = Type("App");

    fn id(&self) -> u128 {
        self.0
    }
}

op_newtype! {
    /// App Id
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
    pub ServiceId(pub u128)
}

impl IntoGenericUid for ServiceId {
    const TYPE: Type = Type("Service");

    fn id(&self) -> u128 {
        self.0
    }
}

// BOILERPLATE THAT CAN BE GENERATED //
impl Eventful for Foo {
    const EVENT_ID: Id = Id(1863291442537893263425065359976496302);

    const EVENT_LEVEL: Level = Level::Info;
}

#[test]
fn foo_event() {
    run_test("foo_event", || {
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!());
        assert!(foo_event.tag_ids().is_none());
        info!(
            "foo_event: {}",
            serde_json::to_string_pretty(&foo_event).unwrap()
        );
        info!("{}", foo_event.data());
        foo_event.log();
        let foo_event2: Event<Foo> = serde_json::from_str(&serde_json::to_string_pretty(&foo_event).unwrap()).unwrap();
        assert_eq!(foo_event2.id(),foo_event.id());
        assert_eq!(foo_event.id(), Foo::EVENT_ID);
        assert_eq!(*foo_event.data(), Foo("foo data".into()));
    });
}

#[test]
fn source_code_location_serde() {
    let loc = op_module_source!();
    let loc_json = serde_json::to_string(&loc).unwrap();
    let loc2: ModuleSource = serde_json::from_str(&loc_json).unwrap();
    assert_eq!(loc,loc2);
    let (module, line) = (module_path!(), line!() - 4);
    assert_eq!(module, loc.module_path());
    assert_eq!(line, loc.line());
}

use std::{
    thread,
    sync::mpsc
};

#[test]
fn event_threadsafety() {
    run_test("event_threadsafety",||{
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!());
        let (tx,rx) = mpsc::channel();
        thread::spawn(move || {
            tx.send(foo_event).unwrap();
        });

        let foo_event = rx.recv().unwrap();
        info!("received foo event: {}", foo_event);
    });
}

#[test]
fn event_tags() {
    run_test("event_tags", || {
        let app_id = AppId(1863291903828500526079298022856535457);
        let service_id = ServiceId(1863291948359469739082252902144828404);
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!())
            .with_tag_id(&app_id)
            .with_tag_id(&app_id)
            .with_tag_id(&service_id)
            .with_tag_id(&service_id);

        foo_event.log();
        let tags = foo_event.tag_ids().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&app_id.generic_uid()));
        assert!(tags.contains(&service_id.generic_uid()));
    });
}
