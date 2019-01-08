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
use crate::marshal;
use chrono::{DateTime, Utc};
use oysterpack_errors::Error;
use oysterpack_uid::{macros::ulid, ULID};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// Used to map a MessageTypeId to a type and provide support for constructing messages
///
/// ## Example
/// ``` rust
/// # use oysterpack_message::message::*;
/// # use serde::*;
/// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// struct GetNextValue;
///
/// impl MessageFactory for GetNextValue {
///    const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1869946728962741078614900012219957643);
/// }
/// ```
pub trait MessageFactory: fmt::Debug + Clone + Send + Serialize + DeserializeOwned {
    /// message type ID
    const MSG_TYPE_ID: MessageTypeId;

    /// message constructor for new session
    fn into_message(self) -> Message<Self> {
        Message {
            header: Header::new(Self::MSG_TYPE_ID),
            body: self,
        }
    }

    /// message constructor for an existing session
    fn into_message_for_session(self, session_id: SessionId) -> Message<Self> {
        Message {
            header: Header::new_for_session(Self::MSG_TYPE_ID, session_id),
            body: self,
        }
    }

    /// message constructor for new session
    fn into_reply_message(self, request: &Header) -> Message<Self> {
        Message {
            header: Header::new_reply(request, Self::MSG_TYPE_ID),
            body: self,
        }
    }

    /// bincode deserialization
    fn decode(msg: &[u8]) -> Result<Self, Error> {
        marshal::deserialize(msg)
    }

    /// bicode serialization
    fn encode(&self) -> Result<Vec<u8>, Error> {
        marshal::serialize(self)
    }

    /// try to decode the BytesMessage
    fn try_decode_message(msg: &Message<BytesMessage>) -> Result<Message<Self>, Error> {
        let body = Self::decode(&msg.body().0)?;
        Ok(Message {
            header: msg.header.clone(),
            body,
        })
    }
}

/// Error message implements MessageFactory. This enables Errors to be returned as messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage(pub Error);

impl MessageFactory for ErrorMessage {
    const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1870100579967440604734092722764315794);
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
        Ok(Message {
            header: self.header,
            body: BytesMessage(marshal::encode(&self.body)?),
        })
    }

    /// Header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Body
    pub fn body(&self) -> &T {
        &self.body
    }

    /// try to encode the message into a `Message<BytesMessage>`
    pub fn try_into_bytes_message(self) -> Result<Message<BytesMessage>, Error> {
        let body = BytesMessage(marshal::serialize(self.body())?);
        Ok(Message {
            header: self.header,
            body,
        })
    }

    /// tries to deserialize the BytesMessage into this message's type
    pub fn try_from_bytes_message(msg: Message<BytesMessage>) -> Result<Message<T>, Error> {
        let body: T = marshal::deserialize(&msg.body().0)?;
        Ok(Message {
            header: msg.header,
            body,
        })
    }
}

impl Message<BytesMessage> {
    /// decompresses the message cia snappy and then deserialize the message via bincode
    pub fn decode<T>(self) -> Result<Message<T>, Error>
    where
        T: fmt::Debug + Clone + Send + serde::Serialize + DeserializeOwned,
    {
        Ok(Message {
            header: self.header,
            body: marshal::decode(self.body.data())?,
        })
    }
}

/// Message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    msg_type_id: MessageTypeId,
    session_id: SessionId,
    instance_id: InstanceId,
    correlation_id: Option<InstanceId>,
    deadline: Option<Deadline>,
    sequence: Option<Sequence>,
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
            session_id,
            instance_id: InstanceId::generate(),
            correlation_id: None,
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
            session_id: request.session_id,
            instance_id: InstanceId::generate(),
            correlation_id: Some(request.instance_id),
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

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::envelope;
    use crate::security;
    use sodiumoxide::crypto::box_;

    #[test]
    fn message() {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        struct GetNextValue;

        impl MessageFactory for GetNextValue {
            const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1869946728962741078614900012219957643);
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        struct NextValue(usize);

        impl MessageFactory for NextValue {
            const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1869947035652420529228505310786809949);
        }

        let request_message = GetNextValue::into_message(GetNextValue);
        let response_message = NextValue(10).into_reply_message(request_message.header());
        assert_eq!(
            request_message.header().session_id(),
            response_message.header().session_id()
        );
        assert_eq!(
            request_message.header().instance_id(),
            response_message.header().correlation_id().unwrap()
        );
        let response_bytes_message = response_message.clone().try_into_bytes_message().unwrap();
        let response_message_2 =
            Message::<NextValue>::try_from_bytes_message(response_bytes_message).unwrap();
        assert_eq!(
            response_message_2.header().instance_id,
            response_message.header().instance_id()
        );
        assert_eq!(response_message_2.body().0, 10);
    }

    #[test]
    fn req_rep_protocol() {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        struct GetNextValue;

        impl MessageFactory for GetNextValue {
            const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1869946728962741078614900012219957643);
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        struct NextValue(usize);

        impl MessageFactory for NextValue {
            const MSG_TYPE_ID: MessageTypeId = MessageTypeId(1869947035652420529228505310786809949);
        }

        let request_message = GetNextValue::into_message(GetNextValue);
        assert_eq!(
            request_message.header().message_type_id(),
            GetNextValue::MSG_TYPE_ID
        );
        let client_session_id = request_message.header().session_id();
        let client_request_instance_id = request_message.header().instance_id();
        let request_message = request_message.try_into_bytes_message().unwrap();
        assert_eq!(client_session_id, request_message.header().session_id());
        assert_eq!(
            client_request_instance_id,
            request_message.header().instance_id()
        );

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let sender = security::Address::from(client_public_key);
        let recipient = security::Address::from(server_public_key);

        let sender_precomputed_key = box_::precompute(recipient.public_key(), &client_private_key);
        let recipient_precomputed_key = box_::precompute(sender.public_key(), &server_private_key);

        let request_nng_message = {
            let request_envelope = envelope::Envelope::new(sender, recipient, request_message);
            let request_envelope = request_envelope.try_into_bytes_message().unwrap();
            let request_sealed_envelope = request_envelope.seal(&sender_precomputed_key);
            request_sealed_envelope.try_into_nng_message().unwrap()
        };

        // client sends request_nng_message
        // server receives request_nng_message
        let request_sealed_envelope =
            envelope::SealedEnvelope::try_from_nng_message(&request_nng_message).unwrap();
        let request_envelope = request_sealed_envelope
            .open(&recipient_precomputed_key)
            .unwrap();
        let request_envelope = request_envelope
            .deserialize::<Message<BytesMessage>>()
            .unwrap();

        let request_message = GetNextValue::try_decode_message(request_envelope.msg()).unwrap();
        let response_message = NextValue(10).into_reply_message(request_message.header());
        assert_eq!(
            response_message.header().message_type_id(),
            NextValue::MSG_TYPE_ID
        );
        assert_eq!(
            response_message.header().correlation_id().unwrap(),
            client_request_instance_id
        );
        assert_eq!(response_message.header().session_id(), client_session_id);
        assert_ne!(
            response_message.header().instance_id(),
            client_request_instance_id
        );
        let response_message = response_message.try_into_bytes_message().unwrap();

        let response_envelope = envelope::Envelope::new(
            *request_envelope.sender(),
            *request_envelope.recipient(),
            response_message,
        );
        let response_envelope = response_envelope.try_into_bytes_message().unwrap();

        let sender_precomputed_key = box_::precompute(sender.public_key(), &server_private_key);
        let response_sealed_envelope = response_envelope.seal(&sender_precomputed_key);
        let response_nng_message = response_sealed_envelope.try_into_nng_message().unwrap();
    }

}
