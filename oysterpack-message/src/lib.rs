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

//! OysterPack Message

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_message/0.1.0")]

use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use std::{fmt, str::FromStr};

/// Addresses are identified by public-keys.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Address(box_::PublicKey);

impl Address {
    /// returns the underlying public-key
    pub fn public_key(&self) -> &box_::PublicKey {
        &self.0
    }

    /// computes an intermediate key that can be used to encrypt / decrypt data
    pub fn precompute_key(&self, secret_key: &box_::SecretKey) -> box_::PrecomputedKey {
        box_::precompute(&self.0, secret_key)
    }
}

impl From<box_::PublicKey> for Address {
    fn from(address: box_::PublicKey) -> Address {
        Address(address)
    }
}

impl fmt::Display for Address {
    /// encodes the address using a [Base58](https://en.wikipedia.org/wiki/Base58) encoding - which is used by Bitcoin
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(&(self.0).0).into_string())
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Address, Self::Err> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|err| op_error!(errors::Base58DecodeError(ErrorMessage(err.to_string()))))?;
        match box_::PublicKey::from_slice(&bytes) {
            Some(key) => Ok(Address(key)),
            None => Err(op_error!(errors::InvalidPublicKeyLength(bytes.len()))),
        }
    }
}

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
            msg: EncryptedMessageBytes::from(msg),
        }
    }

    // TODO: implement TryFrom when it bocomes stable
    /// Converts an nng:Message into a SealedEnvelope.
    pub fn try_from_nng_message(msg: nng::Message) -> Result<SealedEnvelope, Error> {
        bincode::deserialize(&**msg).map_err(|err| {
            op_error!(errors::BincodeDeserializeError(errors::Scope::SealedEnvelope, ErrorMessage(err.to_string())))
        })
    }

    // TODO: implement TryInto when it becomes stable
    /// Converts itself into an nng:Message
    pub fn try_into_nng_message(self) -> Result<nng::Message, Error> {
        let bytes = bincode::serialize(&self)
            .map_err(|err| op_error!(errors::BincodeSerializeError(errors::Scope::SealedEnvelope, ErrorMessage(err.to_string()))))?;
        let mut msg = nng::Message::with_capacity(bytes.len()).map_err(|err| {
            op_error!(errors::NngMessageError::from(ErrorMessage(format!("Failed to create an empty message with a pre-allocated body buffer (capacity = {}): {}", bytes.len(), err))))
        })?;
        msg.push_back(&bytes).map_err(|err| {
            op_error!(errors::NngMessageError::from(ErrorMessage(format!(
                "Failed to append data to the back of the message body: {}",
                err
            ))))
        })?;
        Ok(msg)
    }

    /// open the envelope using the specified precomputed key
    pub fn open(self, key: &box_::PrecomputedKey) -> Result<OpenEnvelope, Error> {
        match box_::open_precomputed(&self.msg.0, &self.nonce, key) {
            Ok(msg) => Ok(OpenEnvelope {
                sender: self.sender,
                recipient: self.recipient,
                msg: MessageBytes(msg),
            }),
            Err(_) => Err(op_error!(errors::DecryptionError(errors::Scope::SealedEnvelope))),
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
}

impl fmt::Display for SealedEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} -> {}, nonce: {}, msg.len: {}",
            self.sender,
            self.recipient,
            bs58::encode(&self.nonce.0).into_string(),
            self.msg.0.len()
        )
    }
}

/// message data bytes that is encrypted
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct EncryptedMessageBytes(Vec<u8>);

impl EncryptedMessageBytes {
    /// returns the message bytess
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// decrypt the message
    pub fn decrypt(
        &self,
        nonce: &box_::Nonce,
        key: &box_::PrecomputedKey,
    ) -> Result<MessageBytes, Error> {
        box_::open_precomputed(&self.0, nonce, key)
            .map(|data| MessageBytes(data))
            .map_err(|_| op_error!(errors::DecryptionError(errors::Scope::EncryptedMessageBytes)))
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

/// message data bytes
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct MessageBytes(Vec<u8>);

impl MessageBytes {
    /// returns the message bytess
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// encrypts the message
    pub fn encrypt(
        &self,
        nonce: &box_::Nonce,
        key: &box_::PrecomputedKey,
    ) -> EncryptedMessageBytes {
        EncryptedMessageBytes(box_::seal_precomputed(&self.0, nonce, key))
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
            msg: MessageBytes::from(msg),
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

/// message related errors
pub mod errors {
    use std::fmt;
    use oysterpack_errors::{ErrorMessage, IsError};
    use serde::{Deserialize, Serialize};

    /// Base58 decoding error
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Base58DecodeError(pub(crate) ErrorMessage);

    impl Base58DecodeError {
        /// Error ID
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869558836149169496880583090618468282);

        /// Error level
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for Base58DecodeError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for Base58DecodeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "Invalid Base58 encoding using the Bitcoin alphabet: {}",
                self.0
            )
        }
    }

    /// PublicKey should be 32 bytes
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct InvalidPublicKeyLength(pub(crate) usize);

    impl InvalidPublicKeyLength {
        /// Error ID
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869558894929538460990404972159560814);

        /// Error level
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for InvalidPublicKeyLength {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for InvalidPublicKeyLength {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "PublicKey should be 32 bytes, but was {}", self.0)
        }
    }

    /// Base58 decoding error
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct DecryptionError(pub(crate) Scope);

    impl DecryptionError {
        /// Error ID
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869570295266385307080584268554182611);

        /// Error level
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for DecryptionError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for DecryptionError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "failed to decrypt: {:?}", self.0)
        }
    }


    #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
    pub(crate) enum Scope {
        EncryptedMessageBytes,
        SealedEnvelope
    }

    /// nng:Message related error
    #[derive(Debug)]
    pub struct NngMessageError(pub(crate) ErrorMessage);

    impl NngMessageError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869218326628258606664054868854559775);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl From<ErrorMessage> for NngMessageError {
        fn from(err_msg: ErrorMessage) -> NngMessageError {
            NngMessageError(err_msg)
        }
    }

    impl IsError for NngMessageError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for NngMessageError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to allocate a new nng:Message: {}", self.0)
        }
    }

    /// bincode serialization related error
    #[derive(Debug)]
    pub struct BincodeSerializeError(pub(crate) Scope,pub(crate) ErrorMessage);

    impl BincodeSerializeError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869574419254846020884390106309931899);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl IsError for BincodeSerializeError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for BincodeSerializeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "`{:?}` bincode deserialization failed: {}", self.0, self.1)
        }
    }

    /// bincode deserialization related error
    #[derive(Debug)]
    pub struct BincodeDeserializeError(pub(crate) Scope,pub(crate) ErrorMessage);

    impl BincodeDeserializeError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1869576546482110294116245028055198653);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl IsError for BincodeDeserializeError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for BincodeDeserializeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "`{:?}` bincode serialization failed: {}", self.0, self.1)
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_precompute_key() {
        sodiumoxide::init().unwrap();
        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        let data: &[u8] = b"cryptocurrency is the future";
        let key = server_addr.precompute_key(&client_private_key);
        let nonce = sodiumoxide::crypto::box_::gen_nonce();
        let data_encrypted = sodiumoxide::crypto::box_::seal_precomputed(data, &nonce, &key);
        let key = client_addr.precompute_key(&server_private_key);
        let data_2 =
            sodiumoxide::crypto::box_::open_precomputed(&data_encrypted, &nonce, &key).unwrap();
        assert_eq!(data, data_2.as_slice());
    }

    #[test]
    fn address_base58_encoding() {
        sodiumoxide::init().unwrap();
        let (client_public_key, _) = sodiumoxide::crypto::box_::gen_keypair();
        let client_addr = Address::from(client_public_key);
        let mut s = client_addr.to_string();
        let client_addr_2: Address = s.parse().unwrap();
        assert_eq!(client_addr, client_addr_2);

        s.push('0');
        match s.parse::<Address>() {
            Ok(_) => {
                panic!("should have failed to parse because '0' is not in the Bitcoin alphabet")
            }
            Err(err) => {
                println!("{}", err);
                assert_eq!(err.id(), errors::Base58DecodeError::ERROR_ID)
            }
        }

        let mut s = client_addr.to_string();
        s.push('2');
        match s.parse::<Address>() {
            Ok(_) => panic!("should have failed to parse because the number of bytes is 33"),
            Err(err) => {
                println!("{}", err);
                assert_eq!(err.id(), errors::InvalidPublicKeyLength::ERROR_ID);
            }
        }
    }

    #[test]
    fn message_bytes() {
        let data: &[u8] = b"cryptocurrency is the future";
        let msg = MessageBytes::from(data);
        let msg_2 = MessageBytes::from(Vec::from(data));
        assert_eq!(msg, msg_2);
        assert_eq!(msg.data(), data);
    }

    #[test]
    fn message_bytes_encrypt_decrypt() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";
        let msg = MessageBytes::from(data);

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        let nonce = box_::gen_nonce();
        let encrypted_msg = msg.encrypt(&nonce, &server_addr.precompute_key(&client_private_key));

        let msg_2 = encrypted_msg
            .decrypt(&nonce, &client_addr.precompute_key(&server_private_key))
            .unwrap();
        assert_eq!(msg, msg_2);
    }

    #[test]
    fn open_envelope_sealed_envelope() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        // create a new OpenEnvelope
        let open_envelope = OpenEnvelope::new(client_addr,server_addr,data);
        assert_eq!(data, open_envelope.msg());
        assert_eq!(client_addr, *open_envelope.sender());
        assert_eq!(server_addr, *open_envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = open_envelope.clone().seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());
        // open the SealedEnvelope
        let open_envelope_2 = sealed_envelope.open(&client_addr.precompute_key(&server_private_key)).unwrap();
        assert_eq!(open_envelope_2.sender(),open_envelope.sender());
        assert_eq!(open_envelope_2.recipient(),open_envelope.recipient());
        assert_eq!(open_envelope_2.msg(),open_envelope.msg());
    }

    #[test]
    fn opening_sealed_envelope_using_different_server_secret_key() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        // create a new OpenEnvelope
        let open_envelope = OpenEnvelope::new(client_addr,server_addr,data);
        assert_eq!(data, open_envelope.msg());
        assert_eq!(client_addr, *open_envelope.sender());
        assert_eq!(server_addr, *open_envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = open_envelope.clone().seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());

        // generate new server keypair
        let (_, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        // open the SealedEnvelope
        match sealed_envelope.open(&client_addr.precompute_key(&server_private_key)) {
            Ok(_) => panic!("decryption should have failed"),
            Err(err) => {
                println!("Decryption error: {}", err);
                assert_eq!(err.id(), errors::DecryptionError::ERROR_ID);
            }
        }
    }

    #[test]
    fn opening_sealed_envelope_using_different_client_secret_key() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        // generate new client keypair
        let (_, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        // create a new OpenEnvelope
        let open_envelope = OpenEnvelope::new(client_addr,server_addr,data);
        assert_eq!(data, open_envelope.msg());
        assert_eq!(client_addr, *open_envelope.sender());
        assert_eq!(server_addr, *open_envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = open_envelope.clone().seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());

        // open the SealedEnvelope
        match sealed_envelope.open(&client_addr.precompute_key(&server_private_key)) {
            Ok(_) => panic!("decryption should have failed"),
            Err(err) => {
                println!("Decryption error: {}", err);
                assert_eq!(err.id(), errors::DecryptionError::ERROR_ID);
            }
        }
    }

    #[test]
    fn opening_sealed_envelope_with_truncated_msg() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        // create a new OpenEnvelope
        let open_envelope = OpenEnvelope::new(client_addr,server_addr,data);
        assert_eq!(data, open_envelope.msg());
        assert_eq!(client_addr, *open_envelope.sender());
        assert_eq!(server_addr, *open_envelope.recipient());
        // seal the OpenEnvelope
        let mut sealed_envelope = open_envelope.clone().seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());
        let mut encrypted_msg = sealed_envelope.msg.0.clone();
        encrypted_msg.truncate(data.len()/2);
        sealed_envelope.msg = EncryptedMessageBytes(encrypted_msg);

        // open the SealedEnvelope
        match sealed_envelope.open(&client_addr.precompute_key(&server_private_key)) {
            Ok(_) => panic!("decryption should have failed"),
            Err(err) => {
                println!("Decryption error: {}", err);
                assert_eq!(err.id(), errors::DecryptionError::ERROR_ID);
            }
        }
    }
}
