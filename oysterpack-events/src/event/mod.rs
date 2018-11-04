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
use oysterpack_uid::{DomainULID, TypedULID};
use serde::Serialize;
use serde_json;
use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

#[macro_use]
mod macros;

pub mod error;

#[cfg(test)]
mod tests;

/// Is applied to some eventful data.
pub trait Eventful: Debug + Display + Send + Sync + Clone + Serialize {
    /// Event Id
    const EVENT_ID: Id;

    /// Event severity level
    const EVENT_LEVEL: Level;

    /// Event constructor
    fn new_event(data: Self, mod_src: ModuleSource) -> Event<Self> {
        Event::new(Self::EVENT_ID, data, mod_src)
    }
}

op_newtype!{
    /// EventId(s) are defined as constants. They uniquely identify the event class, i.e., the logical
    /// event.
    ///
    /// ULIDs should be used to avoid collision. ULIDs are not enforced, but is the convention.
    /// We are not using oysterpack_uid::TypedULID explicitly here because we want the ability to define
    /// event Id(s) as constants.
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
    pub Id(pub u128)
}

impl Id {
    /// converts itself into a TypedULID
    pub fn as_uid(&self) -> TypedULID<Self> {
        TypedULID::from(self.0)
    }
}

/// Represents an Event instance. This is used to define the EventInstanceId type.
#[allow(missing_debug_implementations)]
pub struct Instance;

/// Event instance IDs are generated for each new Event instance that is created.
pub type InstanceId = TypedULID<Instance>;

/// Event features:
/// - the event class is uniquely identified by an Id
///   - the event Id is defined by the Eventful, which must be implemented by the event's data type
/// - each event instance is assigned a new unique InstanceId
/// - the event data is typed
/// - the source code module and line are captured, which enables events to be easily tracked down where
///   in the code they are being generated from
/// - events are threadsafe
/// - events can be cloned
/// - events are serializable via serde, enabling events to be sent over the network
/// - events can be tagged in order to enable events to be linked to other entities. For example, events
///   can be associated with a service, application, transaction, etc. - as long as the related entity
///   can be identified via a DomainULID.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<Data>
where
    Data: Debug + Display + Send + Sync + Clone + Eventful,
{
    id: TypedULID<Id>,
    instance_id: InstanceId,
    data: Data,
    mod_src: ModuleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_ids: Option<HashSet<DomainULID>>,
}

impl<Data> Event<Data>
where
    Data: Debug + Display + Send + Sync + Clone + Eventful,
{
    const EVENT_TARGET_BASE: &'static str = "op_event";

    /// Constructs the new event and logs it.
    pub fn new(id: Id, data: Data, mod_src: ModuleSource) -> Event<Data> {
        Event {
            id: id.as_uid(),
            instance_id: InstanceId::generate(),
            data,
            mod_src,
            tag_ids: None,
        }
    }

    /// Tags the event
    pub fn with_tag_id(mut self, tag_id: &DomainULID) -> Event<Data> {
        if self.tag_ids.is_none() {
            self.tag_ids = Some(HashSet::new())
        }

        for mut tag_ids in self.tag_ids.iter_mut() {
            tag_ids.insert(tag_id.clone());
        }

        self
    }

    /// Logs the event. The log target will take the form: `op_event::<event-id>`,
    /// where `<event-id>` is formatted as a ULID, e.g.
    /// - `op_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    ///
    /// The message format is pretty JSON, i.e., the event is serialized to pretty JSON.
    /// This will make it easier to read.
    pub fn log(&self) {
        let target = format!(
            "{}::{}",
            Event::<Data>::EVENT_TARGET_BASE,
            Data::EVENT_ID.as_uid()
        );
        log!(
            target: &target,
            Data::EVENT_LEVEL.into(),
            "{}",
            serde_json::to_string_pretty(self).unwrap()
        );
    }

    /// Returns the Event Id
    pub fn id(&self) -> Id {
        Data::EVENT_ID
    }

    /// Returns the Event SeverityLevel
    pub fn severity_level(&self) -> Level {
        Data::EVENT_LEVEL
    }

    /// Returns the event timestamp, i.e., when it occurred.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.instance_id.ulid().datetime()
    }

    /// Each event instance is assigned a unique id for tracking purposes.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Returns the event data
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns tags
    pub fn tag_ids(&self) -> Option<&HashSet<DomainULID>> {
        self.tag_ids.as_ref()
    }
}

impl<Data> std::fmt::Display for Event<Data>
where
    Data: Debug + Display + Send + Sync + Clone + Eventful,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => f.write_str(&json),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

/// Refers to a module source code location.
/// This can be used to include information regarding where an event occurs in the code to provide
/// traceability.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct ModuleSource {
    module_path: String,
    line: u32,
}

impl ModuleSource {
    /// constructor - use the module_path!() and line!() macros provided by rust.
    pub fn new(module_path: &str, line: u32) -> ModuleSource {
        ModuleSource {
            module_path: module_path.to_string(),
            line,
        }
    }

    /// refers source code line number
    pub fn line(&self) -> u32 {
        self.line
    }

    /// refers to the source code module path
    pub fn module_path(&self) -> &str {
        &self.module_path
    }

    /// returns the crate name, which is extracted from the module path
    pub fn crate_name(&self) -> &str {
        self.module_path.split("::").next().unwrap()
    }
}

impl std::fmt::Display for ModuleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.module_path, self.line)
    }
}

/// Class is used to define the event class.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    id: Id,
    level: Level,
    name: Name,
    description: Description,
    category_ids: HashSet<DomainULID>,
}

/// Event severity level
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Level {
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

impl Level {
    /// Returns true of the level indicates the event is error related
    pub fn is_error(self) -> bool {
        match self {
            Level::Error | Level::Critical | Level::Alert | Level::Emergency => true,
            _ => false,
        }
    }
}

impl From<error::Level> for Level {
    fn from(error_level: error::Level) -> Level {
        match error_level {
            error::Level::Emergency => Level::Emergency,
            error::Level::Alert => Level::Alert,
            error::Level::Critical => Level::Critical,
            error::Level::Error => Level::Error,
        }
    }
}

/// Maps SeverityLevel to oysterpack_log::Level
/// - Debug =&gt; Debug
/// - Info =&gt; Info
/// - Notice | Warning =&gt; Warn
/// - _ =&gt; Error
impl Into<oysterpack_log::Level> for Level {
    fn into(self) -> oysterpack_log::Level {
        match self {
            Level::Debug => oysterpack_log::Level::Debug,
            Level::Info => oysterpack_log::Level::Info,
            Level::Notice | Level::Warning => oysterpack_log::Level::Warn,
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
