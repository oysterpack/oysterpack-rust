/*
 * Copyright 2018 OysterPack Inc.
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

use super::*;
use crate::tests::run_test;
use oysterpack_uid::{Domain, DomainId, DomainULID, HasDomain, ULID};
use serde_json;
use std::fmt::{self, Display, Formatter};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Foo(String);

impl Foo {}

impl Display for Foo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, r#"Foo says : "{}""#, self.0)
    }
}

/// App Id
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AppId(pub u128);

impl HasDomain for AppId {
    const DOMAIN: Domain = Domain("App");
}

impl Into<DomainULID> for AppId {
    fn into(self) -> DomainULID {
        DomainULID::from_ulid(AppId::DOMAIN, ULID::from(self.0))
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ulid = DomainULID::from_ulid(AppId::DOMAIN, ULID::from(self.0));
        write!(f, "{}", ulid)
    }
}

/// Service Id
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ServiceId(pub u128);

impl HasDomain for ServiceId {
    const DOMAIN: Domain = Domain("Service");
}

impl Into<DomainULID> for ServiceId {
    fn into(self) -> DomainULID {
        DomainULID::from_ulid(ServiceId::DOMAIN, ULID::from(self.0))
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ulid = DomainULID::from_ulid(ServiceId::DOMAIN, ULID::from(self.0));
        write!(f, "{}", ulid)
    }
}

impl Foo {
    const EVENT_ID: DomainId = DomainId(Domain("Foo"), 1863291442537893263425065359976496302);

    const EVENT_LEVEL: Level = Level::Info;
}

// BOILERPLATE THAT CAN BE GENERATED //
impl Eventful for Foo {
    fn event_id(&self) -> DomainULID {
        Self::EVENT_ID.as_domain_ulid()
    }

    fn event_level(&self) -> Level {
        Self::EVENT_LEVEL
    }
}

#[test]
fn foo_event() {
    run_test("foo_event", || {
        let foo_event = op_event!(Foo("foo data".into()));
        assert!(foo_event.tag_ids().is_none());
        info!(
            "foo_event: {}",
            serde_json::to_string_pretty(&foo_event).unwrap()
        );
        info!("{}", foo_event.data());
        foo_event.log();
        let foo_event2: Event<Foo> =
            serde_json::from_str(&serde_json::to_string_pretty(&foo_event).unwrap()).unwrap();
        assert_eq!(foo_event2.id(), foo_event.id());
        assert_eq!(foo_event.id().ulid(), Foo::EVENT_ID.ulid());
        assert_eq!(*foo_event.data(), Foo("foo data".into()));
    });
}

#[test]
fn source_code_location_serde() {
    let loc = op_module_source!();
    let loc_json = serde_json::to_string(&loc).unwrap();
    let loc2: ModuleSource = serde_json::from_str(&loc_json).unwrap();
    assert_eq!(loc, loc2);
    let (module, line) = (module_path!(), line!() - 4);
    assert_eq!(module, loc.module_path());
    assert_eq!(line, loc.line());
}

#[test]
fn event_threadsafety() {
    run_test("event_threadsafety", || {
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            tx.send(foo_event).unwrap();
        });

        let foo_event = rx.recv().unwrap();
        info!("received foo event: {}", foo_event);
    });
}

#[test]
fn event_tags() {
    run_test("event_tags with dups", || {
        let app_id = AppId(1863291903828500526079298022856535457);
        let service_id = ServiceId(1863291948359469739082252902144828404);
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!())
            .with_tag_id(app_id.into())
            .with_tag_id(app_id.into())
            .with_tag_id(service_id.into())
            .with_tag_id(service_id.into());

        foo_event.log();
        let tags = foo_event.tag_ids().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&app_id.into()));
        assert!(tags.contains(&service_id.into()));
    });

    run_test("event_tags", || {
        let app_id = AppId(1863291903828500526079298022856535457);
        let service_id = ServiceId(1863291948359469739082252902144828404);
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!())
            .with_tag_id(app_id.into())
            .with_tag_id(service_id.into());

        foo_event.log();
        let tags = foo_event.tag_ids().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&app_id.into()));
        assert!(tags.contains(&service_id.into()));
    });
}

#[test]
fn event_attributes() {
    run_test("event_attributes with dups", || {
        let app_id = super::AttributeId(1863291903828500526079298022856535457);
        let service_id = super::AttributeId(1863291948359469739082252902144828404);
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!())
            .with_attribute(app_id, "A")
            .with_attribute(app_id, "B")
            .with_attribute(service_id, "C")
            .with_attribute(service_id, "D");

        foo_event.log();
        let attributes = foo_event.attributes().unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes.get(&app_id.to_string()).unwrap(), "B");
        assert_eq!(attributes.get(&service_id.to_string()).unwrap(), "D");
    });

    run_test("event_attributes", || {
        let app_id = super::AttributeId(1863291903828500526079298022856535457);
        let service_id = super::AttributeId(1863291948359469739082252902144828404);
        let foo_event = Foo::new_event(Foo("foo data".into()), op_module_source!())
            .with_attribute(app_id, "B")
            .with_attribute(service_id, "D");

        foo_event.log();
        let attributes = foo_event.attributes().unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes.get(&app_id.to_string()).unwrap(), "B");
        assert_eq!(attributes.get(&service_id.to_string()).unwrap(), "D");
    });
}

#[test]
fn error_event() {
    #[derive(Debug, Fail, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
    #[fail(display = "Invalid auth token")]
    struct InvalidAuthToken;

    impl InvalidAuthToken {
        pub const EVENT_ID: DomainId = DomainId(
            Domain("InvalidAuthToken"),
            1863507426672832691683188093609129621,
        );

        const EVENT_LEVEL: Level = Level::Error;
    }

    impl Eventful for InvalidAuthToken {
        fn event_id(&self) -> DomainULID {
            Self::EVENT_ID.as_domain_ulid()
        }

        fn event_level(&self) -> Level {
            Self::EVENT_LEVEL
        }
    }

    run_test("error_event", || {
        let failure = InvalidAuthToken;
        let failure_event = failure.new_event(op_module_source!());
        info!("failure_event: {}", failure_event);
        let failure_event2: Event<InvalidAuthToken> =
            serde_json::from_str(failure_event.to_string().as_str()).unwrap();
        assert_eq!(*failure_event2.data(), *failure_event.data());
    });
}

#[test]
fn ordered_levels() {
    // higher priority comes first
    assert!(Level::Emergency < Level::Alert);
    assert!(Level::Alert < Level::Critical);
    assert!(Level::Critical < Level::Error);
    assert!(Level::Error < Level::Warning);
    assert!(Level::Warning < Level::Notice);
    assert!(Level::Notice < Level::Info);
    assert!(Level::Info < Level::Debug);
}

#[test]
fn event_id_display() {
    let id = ULID::generate();
    let id = Id(id.into());
    assert_eq!(id.to_string(), id.to_string());
}
