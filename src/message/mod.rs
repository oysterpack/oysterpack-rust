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

//! Message

extern crate chrono;

use self::chrono::prelude::*;

use ::utils::id::Id;

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    id: MessageId,
    timestamp: TimestampMillis,
    correlation_id: Option<CorrelationId>,
    deadline: Deadline,
    data: Vec<Data>,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            id: MessageId::new(),
            timestamp: TimestampMillis::new(),
            correlation_id: None,
            deadline: Deadline::default(),
            data: vec![],
        }
    }
}

/// unique Message identifier
pub type MessageId = Id<Message>;

/// CorrelationId is used to correlate messages, e.g, correlating a response with a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationId(String);

/// Deadline
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Deadline {
    /// No Deadline - default value
    None,
    /// Duration based timeout in milliseconds
    TimeoutMillis(u32),
    /// Time based deadline
    ExpiresOn(DateTime<Utc>),
}

impl Default for Deadline {
    fn default() -> Deadline { Deadline::None }
}

impl Deadline {
    fn timeout_duration(timeout: u32) -> chrono::Duration { chrono::Duration::milliseconds(timeout as i64) }
}

/// Data
///
/// ***NOTE*** turn on compression only after proving that it is needed and provides benefit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    data_type: DataType,
    compression: SerializationFormat,
    data: Vec<u8>,
}

/// unique Data type identifier
pub type DataType = Id<Data>;

/// SerializationFormat
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum SerializationFormat {
    /// [MessagePack](https://msgpack.org/) - default
    MessagePack,
    /// [Bincode](https://github.com/TyOverby/bincode)
    Bincode,
    /// [CBOR](http://cbor.io/)
    CBOR,
    /// [JSON](https://www.json.org/)
    JSON,
}

impl Default for SerializationFormat {
    fn default() -> SerializationFormat { SerializationFormat::MessagePack }
}


/// number of non-leap-milliseconds since January 1, 1970 UTC
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TimestampMillis(i64);

impl TimestampMillis {
    /// returns the current timestamp
    pub fn new() -> TimestampMillis { TimestampMillis(Utc::now().timestamp_millis()) }
}

#[cfg(test)]
mod test {
    extern crate serde_json;
    extern crate bincode;
    extern crate serde_cbor;
    extern crate rmp_serde as rmps;

    use super::*;

    fn message(data: Vec<u8>) -> Message {
        let mut msg = Message::default();
        msg.data = vec![
            Data {
                data_type: DataType::new(),
                compression: SerializationFormat::default(),
                data,
            }
        ];
        msg
    }

    #[test]
    fn serialize_message_json() {
        match serde_json::to_string(&message(vec![])) {
            Ok(json) => {
                println!("{} : {}", &json, json.len());
                let msg: Message = serde_json::from_str(&json).expect("JSON deserialization failed");
            }
            Err(err) => panic!("JSON serialization failed : {}", err)
        }
    }

    #[test]
    fn serialize_message_bincode() {
        match bincode::serialize(&message(vec![])) {
            Ok(bytes) => {
                println!("bincode bytes.len() = {}", bytes.len());
                let msg: Message = bincode::deserialize(&bytes).expect("bincode deserialization failed");
            }
            Err(err) => panic!("bincode serialization failed : {}", err)
        }
    }

    #[test]
    fn serialize_message_cbor() {
        match serde_cbor::to_vec(&message(vec![])) {
            Ok(bytes) => {
                println!("CBOR bytes.len() = {}", bytes.len());
                let msg: Message = serde_cbor::from_slice(&bytes).expect("CBOR deserialization failed");
            }
            Err(err) => panic!("CBOR serialization failed : {}", err)
        }
    }

    #[test]
    fn serialize_message_msgpack() {
        match rmps::to_vec(&message(vec![])) {
            Ok(bytes) => {
                println!("rmps bytes.len() = {}", bytes.len());
                let msg: Message = rmps::from_slice(&bytes).expect("rmps deserialization failed");
            }
            Err(err) => panic!("rmps serialization failed : {}", err)
        }
    }

// impressively, MessagePack (rmps) is the winner
//    rmps bytes.len() = 41
//    bincode bytes.len() = 49
//    CBOR bytes.len() = 119
//    json len = 181
}

