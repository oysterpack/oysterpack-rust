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

//! Message package

use chrono::{DateTime, Duration, Utc};
use oysterpack_uid::ULID;

/// Message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<T> {
    id: Id,
    instance_id: ULID,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline: Option<Deadline>,
    data: T,
}

op_newtype! {
    /// Unique Message type identifier. Messages with the same Request and Response type can have different
    /// message ids. However, a different Message Id implies that it will be potentially processed by
    /// a different processor and have different semantics.
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Id(pub u128)
}

/// Deadline
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Deadline {
    /// Max time allowed for the message to process
    ProcessingTimeoutMillis(u64),
    /// Message timeout is relative to the message timestamp
    MessageTimeoutMillis(u64),
}

/// Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    data_type: ULID,
    message_format: MessageFormat,
    data: Vec<u8>,
}

/// SerializationFormat
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MessageFormat {
    /// [MessagePack](https://msgpack.org/) - default
    MessagePack,
    /// [Bincode](https://github.com/TyOverby/bincode)
    Bincode,
    /// [CBOR](http://cbor.io/)
    CBOR,
    /// [JSON](https://www.json.org/)
    JSON,
    /// The message data is treated simply as bytes - it's up to the message producer / consumer
    BYTES,
}
