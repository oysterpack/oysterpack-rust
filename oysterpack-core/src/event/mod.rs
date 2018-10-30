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
use oysterpack_uid::Uid;
use std::fmt::Debug;

op_newtype!{
    /// EventId(s) are defined as constants. They identify the type of event.
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
    pub Id(pub u128)
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
    Data: Debug + Send + Sync + Clone,
{
    id: Id,
    timestamp: DateTime<Utc>,
    instance_id: InstanceId,
    data: Data,
}

impl<Data> Event<Data>
where
    Data: Debug + Send + Sync + Clone,
{
    /// Constructor
    pub fn new(id: Id, data: Data) -> Event<Data> {
        Event {
            id,
            timestamp: Utc::now(),
            instance_id: InstanceId::new(),
            data,
        }
    }

    /// Returns the Event Id
    pub fn id(&self) -> Id {
        self.id
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
    name: String,
    severity: SeverityLevel,
    short_description: String,
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

