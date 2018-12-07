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

//! Event logging Actor service
//!
//! ## Registering Events
//! Events need to be pre-registered before logging events. Thus, services should register any potential
//! events that could occur upon service startup. If events are logged without being preregistered, then
//! they will be tagged as unregistered.
//!
//! The rationale for pre-registering events is that the application should know ahead of time which
//! events can occur. This information is critical to know in order to be able to support the app from
//! a DevOps perspective.
//!
//! The following events are automatically pre-registered:
//! - ServiceLifeCycleEvent::SERVICE_STARTED
//! - ServiceLifeCycleEvent::STARTED
//! - ServiceLifeCycleEvent::STOPPING
//! - ServiceLifeCycleEvent::STOPPED
//! - ServiceLifeCycleEvent::RESTARTING
//! - AppLifeCycleEvent::STARTED
//! - AppLifeCycleEvent::STOPPED

use crate::actor::{
    events::{AppLifeCycleEvent, ServiceLifeCycleEvent},
    AppService, DisplayName, Id as ServiceId, InstanceId as ServiceInstanceId, LifeCycle,
    ServiceInfo,
};

use actix::dev::{Actor, Addr, Context, Handler, Message, MessageResult, System};
use futures::{future, prelude::Future};
use oysterpack_events::{Event, Eventful, Id as EventId};
use oysterpack_uid::{DomainULID, ULID};
use std::{collections::HashSet, iter::FromIterator};

/// ServiceId (01CX6MMENHAXCTZ8WQ0ACEJAAF)
pub const SERVICE_ID: ServiceId = ServiceId(1865602198802033292836235027287714127);

// TODO: the event log should be designed to run within its own dedicated thread.
// - in case the EventLog mailbox is backed up, we don't want it to overwhelm the system Arbiter.
// - one design is to have an event log worker that runs in a dedicated Arbiter. The EventLog system
//   service would simply relay messages to the worker.
/// EventLog App Service
/// - for now simply logs the event - long term we need centralized event logging
#[derive(Debug, Clone)]
pub struct EventLog {
    service_info: ServiceInfo,
    registered_events: HashSet<EventId>,
    unregistered_events: HashSet<EventId>,
}

op_actor_service! {
    AppService(EventLog)
}

impl LifeCycle for EventLog {}

impl DisplayName for EventLog {
    fn name() -> &'static str {
        "EventLog"
    }
}

impl Default for EventLog {
    fn default() -> EventLog {
        let event_ids = vec![
            ServiceLifeCycleEvent::SERVICE_STARTED,
            ServiceLifeCycleEvent::STARTED,
            ServiceLifeCycleEvent::STOPPING,
            ServiceLifeCycleEvent::STOPPED,
            ServiceLifeCycleEvent::RESTARTING,
            AppLifeCycleEvent::STARTED,
            AppLifeCycleEvent::STOPPED,
        ];
        EventLog {
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID, Self::TYPE),
            registered_events: HashSet::from_iter(event_ids),
            unregistered_events: HashSet::new(),
        }
    }
}

/// LogEvent request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent<T>(pub Event<T>)
where
    T: Eventful;

impl<T> Message for LogEvent<T>
where
    T: Eventful,
{
    type Result = ();
}

/// For now, simply logs the event in pretty format.
///
/// If the event is not pre-registered, then it is tagged as unregistered.
impl<T> Handler<LogEvent<T>> for EventLog
where
    T: Eventful,
{
    type Result = MessageResult<LogEvent<T>>;

    fn handle(&mut self, msg: LogEvent<T>, _: &mut Self::Context) -> Self::Result {
        let event_id: EventId = msg.0.id().into();
        let event = if self.registered_events.contains(&event_id) {
            msg.0
        } else {
            self.unregistered_events.insert(event_id);
            msg.0.unregistered()
        };

        event.log_pretty();

        MessageResult(())
    }
}

/// RegisterEvents Request message
#[derive(Debug, Clone)]
pub struct RegisterEvents<EventIds: IntoIterator<Item = EventId>>(pub EventIds);

impl<EventIds: IntoIterator<Item = EventId>> Message for RegisterEvents<EventIds> {
    type Result = ();
}

impl<EventIds: IntoIterator<Item = EventId>> Handler<RegisterEvents<EventIds>> for EventLog {
    type Result = MessageResult<RegisterEvents<EventIds>>;

    fn handle(&mut self, msg: RegisterEvents<EventIds>, _: &mut Self::Context) -> Self::Result {
        for event_id in msg.0 {
            self.registered_events.insert(event_id);
        }
        MessageResult(())
    }
}

/// GetRegisteredEvents request message
#[derive(Debug, Clone)]
pub struct GetRegisteredEvents;

impl Message for GetRegisteredEvents {
    type Result = Vec<EventId>;
}

impl Handler<GetRegisteredEvents> for EventLog {
    type Result = MessageResult<GetRegisteredEvents>;

    fn handle(&mut self, _: GetRegisteredEvents, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.registered_events.iter().cloned().collect())
    }
}

/// GetUnregisteredEvents request message
#[derive(Debug, Clone)]
pub struct GetUnregisteredEvents;

impl Message for GetUnregisteredEvents {
    type Result = Vec<EventId>;
}

impl Handler<GetUnregisteredEvents> for EventLog {
    type Result = MessageResult<GetUnregisteredEvents>;

    fn handle(&mut self, _: GetUnregisteredEvents, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.unregistered_events.iter().cloned().collect())
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::{EventLog, GetRegisteredEvents, GetUnregisteredEvents, LogEvent, RegisterEvents};
    use crate::actor::{
        app::App,
        events::{self, *},
    };
    use oysterpack_events::{
        Eventful, Id as EventId, InstanceId as EventInstanceId, Level as EventLevel,
    };
    use oysterpack_uid::{Domain, DomainULID, HasDomain, ULID};
    use std::{collections::HashSet, fmt, iter::*};

    use crate::actor::logger::init_logging;
    use actix::dev::System;
    use futures::{future, prelude::*};

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    struct Foo;

    impl Foo {
        const EVENT_ID: EventId = EventId(1865605856143420021742978566891916086);
    }

    impl HasDomain for Foo {
        const DOMAIN: Domain = Domain("Foo");
    }

    impl fmt::Display for Foo {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Foo")
        }
    }

    impl Eventful for Foo {
        fn event_id(&self) -> DomainULID {
            DomainULID::from_ulid(&Self::DOMAIN, Self::EVENT_ID.into())
        }

        /// Event severity level
        fn event_level(&self) -> EventLevel {
            EventLevel::Info
        }
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    struct Bar;

    impl Bar {
        const EVENT_ID: EventId = EventId(1865913099798975682410006091752004393);
    }

    impl HasDomain for Bar {
        const DOMAIN: Domain = Domain("Bar");
    }

    impl fmt::Display for Bar {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Bar")
        }
    }

    impl Eventful for Bar {
        fn event_id(&self) -> DomainULID {
            DomainULID::from_ulid(&Self::DOMAIN, Self::EVENT_ID.into())
        }

        /// Event severity level
        fn event_level(&self) -> EventLevel {
            EventLevel::Info
        }
    }

    #[test]
    fn eventlog() {
        App::run(
            crate::build::get(),
            log_config(),
            future::lazy(|| {
                let eventlog = System::current().registry().get::<EventLog>();
                let register_foo_event = eventlog.send(RegisterEvents(vec![Foo::EVENT_ID]));
                let log_foo_event = eventlog.send(LogEvent(op_event!(Foo)));
                let log_bar_event = eventlog.send(LogEvent(op_event!(Bar)));
                let get_registered_events = eventlog.send(GetRegisteredEvents);
                let get_unregistered_events = eventlog.send(GetUnregisteredEvents);
                register_foo_event
                    .then(|_| log_foo_event)
                    .then(|_| log_bar_event)
                    .then(|_| get_registered_events)
                    .then(|registered_events| {
                        let registered_events = registered_events.unwrap();
                        let mut event_ids = HashSet::new();
                        for event_id in registered_events.iter() {
                            event_ids.insert(*event_id);
                        }

                        vec![
                            ServiceLifeCycleEvent::SERVICE_STARTED,
                            ServiceLifeCycleEvent::STARTED,
                            ServiceLifeCycleEvent::STOPPING,
                            ServiceLifeCycleEvent::STOPPED,
                            ServiceLifeCycleEvent::RESTARTING,
                            AppLifeCycleEvent::STARTED,
                            AppLifeCycleEvent::STOPPED,
                            Foo::EVENT_ID,
                        ]
                        .iter()
                        .for_each(|event_id| {
                            assert!(
                                event_ids.contains(event_id),
                                "{:?} DOES NOT CONTAIN {:?}",
                                registered_events,
                                event_id
                            )
                        });

                        assert!(!event_ids.contains(&Bar::EVENT_ID));

                        get_unregistered_events
                    })
                    .then(|unregistered_events| {
                        let unregistered_events = unregistered_events.unwrap();
                        info!("unregistered_events: {:?}", unregistered_events);
                        assert_eq!(unregistered_events.len(), 1);
                        assert_eq!(*unregistered_events.get(0).unwrap(), Bar::EVENT_ID);
                        future::ok::<(), ()>(())
                    })
            }),
        );
    }
}
