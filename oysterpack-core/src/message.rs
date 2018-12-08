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
//!
//! The message stream is compressed.

use bincode;
use chrono::{DateTime, Duration, Utc};
use oysterpack_errors::{Error, Id as ErrorId, IsError, Level as ErrorLevel};
use oysterpack_uid::ULID;
use rmp_serde;
use serde;
use serde_cbor;
use serde_json;
use std::{error, fmt};

/// A private message that is signed and encrypted.
/// - the message is signed by the sender
/// - the message is encrypted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessage {
    header: PrivateMessageHeader,
    msg_header: BinaryData,
    msg_data: BinaryData,
}

/// PrivateMessage header contains the following info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageHeader {
    from: Address,
    to: Address,
    session: SessionId,
    nonce: Nonce,
    encryption: EncryptionMode,
    signature: Signature,
}

/// Message address
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Address(ULID);

/// Session ID correlates to an authenticated session
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SessionId(ULID);

/// Nonce is used to protect against message replay
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Nonce(ULID);

/// Signature is used to prove authenticity
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Signature(Vec<u8>);

/// Binary data
/// - can be compressed
/// - always has a hash to ensure the data integrity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinaryData {
    compression: Option<CompressionMode>,
    hash: Vec<u8>,
    data: Vec<u8>,
}

/// Encryption mode indicates how the message was encrypted
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EncryptionMode {
    /// The message is encrypted using the Private Key corresponding to the specified Public Key
    PrivateKey(PubKeyRef),
    /// The message is encrypted using a shared key that is established within a connection session.
    /// The shared key is known by the sending and receiving parties.
    SharedKey(SessionId),
}

/// Reference to a Public Key
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PubKeyRef(ULID);

/// Compression mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CompressionMode {
    /// deflate
    Deflate,
    /// zlib
    Zlib,
    /// gzip
    Gzip,
    /// snappy
    Snappy,
    /// LZ4
    Lz4,
}

/// Message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<T> {
    header: MessageHeader,
    data: T,
}

/// Message header contains the following info:
/// - the message ID, which identifies the message type. The message ID is used to parse the the message data.
/// - instance ID, which identifies each message instance. This can be used as a nonce to provide replay
///   protection on the network
/// - optional deadline, which specifies that client requires the message to be processed by the specified deadline.
///   - if the message is received after the deadline, then drop the message
///   - message processing can be cancelled once the deadline is reached
///     - the error response can indicate what progress was made back to the client
///     - depending on the use case, this may mean the message processing transaction was rolled back
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageHeader {
    id: Id,
    instance_id: ULID,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline: Option<Deadline>,
}

op_newtype! {
    /// Unique Message type identifier. Messages with the same Request and Response type can have different
    /// message ids. However, a different Message Id implies that it will be potentially processed by
    /// a different processor and have different semantics.
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Id(pub u128)
}

impl MessageHeader {
    /// Each message type is identified by an Id
    pub fn id(&self) -> Id {
        self.id
    }

    /// Each message instance is assigned a unique ULID. This can be used as a nonce for replay protection
    /// on the network.
    pub fn instance_id(&self) -> ULID {
        self.instance_id
    }

    /// When the message was created. This is derived from the message instance ID.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.instance_id.datetime()
    }

    /// A message can specify that it must be processed by the specified deadline.
    pub fn deadline(&self) -> Option<Deadline> {
        self.deadline
    }
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
    type_id: ULID,
    encoding: Encoding,
    data: Vec<u8>,
}

impl Data {
    /// constructor
    pub fn new(type_id: ULID, encoding: Encoding, data: Vec<u8>) -> Data {
        Data {
            type_id,
            encoding,
            data,
        }
    }

    /// data type id getter
    pub fn type_id(&self) -> ULID {
        self.type_id
    }

    /// encoding getter
    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// data getter
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// deserialize the data using the specified encoding
    /// - [SerializationError](struct.SerializationError.html) defines the Error Id and Level constants
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        match self.encoding {
            Encoding::MessagePack => rmp_serde::from_slice(self.data())
                .map_err(|err| op_error!(DeserializationError::new(Encoding::MessagePack, err))),
            Encoding::Bincode => bincode::deserialize(self.data())
                .map_err(|err| op_error!(DeserializationError::new(Encoding::MessagePack, err))),
            Encoding::CBOR => serde_cbor::from_slice(self.data())
                .map_err(|err| op_error!(DeserializationError::new(Encoding::MessagePack, err))),
            Encoding::JSON => serde_json::from_slice(self.data())
                .map_err(|err| op_error!(DeserializationError::new(Encoding::MessagePack, err))),
        }
    }

    /// constructor which serializes the data using the specified encoding
    /// - [DeserializationError](struct.DeserializationError.html) defines the Error Id and Level constants
    pub fn serialize<T: serde::Serialize>(
        type_id: ULID,
        encoding: Encoding,
        data: &T,
    ) -> Result<Data, Error> {
        match encoding {
            Encoding::MessagePack => rmp_serde::to_vec(data)
                .map_err(|err| op_error!(SerializationError::new(Encoding::MessagePack, err))),
            Encoding::Bincode => bincode::serialize(data)
                .map_err(|err| op_error!(SerializationError::new(Encoding::Bincode, err))),
            Encoding::CBOR => serde_cbor::to_vec(data)
                .map_err(|err| op_error!(SerializationError::new(Encoding::CBOR, err))),
            Encoding::JSON => serde_json::to_vec(data)
                .map_err(|err| op_error!(SerializationError::new(Encoding::JSON, err))),
        }
            .map(|data| Data::new(type_id, encoding, data))
    }
}

/// Message encoding format
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Encoding {
    /// [MessagePack](https://msgpack.org/) - default
    MessagePack,
    /// [Bincode](https://github.com/TyOverby/bincode)
    Bincode,
    /// [CBOR](http://cbor.io/)
    CBOR,
    /// [JSON](https://www.json.org/)
    JSON,
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Encoding::MessagePack => f.write_str("MessagePack"),
            Encoding::Bincode => f.write_str("Bincode"),
            Encoding::CBOR => f.write_str("CBOR"),
            Encoding::JSON => f.write_str("JSON"),
        }
    }
}

/// SerializationError
#[derive(Debug)]
pub struct SerializationError {
    encoding: Encoding,
    err_msg: String,
}

impl SerializationError {
    /// Error Id(01CXMQQXSBYCJWWS916JDJN136)
    pub const ERROR_ID: ErrorId = ErrorId(1866174046782305267123345584340763750);
    /// Level::Error
    pub const ERROR_LEVEL: ErrorLevel = ErrorLevel::Error;

    fn new<Msg: fmt::Display>(encoding: Encoding, err_msg: Msg) -> SerializationError {
        SerializationError {
            encoding,
            err_msg: err_msg.to_string(),
        }
    }
}

impl IsError for SerializationError {
    fn error_id(&self) -> ErrorId {
        Self::ERROR_ID
    }

    fn error_level(&self) -> ErrorLevel {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} serialization failed: {}",
            self.encoding, self.err_msg
        )
    }
}

/// DeserializationError
#[derive(Debug)]
pub struct DeserializationError {
    encoding: Encoding,
    err_msg: String,
}

impl DeserializationError {
    /// Error Id(01CXMRB1X1K091BFNN6X37DVDF)
    pub const ERROR_ID: ErrorId = ErrorId(1866174804543832457347080642119527855);
    /// Level::Error
    pub const ERROR_LEVEL: ErrorLevel = ErrorLevel::Error;

    fn new<Msg: fmt::Display>(encoding: Encoding, err_msg: Msg) -> DeserializationError {
        DeserializationError {
            encoding,
            err_msg: err_msg.to_string(),
        }
    }
}

impl IsError for DeserializationError {
    fn error_id(&self) -> ErrorId {
        Self::ERROR_ID
    }

    fn error_level(&self) -> ErrorLevel {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} deserialization failed: {}",
            self.encoding, self.err_msg
        )
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {

    use oysterpack_uid::ULID;
    use crate::tests::run_test;

    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        fname: String,
        lname: String,
    }

    #[test]
    fn deserialize_byte_stream_using_rmp_serde() {
        let p1 = Person {
            fname: "Alfio".to_string(),
            lname: "Zappala".to_string(),
        };
        let p2 = Person {
            fname: "Andreas".to_string(),
            lname: "Antonopoulos".to_string(),
        };

        let mut p1_bytes = rmp_serde::to_vec(&p1).map_err(|_| ()).unwrap();
        let mut p2_bytes = rmp_serde::to_vec(&p2).map_err(|_| ()).unwrap();
        let p1_bytes_len = p1_bytes.len();
        p1_bytes.append(&mut p2_bytes);
        let bytes = p1_bytes.as_slice();
        let p1: Person = rmp_serde::from_read(bytes).unwrap();
        println!("p1: {:?}", p1);
        let p2: Person = rmp_serde::from_read(&bytes[p1_bytes_len..]).unwrap();
        println!("p2: {:?}", p2);
    }
}
