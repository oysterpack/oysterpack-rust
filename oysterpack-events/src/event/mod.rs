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

//! Event domain model.

use chrono::{DateTime, Utc};
use oysterpack_log;
use oysterpack_uid::Uid;
use serde::Serialize;
use std::fmt::Debug;

#[cfg(test)]
mod tests;

/// Is applied to some eventful data.
pub trait Eventful: Debug + Send + Sync + Clone + Serialize {
    /// Event Id
    const EVENT_ID: Id;

    /// Event severity level
    const EVENT_SEVERITY_LEVEL: SeverityLevel;

    /// Event constructor
    fn new_event(data: Self) -> Event<Self> {
        Event::new(data)
    }
}

op_newtype!{
    /// EventId(s) are defined as constants. They uniquely identify the event class, i.e., the logical
    /// event.
    ///
    /// ULIDs should be used to avoid collision. ULIDs are not enforced, but is the convention.
    /// We are not using ousterpack_uid::Uid explicitly here because we want the ability to define
    /// event Id(s) as constants.
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
    pub Id(pub u128)
}

impl Id {
    /// converts itself into a Uid
    pub fn as_uid(&self) -> Uid<Self> {
        Uid::from(self.0)
    }
}

/// Represents an Event instance. This is used to define the EventInstanceId type.
#[allow(missing_debug_implementations)]
pub struct Instance;

/// Event instance IDs are generated for each new Event instance that is created.
pub type InstanceId = Uid<Instance>;

/// Represents an event. An event type is identified by its EventId.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<Data>
where
    Data: Debug + Send + Sync + Clone + Eventful,
{
    timestamp: DateTime<Utc>,
    instance_id: InstanceId,
    data: Data,
}

impl<Data> Event<Data>
where
    Data: Debug + Send + Sync + Clone + Eventful,
{
    const EVENT_TARGET_BASE: &'static str = "oysterpack_events";

    /// Constructs the new event and logs it.
    /// The log target will take the form: `oysterpack_events::<event-id>`, where `<event-id>` is
    /// formatted as a ULID, e.g.
    /// - `oysterpack_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    pub fn new(data: Data) -> Event<Data> {
        let event = Event {
            timestamp: Utc::now(),
            instance_id: InstanceId::new(),
            data,
        };
        let target = format!(
            "{}::{}",
            Event::<Data>::EVENT_TARGET_BASE,
            Data::EVENT_ID.as_uid()
        );
        let level = Data::EVENT_SEVERITY_LEVEL.log_level();
        log!(
            target: &target,
            level,
            "{}",
            json!({
        "instance_id":event.instance_id.to_string(),
        "data":event.data
        })
        );
        event
    }

    /// Returns the Event Id
    pub fn id(&self) -> Id {
        Data::EVENT_ID
    }

    /// Returns the Event SeverityLevel
    pub fn severity_level(&self) -> SeverityLevel {
        Data::EVENT_SEVERITY_LEVEL
    }

    /// Returns the event timestamp, i.e., when it occurred.
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Each event instance is assigned a unique id for tracking purposes.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Returns the event data
    pub fn data(&self) -> &Data {
        &self.data
    }
}

/// Class is used to define the event class.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    id: Id,
    severity: SeverityLevel,
    name: Name,
    description: Description,
    category: CategoryId,
}

/// Event severity level
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SeverityLevel {
    /// System is unusable.
    /// A panic condition.
    Emergency,
    /// Action must be taken immediately.
    /// A condition that should be corrected immediately.
    Alert,
    /// Critical conditions
    Critical,
    /// Error conditions
    Error,
    /// Warning conditions
    Warning,
    /// Normal but significant conditions.
    /// Conditions that are not error conditions, but that may require special handling.
    Notice,
    /// Informational messages
    Info,
    /// Debug-level messages.
    /// Messages that contain information normally of use only when debugging.
    Debug,
}

impl SeverityLevel {
    /// Maps SeverityLevel to oysterpack_log::Level
    /// - Debug =&gt; Debug
    /// - Info =&gt; Info
    /// - Notice | Warning =&gt; Warn
    /// - _ =&gt; Error
    pub fn log_level(&self) -> oysterpack_log::Level {
        match self {
            SeverityLevel::Debug => oysterpack_log::Level::Debug,
            SeverityLevel::Info => oysterpack_log::Level::Info,
            SeverityLevel::Notice | SeverityLevel::Warning => oysterpack_log::Level::Warn,
            _ => oysterpack_log::Level::Error,
        }
    }
}

op_newtype! {
    /// Name
    #[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
    pub Name(String)
}

op_newtype! {
    /// Description
    #[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
    pub Description(String)
}

op_newtype! {
    /// Event category
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
    pub CategoryId(pub u128)
}

/// Events are grouped into categories.
/// Categories can be hierarchical.
#[derive(Debug)]
pub struct Category {
    id: CategoryId,
    name: Name,
    description: Description,
    parent_id: Option<CategoryId>,
}
