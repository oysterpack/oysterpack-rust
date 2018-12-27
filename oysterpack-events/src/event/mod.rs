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

//! Provides event standardization

use chrono::{DateTime, Utc};
use oysterpack_log;
use oysterpack_uid::{
    macros::{domain, ulid},
    Domain, DomainId, DomainULID,
};
use serde::Serialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};

#[cfg(test)]
mod tests;

/// Is applied to some eventful data.
pub trait Eventful: Debug + Display + Send + Sync + Clone + Serialize {
    /// Event Id
    fn event_id(&self) -> DomainULID;

    /// Event severity level
    fn event_level(&self) -> Level;

    /// Event constructor
    fn new_event(self, mod_src: ModuleSource) -> Event<Self> {
        Event::new(self, mod_src)
    }
}

#[domain(Event)]
#[ulid]
/// Event ID
pub struct Id(pub u128);

#[domain(EventAttribute)]
#[ulid]
/// AttributeId(s) are defined as constants. The intent is to use them as attribute keys to ensure
/// there is no collision
///
/// ULIDs should be used to avoid collision. ULIDs are not enforced, but is the convention.
/// We are not using oysterpack_uid::TypedULID explicitly here because we want the ability to define
/// event Id(s) as constants.
pub struct AttributeId(pub u128);

#[domain(EventInstance)]
#[ulid]
/// Event instance ID
pub struct InstanceId(pub u128);

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<Data>
where
    Data: Debug + Display + Send + Sync + Clone + Eventful,
{
    id: DomainULID,
    instance_id: InstanceId,
    data: Data,
    msg: String,
    mod_src: ModuleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_ids: Option<HashSet<DomainULID>>,
    // using a AttributeId as the key because over time the key label may change, but the key ULID remains const
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, String>>,
}

impl<Data> Event<Data>
where
    Data: Debug + Display + Send + Sync + Clone + Eventful,
{
    /// Event log target base.
    ///
    /// The log target will take the form: `op_event::<event-id>`,
    /// where `<event-id>` is formatted as a ULID, e.g.
    /// - `op_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    pub const EVENT_TARGET_BASE: &'static str = "op_event";

    /// Event log target base.
    ///
    /// The log target will take the form: `op_event::<event-id>`,
    /// where `<event-id>` is formatted as a ULID, e.g.
    /// - `op_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    pub fn log_target_base() -> oysterpack_log::config::Target {
        oysterpack_log::config::Target::new(Self::EVENT_TARGET_BASE.to_string())
    }

    /// Constructor
    pub fn new(data: Data, mod_src: ModuleSource) -> Event<Data> {
        let msg = data.to_string();
        Event {
            id: data.event_id().clone(),
            instance_id: InstanceId::generate(),
            data,
            msg,
            mod_src,
            tag_ids: None,
            attributes: None,
        }
    }

    /// Constructor
    pub fn from(instance_id: InstanceId, data: Data, mod_src: ModuleSource) -> Event<Data> {
        let msg = data.to_string();
        Event {
            id: data.event_id().clone(),
            instance_id,
            data,
            msg,
            mod_src,
            tag_ids: None,
            attributes: None,
        }
    }

    /// Tags the event. DomainULID(s) are used to tag the Event.
    pub fn with_tag_id(self, tag_id: DomainULID) -> Event<Data> {
        let mut event = self;
        if event.tag_ids.is_none() {
            event.tag_ids = Some(HashSet::new())
        }

        {
            let tag_ids = event.tag_ids.iter_mut().next().unwrap();
            tag_ids.insert(tag_id);
        }

        event
    }

    /// Tags the event. DomainULID(s) are used to tag the Event.
    pub fn with_attribute<K: Display, V: Display>(self, key: K, value: V) -> Event<Data> {
        let mut event = self;
        if event.attributes.is_none() {
            event.attributes = Some(HashMap::new())
        }

        {
            let attributes = event.attributes.iter_mut().next().unwrap();
            attributes.insert(key.to_string(), value.to_string());
        }

        event
    }

    /// Logs the event. The log target will take the form: `op_event::<event-id>`,
    /// where `<event-id>` is formatted as a ULID, e.g.
    /// - `op_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    ///
    /// The message format is JSON.
    pub fn log(&self) {
        let target = format!(
            "{}::{}",
            Event::<Data>::EVENT_TARGET_BASE,
            self.data.event_id().ulid()
        );
        log!(
            target: &target,
            self.data.event_level().into(),
            "{}",
            serde_json::to_string(self).unwrap()
        );
    }

    /// Logs the event. The log target will take the form: `op_event::<event-id>`,
    /// where `<event-id>` is formatted as a ULID, e.g.
    /// - `op_event::01CV38FM3Z4M2A8G50QRTGJHP4`
    ///
    /// The message format is pretty JSON, i.e., the event is serialized to pretty JSON.
    /// This will make it easier to read.
    pub fn log_pretty(&self) {
        let target = format!(
            "{}::{}",
            Event::<Data>::EVENT_TARGET_BASE,
            self.data.event_id().ulid()
        );
        log!(
            target: &target,
            self.data.event_level().into(),
            "{}",
            serde_json::to_string_pretty(self).unwrap()
        );
    }

    /// Returns the Event Id
    pub fn id(&self) -> DomainULID {
        self.data.event_id()
    }

    /// Returns the Event SeverityLevel
    pub fn severity_level(&self) -> Level {
        self.data.event_level()
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

    /// Returns the event message, which is meant to be in human readable form derived from the
    /// event's data.
    pub fn message(&self) -> &str {
        &self.msg
    }

    /// Returns where in the source code base the event was created
    pub fn module_source(&self) -> &ModuleSource {
        &self.mod_src
    }

    /// Returns tags
    pub fn tag_ids(&self) -> Option<&HashSet<DomainULID>> {
        self.tag_ids.as_ref()
    }

    /// Returns attributes
    pub fn attributes(&self) -> Option<&HashMap<String, String>> {
        self.attributes.as_ref()
    }

    /// tags the event as unregistered
    pub fn unregistered(self) -> Self {
        self.with_tag_id(UNREGISTERED_EVENT.as_domain_ulid())
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

/// Event severity level
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

/// Domain for unregistered events.
///
/// All possible Event(s) that can be produced by the App should be pre-registered.
/// This enables one to know and beware of the kinds of events that an App can produce.
/// This makes it easier to support the application.
///
/// ULID(01CXE7Q2CYC0ZT2YSZGBBKV757)
pub const UNREGISTERED_EVENT: DomainId = DomainId(
    Domain("UnregisteredEvent"),
    1865910341552439472821557183186312359,
);
