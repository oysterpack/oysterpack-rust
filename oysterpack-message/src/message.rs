/*
 * Copyright 2019 OysterPack Inc.
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

//! defines message layer
//! - bincode is used for serialization
//! - snappy is used for compression

use crate::envelope::BytesMessage;
use crate::errors;
use chrono::{DateTime, Utc};
use oysterpack_errors::{op_error, Error, ErrorMessage};
use oysterpack_uid::{macros::ulid, ULID};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt, marker::PhantomData};

/// Represents a message specification.
/// - maps a MessageTypeId to type `T`.
/// - knows how to decode / encode messages
///   - messages are serialized via [bincode](https://crates.io/crates/bincode) and compressed via [snappy](https://google.github.io/snappy/).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Factory<T>(MessageTypeId, PhantomData<T>)
where
    T: fmt::Debug + Clone + Send + Serialize + DeserializeOwned;

impl<T> Factory<T>
where
    T: fmt::Debug + Clone + Send + Serialize + DeserializeOwned,
{
    /// unique message type ID defined as a ULID
    pub fn type_id(&self) -> MessageTypeId {
        self.0
    }

    /// message constructor for new session
    pub fn new_message(&self, body: &[u8]) -> Result<Message<T>, Error> {
        let header = Header::new(self.0);
        let body: T = self.decode(body)?;
        Ok(Message { header, body })
    }

    /// message constructor for an existing session
    pub fn new_message_for_session(
        &self,
        body: &[u8],
        session_id: SessionId,
    ) -> Result<Message<T>, Error> {
        let header = Header::new_for_session(self.0, session_id);
        let body: T = self.decode(body)?;
        Ok(Message { header, body })
    }

    /// message constructor for new session
    pub fn new_reply_message(&self, body: &[u8], request: &Header) -> Result<Message<T>, Error> {
        let header = Header::new_reply(request, self.0);
        let body: T = self.decode(body)?;
        Ok(Message { header, body })
    }

    /// message constructor for new session
    pub fn new_reply_message_for_session(
        &self,
        body: &[u8],
        request: &Header,
        session_id: SessionId,
    ) -> Result<Message<T>, Error> {
        let header = Header::new_reply_for_session(request, self.0, session_id);
        let body: T = self.decode(body)?;
        Ok(Message { header, body })
    }

    /// decompresses the message cia snappy and then deserialize the message via bincode
    pub fn decode(&self, msg: &[u8]) -> Result<T, Error> {
        let decompressed_msg = parity_snappy::decompress(msg).map_err(|err| {
            op_error!(errors::SnappyDecompressError(ErrorMessage(err.to_string())))
        })?;
        bincode::deserialize(&decompressed_msg).map_err(|err| {
            op_error!(errors::BincodeDeserializeError(
                errors::Scope::BytesMessage,
                ErrorMessage(err.to_string())
            ))
        })
    }

    /// compresses the bincode serialized message via snappy
    pub fn encode(&self, msg: &T) -> Result<Vec<u8>, Error> {
        let bytes = bincode::serialize(msg).map_err(|err| {
            op_error!(errors::BincodeSerializeError(
                errors::Scope::BytesMessage,
                ErrorMessage(err.to_string())
            ))
        })?;

        Ok(parity_snappy::compress(&bytes))
    }
}

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T>
where
    T: fmt::Debug + Clone + Send,
{
    header: Header,
    body: T,
}

impl<T> Message<T>
where
    T: fmt::Debug + Clone + Send + serde::Serialize + DeserializeOwned,
{
    /// compresses the bincode serialized message via snappy
    pub fn encode(self) -> Result<Message<BytesMessage>, Error> {
        let bytes = bincode::serialize(&self.body).map_err(|err| {
            op_error!(errors::BincodeSerializeError(
                errors::Scope::BytesMessage,
                ErrorMessage(err.to_string())
            ))
        })?;

        Ok(Message {
            header: self.header,
            body: BytesMessage(parity_snappy::compress(&bytes)),
        })
    }
}

impl Message<BytesMessage> {
    /// decompresses the message cia snappy and then deserialize the message via bincode
    pub fn decode<T>(self) -> Result<Message<T>, Error>
    where
        T: fmt::Debug + Clone + Send + serde::Serialize + DeserializeOwned,
    {
        let decompressed_msg = parity_snappy::decompress(self.body.data()).map_err(|err| {
            op_error!(errors::SnappyDecompressError(ErrorMessage(err.to_string())))
        })?;
        let body: T = bincode::deserialize(&decompressed_msg).map_err(|err| {
            op_error!(errors::BincodeDeserializeError(
                errors::Scope::BytesMessage,
                ErrorMessage(err.to_string())
            ))
        })?;
        Ok(Message {
            header: self.header,
            body,
        })
    }
}

/// Message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    msg_type_id: MessageTypeId,
    session_id: SessionId,
    instance_id: InstanceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: Option<InstanceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline: Option<Deadline>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sequence: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, Vec<u8>>>,
}

impl Header {
    /// constructor
    /// - generates a new SessionId
    pub fn new(msg_type_id: MessageTypeId) -> Header {
        Header {
            msg_type_id,
            session_id: SessionId::generate(),
            instance_id: InstanceId::generate(),
            correlation_id: None,
            correlation_session_id: None,
            deadline: None,
            sequence: None,
            attributes: None,
        }
    }

    /// constructor
    /// - for a pre-existing SessionId
    pub fn new_for_session(msg_type_id: MessageTypeId, session_id: SessionId) -> Header {
        Header {
            msg_type_id,
            session_id: session_id,
            instance_id: InstanceId::generate(),
            correlation_id: None,
            correlation_session_id: None,
            deadline: None,
            sequence: None,
            attributes: None,
        }
    }

    /// creates a new reply header based off the request header
    /// - correlates the reply to the request
    /// - correlates the request session
    pub fn new_reply(request: &Header, msg_type_id: MessageTypeId) -> Header {
        Header {
            msg_type_id,
            session_id: SessionId::generate(),
            instance_id: InstanceId::generate(),
            correlation_id: Some(request.instance_id),
            correlation_session_id: Some(request.session_id),
            deadline: None,
            sequence: None,
            attributes: None,
        }
    }

    /// creates a new reply header based off the request header
    /// - correlates the reply to the request
    /// - correlates the request session
    pub fn new_reply_for_session(
        request: &Header,
        msg_type_id: MessageTypeId,
        session_id: SessionId,
    ) -> Header {
        Header {
            msg_type_id,
            session_id: session_id,
            instance_id: InstanceId::generate(),
            correlation_id: Some(request.instance_id),
            correlation_session_id: Some(request.session_id),
            deadline: None,
            sequence: None,
            attributes: None,
        }
    }

    /// sets a header attribute
    pub fn set_attribute(self, key: String, value: &[u8]) -> Header {
        let mut header = self;
        if header.attributes.is_none() {
            let mut attrs = HashMap::new();
            attrs.insert(key, value.into());
            header.attributes = Some(attrs);
            return header;
        }
        let mut attrs = header.attributes.take().unwrap();
        attrs.insert(key, value.into());
        header.attributes = Some(attrs);
        header
    }

    /// sets header attributes
    pub fn set_attributes(self, attrs: HashMap<String, Vec<u8>>) -> Header {
        let mut header = self;
        header.attributes = Some(attrs);
        header
    }

    /// set the message processing deadline
    pub fn set_deadline(self, deadline: Deadline) -> Header {
        Header {
            deadline: Some(deadline),
            ..self
        }
    }

    /// sets the message sequence
    /// - the use case is to enable sequential / ordered message processing
    /// - the sequential / ordering semantics are specified by the server
    pub fn set_sequence(self, sequence: Sequence) -> Header {
        Header {
            sequence: Some(sequence),
            ..self
        }
    }

    /// correlation ID getter
    pub fn correlation_id(&self) -> Option<InstanceId> {
        self.correlation_id
    }

    /// correlation SessionId getter
    pub fn correlation_session_id(&self) -> Option<SessionId> {
        self.correlation_session_id
    }

    /// Each message type is identified by an ID
    pub fn message_type_id(&self) -> MessageTypeId {
        self.msg_type_id
    }

    /// Each message instance is assigned a unique ULID.
    /// - This could be used as a nonce for replay protection on the network.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// When the message was created. This is derived from the message instance ID.
    ///
    /// ## NOTES
    /// The timestamp has millisecond granularity. If sub-millisecond granularity is required, then
    /// a numeric sequence based nonce would be required.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.instance_id.ulid().datetime()
    }

    /// A message can specify that it must be processed by the specified deadline.
    pub fn deadline(&self) -> Option<Deadline> {
        self.deadline
    }

    /// Message sequence is relative to the current session.
    ///
    /// No message sequence implies that messages can be processed in any order.
    ///
    /// ## Use Cases
    /// 1. The client-server protocol can use the sequence to strictly process messages in order.
    ///    For example, if the client sends a message with sequence=2, the sequence=2 message will
    ///    not be processed until the server knows that sequence=1 message had been processed.
    ///    The sequence=2 message will be held until sequence=1 message is received. The sequencing
    ///    protocol can be negotiated between the client and server.
    pub fn sequence(&self) -> Option<Sequence> {
        self.sequence
    }

    /// A message can be associated with a session.
    /// In a request / response protocol, the response should reply with the request SessionId.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// returns the header attrbute for the specified key
    pub fn attribute(&self, key: &str) -> Option<&[u8]> {
        match &self.attributes {
            None => None,
            Some(attrs) => attrs.get(key).map(|value| value.as_slice()),
        }
    }

    /// returns the header attribute keys
    pub fn attribute_keys(&self) -> Option<Vec<&str>> {
        match &self.attributes {
            None => None,
            Some(attrs) => {
                let mut keys = Vec::with_capacity(attrs.len());
                for key in attrs.keys() {
                    keys.push(key.as_str());
                }
                Some(keys)
            }
        }
    }
}

/// Message sequence
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Sequence(pub u64);

/// Each new client connection is assigned a new SessionId
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SessionId(ULID);

impl SessionId {
    /// constructor
    pub fn generate() -> SessionId {
        SessionId(ULID::generate())
    }

    /// session ULID
    pub fn ulid(&self) -> ULID {
        self.0
    }
}

/// Deadline
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Deadline {
    /// Max time allowed for the message to process
    ProcessingTimeoutMillis(u64),
    /// Message timeout is relative to the message timestamp
    MessageTimeoutMillis(u64),
}

/// Message type ULID
#[ulid]
pub struct MessageTypeId(pub u128);

/// Message type ULID
#[ulid]
pub struct InstanceId(pub u128);
