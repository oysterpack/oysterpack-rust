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

//! defines the message envelope layer

use crate::errors;
use crate::marshal;
use crate::security::Address;
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use std::fmt;

/// A sealed envelope is secured via public-key authenticated encryption. It contains a private message
/// that is encrypted using the recipient's public-key and the sender's private-key. If the recipient
/// is able to decrypt the message, then the recipient knows it was sealed by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedEnvelope {
    sender: Address,
    recipient: Address,
    nonce: box_::Nonce,
    msg: EncryptedBytesMessage,
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
            msg: EncryptedBytesMessage::from(msg),
        }
    }

    // TODO: implement TryFrom when it bocomes stable
    /// Converts an nng:Message into a SealedEnvelope.
    pub fn try_from_nng_message(msg: &nng::Message) -> Result<SealedEnvelope, Error> {
        bincode::deserialize(&**msg).map_err(|err| {
            op_error!(errors::BincodeDeserializeError(ErrorMessage(
                err.to_string()
            )))
        })
    }

    // TODO: implement TryInto when it becomes stable
    /// Converts itself into an nng:Message
    pub fn try_into_nng_message(self) -> Result<nng::Message, Error> {
        let bytes = bincode::serialize(&self).map_err(|err| {
            op_error!(errors::BincodeSerializeError(ErrorMessage(format!(
                "SealedEnvelope : {}",
                err
            ))))
        })?;
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
    pub fn open(self, key: &box_::PrecomputedKey) -> Result<Envelope<BytesMessage>, Error> {
        match box_::open_precomputed(&self.msg.0, &self.nonce, key) {
            Ok(msg) => Ok(Envelope {
                sender: self.sender,
                recipient: self.recipient,
                msg: BytesMessage(msg),
            }),
            Err(_) => Err(op_error!(errors::DecryptionError(ErrorMessage::from(
                "SealedEnvelope"
            )))),
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

/// encrypted raw bytes message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct EncryptedBytesMessage(Vec<u8>);

impl EncryptedBytesMessage {
    /// returns the message bytess
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// decrypt the message
    pub fn decrypt(
        &self,
        nonce: &box_::Nonce,
        key: &box_::PrecomputedKey,
    ) -> Result<BytesMessage, Error> {
        box_::open_precomputed(&self.0, nonce, key)
            .map(BytesMessage)
            .map_err(|_| {
                op_error!(errors::DecryptionError(ErrorMessage::from(
                    "EncryptedBytesMessage"
                )))
            })
    }
}

impl From<&[u8]> for EncryptedBytesMessage {
    fn from(bytes: &[u8]) -> EncryptedBytesMessage {
        EncryptedBytesMessage(Vec::from(bytes))
    }
}

impl From<Vec<u8>> for EncryptedBytesMessage {
    fn from(bytes: Vec<u8>) -> EncryptedBytesMessage {
        EncryptedBytesMessage(bytes)
    }
}

/// raw bytes message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct BytesMessage(pub Vec<u8>);

impl BytesMessage {
    /// serializes the message into a BytesMessage using bincode
    pub fn serialize<T: Serialize>(msg: &T) -> Result<BytesMessage, Error> {
        marshal::serialize(msg).map(BytesMessage)
    }

    /// returns the message bytes
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// encrypts the message
    pub fn encrypt(
        &self,
        nonce: &box_::Nonce,
        key: &box_::PrecomputedKey,
    ) -> EncryptedBytesMessage {
        EncryptedBytesMessage(box_::seal_precomputed(&self.0, nonce, key))
    }
}

impl From<&[u8]> for BytesMessage {
    fn from(bytes: &[u8]) -> BytesMessage {
        BytesMessage(Vec::from(bytes))
    }
}

impl From<Vec<u8>> for BytesMessage {
    fn from(bytes: Vec<u8>) -> BytesMessage {
        BytesMessage(bytes)
    }
}

/// A message envelope that is addressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T: Send> {
    sender: Address,
    recipient: Address,
    msg: T,
}

impl<T: Send + Serialize> Envelope<T> {
    /// constructor
    pub fn new(sender: Address, recipient: Address, msg: T) -> Envelope<T> {
        Envelope {
            sender,
            recipient,
            msg,
        }
    }

    /// msg bytes
    pub fn msg(&self) -> &T {
        &self.msg
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

impl<T: Send + Serialize> Envelope<T> {
    /// serializes the message into `Envelope<BytesMessage>`
    pub fn try_into_bytes_message(self) -> Result<Envelope<BytesMessage>, Error> {
        let msg = BytesMessage::serialize(&self.msg)?;
        let envelope = Envelope {
            sender: self.sender,
            recipient: self.recipient,
            msg,
        };
        Ok(envelope)
    }
}

impl Envelope<BytesMessage> {
    /// constructor
    pub fn bytes_message(
        sender: Address,
        recipient: Address,
        msg: &[u8],
    ) -> Envelope<BytesMessage> {
        Envelope {
            sender,
            recipient,
            msg: BytesMessage::from(msg),
        }
    }

    /// seals the envelope
    pub fn seal(self, key: &box_::PrecomputedKey) -> SealedEnvelope {
        let nonce = box_::gen_nonce();
        SealedEnvelope {
            sender: self.sender,
            recipient: self.recipient,
            nonce,
            msg: EncryptedBytesMessage(box_::seal_precomputed(&self.msg.0, &nonce, key)),
        }
    }

    /// deserializes the BytesMessage into `T`
    pub fn deserialize<T: Send + DeserializeOwned>(self) -> Result<Envelope<T>, Error> {
        let msg: T = marshal::deserialize(self.msg.data())?;
        Ok(Envelope {
            sender: self.sender,
            recipient: self.recipient,
            msg,
        })
    }
}

impl fmt::Display for Envelope<BytesMessage> {
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

        let s = client_addr.to_string();
        let s = format!("222222{}333333", s);
        match s.parse::<Address>() {
            Ok(_) => panic!(
                "should have failed to parse because the number of bytes should be 32: {}",
                s
            ),
            Err(err) => {
                println!("{}", err);
                assert_eq!(err.id(), errors::InvalidPublicKeyLength::ERROR_ID);
            }
        }
    }

    #[test]
    fn message_bytes() {
        let data: &[u8] = b"cryptocurrency is the future";
        let msg = BytesMessage::from(data);
        let msg_2 = BytesMessage::from(Vec::from(data));
        assert_eq!(msg, msg_2);
        assert_eq!(msg.data(), data);
    }

    #[test]
    fn message_bytes_encrypt_decrypt() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";
        let msg = BytesMessage::from(data);

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
    fn open_sealed_envelope() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        // create a new OpenEnvelope
        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        assert_eq!(data, envelope.msg().data());
        assert_eq!(client_addr, *envelope.sender());
        assert_eq!(server_addr, *envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = envelope
            .clone()
            .seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());
        // open the SealedEnvelope
        let envelope_2 = sealed_envelope
            .open(&client_addr.precompute_key(&server_private_key))
            .unwrap();
        assert_eq!(envelope_2.sender(), envelope.sender());
        assert_eq!(envelope_2.recipient(), envelope.recipient());
        assert_eq!(envelope_2.msg(), envelope.msg());
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
        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        assert_eq!(data, envelope.msg().data());
        assert_eq!(client_addr, *envelope.sender());
        assert_eq!(server_addr, *envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = envelope
            .clone()
            .seal(&server_addr.precompute_key(&client_private_key));
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
        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        assert_eq!(data, envelope.msg().data());
        assert_eq!(client_addr, *envelope.sender());
        assert_eq!(server_addr, *envelope.recipient());
        // seal the OpenEnvelope
        let sealed_envelope = envelope
            .clone()
            .seal(&server_addr.precompute_key(&client_private_key));
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
        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        assert_eq!(data, envelope.msg().data());
        assert_eq!(client_addr, *envelope.sender());
        assert_eq!(server_addr, *envelope.recipient());
        // seal the OpenEnvelope
        let mut sealed_envelope = envelope
            .clone()
            .seal(&server_addr.precompute_key(&client_private_key));
        assert_eq!(client_addr, *sealed_envelope.sender());
        assert_eq!(server_addr, *sealed_envelope.recipient());
        let mut encrypted_msg = sealed_envelope.msg.0.clone();
        encrypted_msg.truncate(data.len() / 2);
        sealed_envelope.msg = EncryptedBytesMessage(encrypted_msg);

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
    fn sealed_envelope_nng_conversions() {
        sodiumoxide::init().unwrap();
        let data: &[u8] = b"cryptocurrency is the future";

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        let sealed_envelope = envelope
            .clone()
            .seal(&server_addr.precompute_key(&client_private_key));
        let nng_msg = sealed_envelope.clone().try_into_nng_message().unwrap();
        let sealed_envelope_2 = SealedEnvelope::try_from_nng_message(&nng_msg).unwrap();
        let envelope_2 = sealed_envelope_2
            .open(&client_addr.precompute_key(&server_private_key))
            .unwrap();
        assert_eq!(envelope_2.sender(), envelope.sender());
        assert_eq!(envelope_2.recipient(), envelope.recipient());
        assert_eq!(envelope_2.msg(), envelope.msg());
    }

    #[test]
    fn envelope_try_into_bytes_message() {
        #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
        struct Foo(String);

        sodiumoxide::init().unwrap();
        let foo = Foo("cryptocurrency is the future".to_string());
        let data: &[u8] = &bincode::serialize(&foo).unwrap();

        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let server_addr = Address::from(server_public_key);

        let envelope = Envelope::bytes_message(client_addr, server_addr, data);
        let envelope_foo: Envelope<Foo> = envelope.deserialize().unwrap();
        println!("envelope_foo: {:?}", envelope_foo);
        assert_eq!(*envelope_foo.msg(), foo);
        let envelope = envelope_foo.try_into_bytes_message().unwrap();
        let envelope_foo: Envelope<Foo> = envelope.deserialize().unwrap();
        assert_eq!(*envelope_foo.msg(), foo);
    }
}
