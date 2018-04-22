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

//! OysterPack Message

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_message/0.1.0")]

extern crate chrono;
extern crate oysterpack_id;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::cell::RefCell;

use self::chrono::prelude::*;

use oysterpack_id::Id;

// TODO: Message encryption

/// Standard Message
///
/// Use [Builder](struct.Builder.html#method.new) to construct new messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    id: MessageId,
    timestamp: TimestampMillis,
    deadline: Deadline,

    correlation_id: Option<MessageId>,
    topic: Topic,
    reply_to: Option<Topic>,

    data: Option<Data>,
}

impl Message {
    /// returns the message id
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// returns the message timestamp, i.e., when the message was created
    pub fn timestamp(&self) -> TimestampMillis {
        self.timestamp
    }

    /// returns the message deadline
    ///
    /// If the deadline is exceeded, then stop processing the message.
    /// The deadline is relative to the message timestamp.
    pub fn deadline(&self) -> Deadline {
        self.deadline
    }

    /// returns the message data
    pub fn data(&self) -> Option<&Data> {
        match self.data {
            Some(ref data) => Some(data),
            None => None,
        }
    }

    /// returns the Topic
    ///
    /// For request-response messaging, this is the default reply to address.
    pub fn topic(&self) -> &Topic {
        &self.topic
    }

    /// returns where to send reply message(s) to
    pub fn reply_to(&self) -> Option<&Topic> {
        match self.reply_to {
            Some(ref reply_to) => Some(reply_to),
            None => Some(&self.topic),
        }
    }

    /// returns optional message correlation id.
    ///
    /// # use case
    /// correlate a response message with a request message
    pub fn correlation_id(&self) -> Option<MessageId> {
        self.correlation_id
    }
}

impl Default for Message {
    fn default() -> Message {
        Message {
            id: MessageId::new(),
            timestamp: TimestampMillis::new(),
            correlation_id: None,
            deadline: Deadline::default(),
            data: None,
            topic: Topic("".to_string()),
            reply_to: None,
        }
    }
}

/// Builder
#[derive(Debug)]
pub struct Builder {
    msg: RefCell<Message>,
}

impl Builder {
    /// returns a new instance with the specified data
    pub fn new(data: Data) -> Builder {
        let mut msg = Message::default();
        msg.data = Some(data);
        Builder {
            msg: RefCell::new(msg),
        }
    }

    /// Set correlation id
    pub fn correlation_id(&self, correlation_id: MessageId) -> &Builder {
        let mut msg = self.msg.borrow_mut();
        msg.correlation_id = Some(correlation_id);
        self
    }

    /// Set sender address id - used as the reply to address id, unless overridden by setting the
    /// reply to address id explicitly.
    pub fn topic(&self, from: Topic) -> &Builder {
        let mut msg = self.msg.borrow_mut();
        msg.topic = from;
        self
    }

    /// Set reply to address id
    pub fn reply_to(&self, reply_to: Topic) -> &Builder {
        let mut msg = self.msg.borrow_mut();
        msg.reply_to = Some(reply_to);
        self
    }

    /// Set deadline
    pub fn deadline(&self, deadline: Deadline) -> &Builder {
        let mut msg = self.msg.borrow_mut();
        msg.deadline = deadline;
        self
    }

    /// terminal operation that returns the message
    pub fn build(self) -> Message {
        self.msg.into_inner()
    }
}

/// unique Message identifier
pub type MessageId = Id<Message>;

/// Message address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic(String);

impl Topic {
    // TODO: validate topic names - cannot be blank, regex
    /// Topic constructor
    pub fn new(topic: &str) -> Topic {
        Topic(topic.trim().to_string())
    }

    /// Topic name
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Deadline
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Deadline {
    /// No Deadline - default value
    None,
    /// Max time allowed for the message to process
    ProcessingTimeoutMillis(u64),
    /// Message timeout is relative to the message timestamp
    MessageTimeoutMillis(u64),
}

impl Default for Deadline {
    fn default() -> Deadline {
        Deadline::None
    }
}

/// Data
///
/// ***NOTE*** turn on compression only after proving that it is needed and provides benefit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    data_type: DataType,
    message_format: MessageFormat,
    data: Vec<u8>,
}

/// unique Data type identifier
pub type DataType = Id<Data>;

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

impl Default for MessageFormat {
    fn default() -> MessageFormat {
        MessageFormat::MessagePack
    }
}

/// number of non-leap-milliseconds since January 1, 1970 UTC
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TimestampMillis(i64);

impl TimestampMillis {
    /// returns the current timestamp
    pub fn new() -> TimestampMillis {
        TimestampMillis(Utc::now().timestamp_millis())
    }
}

#[cfg(test)]
mod test {
    extern crate bincode;
    extern crate exonum_sodiumoxide;
    extern crate rmp_serde as rmps;
    extern crate serde_cbor;
    extern crate serde_json;

    use self::exonum_sodiumoxide::crypto::box_;

    use super::*;

    fn message(data: Vec<u8>) -> Message {
        Builder::new(Data {
            data_type: DataType::new(),
            message_format: MessageFormat::default(),
            data,
        }).build()
    }

    #[test]
    fn serialize_message_json() {
        match serde_json::to_string(&message(vec![])) {
            Ok(json) => {
                println!("{} : {}", &json, json.len());
                let _msg: Message =
                    serde_json::from_str(&json).expect("JSON deserialization failed");
                encrypt_decrypt("json", json.as_bytes());
            }
            Err(err) => panic!("JSON serialization failed : {}", err),
        }
    }

    #[test]
    fn serialize_message_bincode() {
        match bincode::serialize(&message(vec![])) {
            Ok(bytes) => {
                println!("bincode bytes.len() = {}", bytes.len());
                let _msg: Message =
                    bincode::deserialize(&bytes).expect("bincode deserialization failed");
                encrypt_decrypt("bincode", &bytes);
            }
            Err(err) => panic!("bincode serialization failed : {}", err),
        }
    }

    #[test]
    fn serialize_message_cbor() {
        match serde_cbor::to_vec(&message(vec![])) {
            Ok(bytes) => {
                println!("CBOR bytes.len() = {}", bytes.len());
                let _msg: Message =
                    serde_cbor::from_slice(&bytes).expect("CBOR deserialization failed");
                encrypt_decrypt("CBOR", &bytes);
            }
            Err(err) => panic!("CBOR serialization failed : {}", err),
        }
    }

    #[test]
    fn serialize_message_msgpack() {
        let bytes = rmps::to_vec(&message(vec![]));
        match bytes {
            Ok(bytes) => {
                println!("rmps bytes.len() = {}", bytes.len());
                let _msg: Message = rmps::from_slice(&bytes).expect("rmps deserialization failed");
                encrypt_decrypt("rmps", &bytes);
            }
            Err(err) => panic!("rmps serialization failed : {}", err),
        }
    }

    fn encrypt_decrypt(format: &str, bytes: &[u8]) {
        let (ourpk, oursk) = box_::gen_keypair();
        let (theirpk, theirsk) = box_::gen_keypair();
        let our_precomputed_key = box_::precompute(&theirpk, &oursk);
        let nonce = box_::gen_nonce();
        let encrypted_bytes = box_::seal_precomputed(&bytes, &nonce, &our_precomputed_key);
        println!("{} encrypted length = {}", format, encrypted_bytes.len());
        // this will be identical to our_precomputed_key
        let their_precomputed_key = box_::precompute(&ourpk, &theirsk);
        let unencrypted_bytes =
            box_::open_precomputed(&encrypted_bytes, &nonce, &their_precomputed_key).unwrap();

        assert_eq!(unencrypted_bytes, &bytes[..]);
    }

    // impressively, MessagePack (rmps) is the winner
    //    rmps bytes.len() = 41
    //    bincode bytes.len() = 49
    //    CBOR bytes.len() = 119
    //    json len = 181
}
