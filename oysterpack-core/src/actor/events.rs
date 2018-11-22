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
    /// EventId
    pub const EVENT_ID: EventId = EventId(1865187483179844794403987534312933829);

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
    fn event_id(&self) -> EventId {
        ServiceLifeCycleEvent::EVENT_ID
    }

    /// Event severity level
    fn event_level(&self) -> Level {
        match self.state {
            LifeCycle::Restarting => Level::Warning,
            _ => Level::Info,
        }
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
