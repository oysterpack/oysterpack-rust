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

//! Actor Events

use super::*;
use oysterpack_events::{event::ModuleSource, Event, Eventful, Id as EventId, Level};
use oysterpack_uid::{Domain, DomainULID};
use std::fmt;

/// Actor Service lifecycle event
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ServiceLifeCycleEvent {
    // the id is stored as a ULID for JSON marshalling purposes - to be compatible with GraphQL.
    // GraphQL only supports signed 32‐bit integers.
    id: ULID,
    instance_id: InstanceId,
    scope: Scope,
    state: LifeCycle,
}

/// Actor lifecycle
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum LifeCycle {
    /// Service has bee started
    ServiceStarted,
    /// Actor has been started
    Started,
    /// Actor has been requested to stop
    Stopping,
    /// Supervised actors may be restarted when failures occur
    Restarting,
    /// If actor does not modify execution context during stopping state actor state changes to Stopped.
    /// This state is considered final and at this point actor get dropped.
    Stopped,
}

impl LifeCycle {
    /// Maps a Service lifecycle event to an EventId
    pub fn event_id(&self) -> EventId {
        match self {
            LifeCycle::ServiceStarted => ServiceLifeCycleEvent::SERVICE_STARTED,
            LifeCycle::Started => ServiceLifeCycleEvent::STARTED,
            LifeCycle::Stopping => ServiceLifeCycleEvent::STOPPING,
            LifeCycle::Restarting => ServiceLifeCycleEvent::RESTARTING,
            LifeCycle::Stopped => ServiceLifeCycleEvent::STOPPED,
        }
    }
}

impl fmt::Display for LifeCycle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LifeCycle::ServiceStarted => f.write_str("ServiceStarted"),
            LifeCycle::Started => f.write_str("Started"),
            LifeCycle::Stopping => f.write_str("Stopping"),
            LifeCycle::Restarting => f.write_str("Restarting"),
            LifeCycle::Stopped => f.write_str("Stopped"),
        }
    }
}

/// Actor service scope
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Scope {
    /// The Actor service is scoped per Arbiter
    Arbiter,
    /// The Actor service is scoped per System
    System,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scope::Arbiter => f.write_str("Arbiter"),
            Scope::System => f.write_str("System"),
        }
    }
}

impl ServiceLifeCycleEvent {
    /// Service lifecycle domain is used to tag Service lifecycle events
    pub const DOMAIN: Domain = Domain("ServiceLifeCycle");
    /// Service lifecycle domain ULID (01CX33EAT3VHNRQ4WNMBS9YH9Q)
    const DOMAIN_ULID: u128 = 1865458711825091376828104373373453623;

    /// Service lifecycle domain ULID
    pub fn domain_ulid() -> DomainULID {
        DomainULID::from_ulid(&Self::DOMAIN, Self::DOMAIN_ULID.into())
    }

    /// Service started EventId (01CX32ZXWW6Z5NKPHMFXB0Y9SV)
    pub const SERVICE_STARTED: EventId = EventId(1865458141241550241048255441954350907);
    /// Actor started EventId (01CX32RTCRQCA3BZ592WAQX0HG)
    pub const STARTED: EventId = EventId(1865457859605975574382982648968413744);
    /// Actor stopping EventId (01CX32TDDHQPT0049EMBTVHPGW)
    pub const STOPPING: EventId = EventId(1865457922771153115754426188305062428);
    /// Supervised Actor restarting Event id (01CX32TSAAKSKRV97RPRJVVTDF)
    pub const RESTARTING: EventId = EventId(1865457937501766424200139812145850799);
    /// Actor stopped EventId (01CX32VJ5NSW78JM82XM75C30S)
    pub const STOPPED: EventId = EventId(1865457968270367213097641946809502745);

    /// Constructs a new event for a Service
    pub fn for_service(service: &impl Service, state: LifeCycle) -> ServiceLifeCycleEvent {
        ServiceLifeCycleEvent {
            id: service.id().into(),
            instance_id: service.instance_id(),
            scope: Scope::Arbiter,
            state,
        }
    }

    /// Constructs a new event for a Service
    pub fn for_app_service(service: &impl AppService, state: LifeCycle) -> ServiceLifeCycleEvent {
        ServiceLifeCycleEvent {
            id: service.id().into(),
            instance_id: service.instance_id(),
            scope: Scope::System,
            state,
        }
    }

    /// Actor Service Id getter
    pub fn id(&self) -> Id {
        self.id.into()
    }

    /// Actor Service InstanceId getter
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Actor Service Scope getter
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// Actor Service LifeCycle state getter
    pub fn state(&self) -> LifeCycle {
        self.state
    }
}

impl Eventful for ServiceLifeCycleEvent {
    /// Event Id
    fn event_id(&self) -> DomainULID {
        DomainULID::from_ulid(&Self::DOMAIN, self.id)
    }

    /// Event severity level
    fn event_level(&self) -> Level {
        match self.state {
            LifeCycle::Restarting => Level::Warning,
            _ => Level::Info,
        }
    }

    fn new_event(self, mod_src: ModuleSource) -> Event<Self> {
        Event::new(self, mod_src).with_tag_id(&Self::domain_ulid())
    }
}

impl fmt::Display for ServiceLifeCycleEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} Actor Service ({}:{}) {}",
            self.scope, self.id, self.instance_id, self.state
        )
    }
}
