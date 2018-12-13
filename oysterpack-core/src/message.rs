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

//! Message package. Messages are designed to be highly secure.
//!
//! Messages are processed as streams, transitioning states while being processed.
//!
//! - when a peer connects, the initial message is the handshake.
//!   - each peer is identified by a public-key
//!   - the connecting peer plays the role of `client`; the peer being connected to plays the role of
//!     `server`
//!   - the client initiates a connection with a server by encrypting a `Connect` message using the
//!     server's public-key. Thus, only a specific server can decrypt the message.
//!   - the connect message contains a `PaymentChannel`
//!     - the client must commit funds in order to do business with the server
//!     - all payments are in Bitcoin
//!   - the client establishes a payment channel using secured funds
//!     - all payments are made via cryptocurrency
//!       - Bitcoin will initially be supported
//!       - payment is enforced via a smart contract
//!         - the smart contract defines the statement of work
//!         - funds are secured on a payment channel via a smart contract
//!         - the server provides proof of work to collect payment
//!         - when the connection is terminated, the server closes the contract and gets paid
//!           - change is returned to the client
//!     - each message contains a payment transaction
//!     - all messages processing fees are flat rates
//!       - a flat rate per unit of time for the connection
//!       - a flat rate per message byte
//!       - a flat rate for each message type
//!   - if the server successfully authenticates the client, then the server will reply with a
//!     `ConnectAccepted` reply
//!     - the message contains a shared secret cipher, which will be used to encrypt all future messages
//!       on this connection
//!       - the cipher expires and will be renewed by the server automatically
//!         - the server may push to the client a new cipher key. The client should switch over to using
//!           the new cipher key effective immediately
//!     - the message is hashed
//!     - the hash is digitally signed by the server
//!     - the message is encrypted using the client's private-key
//!
//! - when a peer comes online they register themselves with the services they provide
//!   - this enables clients to discover peers that offer services that the client is interested in
//!   - peers can advertise service metadata
//!     - service price
//!     - quality of service
//!     - capacity
//!     - hardware specs
//!     - smart contract
//!       - specifies message processing terms, prices, and payments
//!   - realtime metrics will be collected, which can help clients choose servers
//!   - clients can rate the server
//! - servers can blacklist clients that are submitting invalid requests
//! - clients can bid for services
//!   - clients can get immediate service if they pay the service ask price
//!   - clients can bid for a service at a lower price, sellers may choose to take the lower price
//!   - clients can bid higher, if service supply is low, in order to get higher priority
//!
//! ### Notes
//! - rmp_serde does not support Serde #[serde(skip_serializing_if="Option::is_none")] - it fails
//!   on deserialization - [https://github.com/3Hren/msgpack-rust/issues/86]
//!   - take away lesson is don't use the Serde #[serde(skip_serializing_if="Option::is_none")] feature
//!

use bincode;
use chrono::{DateTime, Duration, Utc};
use exonum_sodiumoxide::crypto::{box_, hash, secretbox, sign};
use oysterpack_errors::{Error, Id as ErrorId, IsError, Level as ErrorLevel};
use oysterpack_uid::ULID;
use rmp_serde;
use serde;
use serde_cbor;
use serde_json;
use std::{cmp, error, fmt, io};

pub mod base58;
pub mod codec;
pub mod errors;

/// Max message size - 256 KB
pub const MAX_MSG_SIZE: usize = 1000 * 256;

/// Min message size for SealedEnvelope using MessagePack encoding
pub const SEALED_ENVELOPE_MIN_SIZE: usize = 90;

/// A sealed envelope is secured via public-key authenticated encryption. It contains a private message
/// that is encrypted using the recipient's public-key and the sender's private-key. If the recipient
/// is able to decrypt the message, then the recipient knows it was sealed by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedEnvelope {
    sender: Address,
    recipient: Address,
    nonce: box_::Nonce,
    msg: EncryptedMessageBytes,
}

impl SealedEnvelope {
    /// decodes the io stream to construct a new SealedEnvelope
    pub fn decode<R>(read: R) -> Result<SealedEnvelope, Error>
    where
        R: io::Read,
    {
        rmp_serde::from_read(read).map_err(|err| {
            op_error!(errors::MessageError::DecodingError(
                errors::DecodingError::InvalidSealedEnvelope(err)
            ))
        })
    }

    /// encode the SealedEnvelope and write it to the io stream
    pub fn encode<W: ?Sized>(&self, wr: &mut W) -> Result<(), Error>
    where
        W: io::Write,
    {
        rmp_serde::encode::write(wr, self).map_err(|err| {
            op_error!(errors::MessageError::EncodingError(
                errors::EncodingError::InvalidSealedEnvelope(err)
            ))
        })
    }

    /// constructor
    pub fn new(
        sender: Address,
        recipient: Address,
        nonce: box_::Nonce,
        msg: &[u8],
    ) -> SealedEnvelope {
        SealedEnvelope {
            sender,
            recipient,
            nonce,
            msg: EncryptedMessageBytes(msg.into()),
        }
    }

    /// open the envelope using the specified precomputed key
    pub fn open(self, key: &box_::PrecomputedKey) -> Result<OpenEnvelope, Error> {
        match box_::open_precomputed(&self.msg.0, &self.nonce, key) {
            Ok(msg) => Ok(OpenEnvelope {
                sender: self.sender,
                recipient: self.recipient,
                msg: MessageBytes(msg),
            }),
            Err(_) => Err(op_error!(errors::SealedEnvelopeOpenFailed(&self))),
        }
    }

    /// msg bytes
    pub fn msg(&self) -> &[u8] {
        &self.msg.0
    }

    /// returns the sender address
    pub fn sender(&self) -> &Address {
        &self.sender
    }

    /// returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// returns the nonce
    pub fn nonce(&self) -> &box_::Nonce {
        &self.nonce
    }
}

impl fmt::Display for SealedEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} -> {}, nonce: {}, msg.len: {}",
            self.sender,
            self.recipient,
            base58::encode(&self.nonce.0),
            self.msg.0.len()
        )
    }
}

/// Represents an envelope that is open, i.e., its message is not encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenEnvelope {
    sender: Address,
    recipient: Address,
    msg: MessageBytes,
}

impl OpenEnvelope {
    /// constructor
    pub fn new(sender: Address, recipient: Address, msg: &[u8]) -> OpenEnvelope {
        OpenEnvelope {
            sender,
            recipient,
            msg: MessageBytes(msg.into()),
        }
    }

    /// seals the envelope
    pub fn seal(self, key: &box_::PrecomputedKey) -> SealedEnvelope {
        let nonce = box_::gen_nonce();
        SealedEnvelope {
            sender: self.sender,
            recipient: self.recipient,
            nonce,
            msg: EncryptedMessageBytes(box_::seal_precomputed(&self.msg.0, &nonce, key)),
        }
    }

    /// msg bytes
    pub fn msg(&self) -> &[u8] {
        &self.msg.0
    }

    /// returns the sender address
    pub fn sender(&self) -> &Address {
        &self.sender
    }

    /// returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// parses the message data into an encoded message
    pub fn encoded_message(self) -> Result<EncodedMessage, Error> {
        let msg: Message = rmp_serde::from_slice(self.msg()).map_err(|err| {
            op_error!(errors::MessageError::MessageDataDeserializationFailed(
                &self.sender,
                errors::ErrorInfo(err.to_string())
            ))
        })?;
        Ok(EncodedMessage {
            sender: self.sender,
            recipient: self.recipient,
            msg,
        })
    }
}

impl fmt::Display for OpenEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} -> {}, msg.len: {}",
            self.sender,
            self.recipient,
            self.msg.0.len()
        )
    }
}

/// Addresses are identified by public-keys.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Address(box_::PublicKey);

impl Address {
    /// returns the underlying public-key
    pub fn public_key(&self) -> &box_::PublicKey {
        &self.0
    }

    /// precompute the key that can be used to seal the envelope by the sender
    pub fn precompute_sealing_key(
        &self,
        sender_private_key: &box_::SecretKey,
    ) -> box_::PrecomputedKey {
        box_::precompute(&self.0, sender_private_key)
    }

    /// precompute the key that can be used to open the envelope by the recipient
    pub fn precompute_opening_key(
        &self,
        recipient_private_key: &box_::SecretKey,
    ) -> box_::PrecomputedKey {
        box_::precompute(&self.0, recipient_private_key)
    }
}

impl From<box_::PublicKey> for Address {
    fn from(address: box_::PublicKey) -> Address {
        Address(address)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base58::encode(&(self.0).0))
    }
}

/// message data bytes that is encrypted
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EncryptedMessageBytes(Vec<u8>);

impl EncryptedMessageBytes {
    /// returns the message bytess
    pub fn data(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for EncryptedMessageBytes {
    fn from(bytes: &[u8]) -> EncryptedMessageBytes {
        EncryptedMessageBytes(Vec::from(bytes))
    }
}

impl From<Vec<u8>> for EncryptedMessageBytes {
    fn from(bytes: Vec<u8>) -> EncryptedMessageBytes {
        EncryptedMessageBytes(bytes)
    }
}

impl std::iter::FromIterator<u8> for EncryptedMessageBytes {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        EncryptedMessageBytes(Vec::from_iter(iter))
    }
}

/// message data bytes
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageBytes(Vec<u8>);

impl MessageBytes {
    /// returns the message bytess
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// hashes the message data
    pub fn hash(&self) -> hash::Digest {
        hash::hash(&self.0)
    }
}

impl From<&[u8]> for MessageBytes {
    fn from(bytes: &[u8]) -> MessageBytes {
        MessageBytes(Vec::from(bytes))
    }
}

impl From<Vec<u8>> for MessageBytes {
    fn from(bytes: Vec<u8>) -> MessageBytes {
        MessageBytes(bytes)
    }
}

impl std::iter::FromIterator<u8> for MessageBytes {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        MessageBytes(Vec::from_iter(iter))
    }
}

/// Message metadata
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Metadata {
    msg_type: MessageType,
    instance_id: InstanceId,
    encoding: Encoding,
    deadline: Option<Deadline>,
}

impl Metadata {
    /// constructor
    pub fn new(msg_type: MessageType, encoding: Encoding, deadline: Option<Deadline>) -> Metadata {
        Metadata {
            msg_type,
            instance_id: InstanceId::generate(),
            encoding,
            deadline,
        }
    }

    /// Each message type is identified by an Id
    pub fn message_type(&self) -> MessageType {
        self.msg_type
    }

    /// Each message instance is assigned a unique ULID. This can be used as a nonce for replay protection
    /// on the network.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// When the message was created. This is derived from the message instance ID.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.instance_id.ulid().datetime()
    }

    /// A message can specify that it must be processed by the specified deadline.
    pub fn deadline(&self) -> Option<Deadline> {
        self.deadline
    }

    /// return the message data encoding
    pub fn encoding(&self) -> Encoding {
        self.encoding
    }
}

/// Compression mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Compression {
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

/// Message encoding format
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Encoding {
    /// [MessagePack](https://msgpack.org/) - default
    MessagePack(Option<Compression>),
    /// [Bincode](https://github.com/TyOverby/bincode)
    Bincode(Option<Compression>),
    /// [CBOR](http://cbor.io/)
    CBOR(Option<Compression>),
    /// [JSON](https://www.json.org/)
    JSON(Option<Compression>),
}

/// Deadline
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Deadline {
    /// Max time allowed for the message to process
    ProcessingTimeoutMillis(u64),
    /// Message timeout is relative to the message timestamp
    MessageTimeoutMillis(u64),
}

op_ulid! {
    /// Unique message type identifier
    pub MessageTypeId
}

impl MessageTypeId {
    /// converts itself into a MessageType
    pub fn message_type(&self) -> MessageType {
        MessageType(self.ulid())
    }
}

/// Identifies the message type, whcih tells us how to decode the bytes message data.
///
/// # Why is there a MessageTypeId and MessageType - aren't they redundant ?
/// Underneath the covers, MessageTypeId is really a u128. MessagePack does not support u128.
/// Thus, for serializing messages, we use MessageType(ULID). MessageTypeId is used to define
/// constants.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MessageType(ULID);

impl From<MessageTypeId> for MessageType {
    fn from(type_id: MessageTypeId) -> MessageType {
        MessageType(type_id.ulid())
    }
}

impl MessageType {
    /// ULID getter
    pub fn ulid(&self) -> ULID {
        self.0
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Message instance unique identifier.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct InstanceId(ULID);

impl InstanceId {
    /// generates a new MessageInstance
    pub fn generate() -> InstanceId {
        InstanceId(ULID::generate())
    }

    /// ULID getter
    pub fn ulid(&self) -> ULID {
        self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Encoded message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    metadata: Metadata,
    data: MessageBytes,
}

impl Message {
    /// constructor
    pub fn new(metadata: Metadata, data: MessageBytes) -> Message {
        Message { metadata, data }
    }

    /// returns the message metadata
    pub fn metadata(&self) -> Metadata {
        self.metadata
    }

    /// returns the message data
    pub fn data(&self) -> &MessageBytes {
        &self.data
    }
}

/// Encoded message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedMessage {
    sender: Address,
    recipient: Address,
    msg: Message,
}

impl EncodedMessage {
    /// returns the message metadata
    pub fn metadata(&self) -> Metadata {
        self.msg.metadata
    }

    /// returns the message data
    pub fn data(&self) -> &MessageBytes {
        &self.msg.data
    }

    /// return the sender's address
    pub fn sender(&self) -> &Address {
        &self.sender
    }

    /// return the recipient's address
    pub fn recipient(&self) -> &Address {
        &self.sender
    }

    /// converts into an OpenEnvelope
    pub fn open_envelope(self) -> Result<OpenEnvelope, Error> {
        let msg = MessageBytes(rmp_serde::to_vec(&self.msg).map_err(|err| {
            op_error!(errors::MessageError::EncodedMessageSerializationFailed(
                self.sender(),
                errors::ErrorInfo(err.to_string())
            ))
        })?);
        Ok(OpenEnvelope {
            sender: self.sender,
            recipient: self.recipient,
            msg,
        })
    }
}

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

impl From<ULID> for SessionId {
    fn from(ulid: ULID) -> SessionId {
        SessionId(ulid)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Encrypted digitally signed hash
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EncryptedSignedHash(Vec<u8>, secretbox::Nonce);

impl EncryptedSignedHash {
    /// decrypts the signed hash and verifies the signature
    pub fn verify(
        &self,
        key: &secretbox::Key,
        public_key: &sign::PublicKey,
    ) -> Result<hash::Digest, Error> {
        match secretbox::open(&self.0, &self.1, key) {
            Ok(signed_hash) => match sign::verify(&signed_hash, public_key) {
                Ok(digest) => match hash::Digest::from_slice(&digest) {
                    Some(digest) => Ok(digest),
                    None => Err(op_error!(errors::MessageError::InvalidDigestLength {
                        from: public_key,
                        len: digest.len()
                    })),
                },
                Err(_) => Err(op_error!(errors::MessageError::InvalidSignature(
                    public_key
                ))),
            },
            Err(_) => Err(op_error!(errors::MessageError::DecryptionFailed(
                public_key
            ))),
        }
    }

    /// return the nonce used to encrypt this signed hash
    pub fn nonce(&self) -> &secretbox::Nonce {
        &self.1
    }
}

/// A digitally signed hash
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SignedHash(Vec<u8>);

impl SignedHash {
    /// constructor - signs the hash using the specified private-key
    pub fn sign(digest: &hash::Digest, key: &sign::SecretKey) -> SignedHash {
        SignedHash(sign::sign(&digest.0, key))
    }

    /// verifies the hash's signature against the specified PublicKey, and then verifies the message
    /// integrity by checking its hash
    pub fn verify(&self, msg: &[u8], key: &sign::PublicKey) -> Result<(), Error> {
        let digest = sign::verify(&self.0, key)
            .map_err(|_| op_error!(errors::MessageError::InvalidSignature(key)))?;
        match hash::Digest::from_slice(&digest) {
            Some(digest) => {
                let msg_digest = hash::hash(msg);
                if msg_digest == digest {
                    Ok(())
                } else {
                    Err(op_error!(errors::MessageError::ChecksumFailed(key)))
                }
            }
            None => Err(op_error!(errors::MessageError::InvalidDigestLength {
                from: key,
                len: digest.len()
            })),
        }
    }

    /// encrypt the signed hash
    pub fn encrypt(&self, key: &secretbox::Key) -> EncryptedSignedHash {
        let nonce = secretbox::gen_nonce();
        EncryptedSignedHash(secretbox::seal(&self.0, &nonce, key), nonce)
    }
}

impl From<&[u8]> for SignedHash {
    fn from(bytes: &[u8]) -> SignedHash {
        SignedHash(Vec::from(bytes))
    }
}

impl From<Vec<u8>> for SignedHash {
    fn from(bytes: Vec<u8>) -> SignedHash {
        SignedHash(bytes)
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {

    use super::{
        base58, Address, EncryptedMessageBytes, MessageBytes, MessageType, OpenEnvelope,
        SealedEnvelope,
    };
    use crate::tests::run_test;
    use exonum_sodiumoxide::crypto::{box_, hash, secretbox, sign};
    use oysterpack_uid::ULID;
    use std::io;

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

    #[test]
    fn seal_open_envelope() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let (client_addr, server_addr) =
            (Address::from(client_pub_key), Address::from(server_pub_key));
        let opening_key = client_addr.precompute_opening_key(&server_priv_key);
        let sealing_key = server_addr.precompute_sealing_key(&client_priv_key);
        let msg = b"data";

        run_test("seal_open_envelope", || {
            info!("addresses: {} -> {}", client_addr, server_addr);
            let open_envelope =
                OpenEnvelope::new(client_pub_key.into(), server_pub_key.into(), msg);
            let open_envelope_rmp = rmp_serde::to_vec(&open_envelope).unwrap();
            info!("open_envelope_rmp len = {}", open_envelope_rmp.len());
            let sealed_envelope = open_envelope.seal(&sealing_key);
            let sealed_envelope_rmp = rmp_serde::to_vec(&sealed_envelope).unwrap();
            info!("sealed_envelope_rmp len = {}", sealed_envelope_rmp.len());
            info!(
                "sealed_envelope json: {}",
                serde_json::to_string_pretty(&sealed_envelope).unwrap()
            );
            info!("sealed_envelope msg len: {}", sealed_envelope.msg().len());

            let open_envelope_2 = sealed_envelope.open(&opening_key).unwrap();
            info!(
                "open_envelope_2 json: {}",
                serde_json::to_string_pretty(&open_envelope_2).unwrap()
            );
            info!("open_envelope_2 msg len: {}", open_envelope_2.msg().len());
            assert_eq!(*open_envelope_2.msg(), *msg);
        });

        let msg = &[0 as u8; 1000 * 256];
        let msg = &msg[..];
        let open_envelope = OpenEnvelope::new(client_pub_key.into(), server_pub_key.into(), msg);
        run_test("seal_envelope", || {
            let _ = open_envelope.seal(&sealing_key);
        });

        let open_envelope = OpenEnvelope::new(client_pub_key.into(), server_pub_key.into(), msg);
        let sealed_envelope = open_envelope.seal(&sealing_key);
        run_test("open_envelope", || {
            let _ = sealed_envelope.open(&opening_key).unwrap();
        });
    }

    #[test]
    fn sealed_envelope_encoding_decoding() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let (client_addr, server_addr) =
            (Address::from(client_pub_key), Address::from(server_pub_key));
        let opening_key = client_addr.precompute_opening_key(&server_priv_key);
        let sealing_key = server_addr.precompute_sealing_key(&client_priv_key);

        run_test("sealed_envelope_encoding_decoding", || {
            info!("addresses: {} -> {}", client_addr, server_addr);
            let open_envelope =
                OpenEnvelope::new(client_pub_key.into(), server_pub_key.into(), b"");
            let mut sealed_envelope = open_envelope.seal(&sealing_key);

            let mut buf: io::Cursor<Vec<u8>> = io::Cursor::new(Vec::new());
            sealed_envelope.encode(&mut buf);
            info!(
                "SealedEnvelope[{}]: {:?} - {}",
                buf.get_ref().as_slice().len(),
                buf.get_ref().as_slice(),
                sealed_envelope.msg().len()
            );

            let sealed_envelope_decoded = SealedEnvelope::decode(buf.get_ref().as_slice()).unwrap();
            assert_eq!(sealed_envelope.sender(), sealed_envelope_decoded.sender());
            assert_eq!(
                sealed_envelope.recipient(),
                sealed_envelope_decoded.recipient()
            );

            sealed_envelope.msg = EncryptedMessageBytes(vec![1]);
            let mut buf: io::Cursor<Vec<u8>> = io::Cursor::new(Vec::new());
            sealed_envelope.encode(&mut buf);
            info!(
                "SealedEnvelope[{}]: {:?} - {}",
                buf.get_ref().as_slice().len(),
                buf.get_ref().as_slice(),
                sealed_envelope.msg().len()
            );
        });
    }

    #[test]
    fn base58_encoding_keys() {
        let (pub_key, priv_key) = box_::gen_keypair();

        let pub_key_base58 = base58::encode(&pub_key.0);
        let pub_key_bytes = base58::decode(&pub_key_base58).unwrap();
        let pub_key2 = box_::PublicKey::from_slice(&pub_key_bytes).unwrap();
        assert_eq!(pub_key, pub_key2);

        let key_base58 = base58::encode(&priv_key.0);
        let key_bytes = base58::decode(&key_base58).unwrap();
        let key2 = box_::SecretKey::from_slice(&key_bytes).unwrap();
        assert_eq!(priv_key, key2);
    }

    #[test]
    fn ulid_msg_pack_size() {
        op_ulid! {
            Foo
        }

        let ulid = oysterpack_uid::ULID::generate();
        let foo = Foo(ulid.into());
        let ulid_bytes = rmp_serde::to_vec(&ulid).unwrap();
        let foo_bytes = rmp_serde::to_vec(&foo).unwrap();
        println!(
            "foo_bytes.len = {}, ulid_bytes.len = {}",
            foo_bytes.len(),
            ulid_bytes.len()
        ); // foo_bytes.len = 19, ulid_bytes.len = 27
        assert!(
            foo_bytes.len() < ulid_bytes.len(),
            "in binary form, (u64,u64) should be smaller than a 27 char ULID"
        );
    }

    #[test]
    fn encrypted_signed_hash() {
        let (client_pub_key, client_priv_key) = sign::gen_keypair();
        let cipher = secretbox::gen_key();
        let session_id = super::SessionId::generate();

        let data = b"some data";
        let data_hash = hash::hash(data);
        let signed_hash_1 = super::SignedHash::sign(&data_hash, &client_priv_key);
        let encrypted_signed_hash_1 = signed_hash_1.encrypt(&cipher);
        let encrypted_signed_hash_2 = signed_hash_1.encrypt(&cipher);
        assert_ne!(
            encrypted_signed_hash_1.nonce(),
            encrypted_signed_hash_2.nonce(),
            "A new nonce should be used each time the signed session id is encrypted"
        );
        let digest_1 = encrypted_signed_hash_1
            .verify(&cipher, &client_pub_key)
            .unwrap();
        let digest_2 = encrypted_signed_hash_2
            .verify(&cipher, &client_pub_key)
            .unwrap();
        assert_eq!(digest_1, digest_2);
        assert_eq!(digest_1, data_hash);
    }

    #[test]
    fn encoded_message() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let (client_addr, server_addr) =
            (Address::from(client_pub_key), Address::from(server_pub_key));
        let opening_key = client_addr.precompute_opening_key(&server_priv_key);
        let sealing_key = server_addr.precompute_sealing_key(&client_priv_key);

        fn new_msg() -> super::Message {
            const MESSAGE_TYPE: super::MessageTypeId = super::MessageTypeId(1867384532653698871582487715619812439);
            let metadata = super::Metadata::new(
                MESSAGE_TYPE.message_type(),
                super::Encoding::MessagePack(None),
                None,
            );
            let data = super::MessageBytes::from(b"data".as_ref());
            super::Message {
                metadata,
                data
            }
        }
        let msg = new_msg();
        let msg_bytes = rmp_serde::to_vec(&msg).unwrap();
        if let Err(err) = rmp_serde::from_slice::<super::Message>(&msg_bytes) {
            panic!("Failed to deserialize Message: {}", err);
        }

        run_test("sealed_envelope_encoding_decoding", || {
            info!("addresses: {} -> {}", client_addr, server_addr);
            let open_envelope =
                OpenEnvelope::new(client_pub_key.into(), server_pub_key.into(), &rmp_serde::to_vec(&msg).unwrap());
            let encoded_message = open_envelope.clone().encoded_message().unwrap();
            let open_envelope_2 = encoded_message.open_envelope().unwrap();
            assert_eq!(open_envelope.sender(), open_envelope_2.sender());
            assert_eq!(open_envelope.recipient(), open_envelope_2.recipient());
            assert_eq!(open_envelope.msg(), open_envelope_2.msg());
        });
    }

    #[test]
    #[ignore]
    fn rmp_serde_deserialize_fails_when_skipping_none_option() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Foo {
            #[serde(skip_serializing_if = "Option::is_none")]
            bar: Option<bool>
        }

        let foo = Foo { bar: None } ;
        let foo_bytes = rmp_serde::to_vec(&foo).unwrap();
        // deserializing panics because rmp is expecting 1 element
        // this is a bug in rmp_serde: https://github.com/3Hren/msgpack-rust/issues/86
        let foo : Foo = rmp_serde::from_slice(&foo_bytes).unwrap();
    }

}
