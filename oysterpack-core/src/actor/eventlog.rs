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

use actor::{Id as ServiceId, InstanceId as ServiceInstanceId, LifeCycle, ServiceInfo};

use actix::dev::{Actor, Addr, Context, Handler, Message, MessageResult, System};
use futures::{future, prelude::Future};
use oysterpack_events::{Event, Eventful};

/// ServiceId (01CX6MMENHAXCTZ8WQ0ACEJAAF)
pub const SERVICE_ID: ServiceId = ServiceId(1865602198802033292836235027287714127);

/// EventLog App Service
/// - for now simply logs the event - long term we need centralized event logging
#[derive(Debug, Clone)]
pub struct EventLog {
    service_info: ServiceInfo,
}

op_actor_service! {
    AppService(EventLog)
}

impl LifeCycle for EventLog {}

impl Default for EventLog {
    fn default() -> EventLog {
        EventLog {
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID),
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

/// For now, simply logs the event in pretty format
impl<T> Handler<LogEvent<T>> for EventLog
where
    T: Eventful,
{
    type Result = MessageResult<LogEvent<T>>;

    fn handle(&mut self, msg: LogEvent<T>, _: &mut Self::Context) -> Self::Result {
        msg.0.log_pretty();
        MessageResult(())
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::{EventLog, LogEvent};
    use crate::actor::{app::App, events};
    use oysterpack_events::{
        Eventful, Id as EventId, InstanceId as EventInstanceId, Level as EventLevel,
    };
    use oysterpack_uid::{Domain, DomainULID, HasDomain, ULID};
    use std::fmt;

    use actix::dev::System;
    use crate::actor::logger::init_logging;
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

    #[test]
    fn eventlog() {
        App::run(
            ::build::get(),
            log_config(),
            future::lazy(|| {
                let eventlog = System::current().registry().get::<EventLog>();
                eventlog
                    .send(LogEvent(op_event!(Foo)))
                    .then(|_| future::ok::<(), ()>(()))
            }),
        );
    }
}
