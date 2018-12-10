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
//! Messages are processed as streams, transitioning states while being processed.
//!
//! - when a peer connects, the initial message is the handshake.
//!   - each peer is identified by a public-key
//!   - the connecting peer plays the role of `client`; the peer being connected to plays the role of
//!     `server`
//!   - the client initiates a connection with a server by encrypting a `Connect` message using the
//!     server's public-key. Thus, only a specific server can decrypt the message.
//!   - the connect message is hashed (SHA-512)
//!     - this enables the server to check that the message was not altered
//!   - the hash is digitally signed by the client using its private-key
//!     - this enables the server to check that the client owns the private-key corresponding to the
//!       client's public key
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

use bytes::{BufMut, BytesMut};
use tokio::codec::{Decoder, Encoder};
use tokio::prelude::*;

pub mod base58;
pub mod errors;

/// Max message size - 256 KB
pub const MAX_MSG_SIZE: usize = 1000 * 256;

/// Min message size for SealedEnvelope using MessagePack encoding
pub const SEALED_ENVELOPE_MIN_SIZE: usize = 90;

/// A sealed envelope contains a private message that was encrypted using the recipient's public-key
/// and the sender's private-key. If the recipient is able to decrypt the message, then the recipient
/// knows it was sealed by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedEnvelope {
    addresses: Addresses,
    nonce: box_::Nonce,
    msg: Vec<u8>,
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
    pub fn new(addresses: Addresses, nonce: box_::Nonce, msg: &[u8]) -> SealedEnvelope {
        SealedEnvelope {
            addresses,
            nonce,
            msg: msg.into(),
        }
    }

    /// open the envelope using the specified precomputed key
    pub fn open(self, key: &box_::PrecomputedKey) -> Result<OpenEnvelope, Error> {
        box_::open_precomputed(&self.msg, &self.nonce, key)
            .map(|msg| OpenEnvelope {
                addresses: self.addresses.clone(),
                msg,
            })
            .map_err(|_| op_error!(errors::SealedEnvelopeOpenFailed(&self)))
    }

    /// msg bytes
    pub fn msg(&self) -> &[u8] {
        &self.msg
    }

    /// returns the addresses
    pub fn addresses(&self) -> &Addresses {
        &self.addresses
    }

    /// returns the nonce
    pub fn nonce(&self) -> &box_::Nonce {
        &self.nonce
    }
}

/// Represents an envelope that is open, i.e., its message is not encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenEnvelope {
    addresses: Addresses,
    msg: Vec<u8>,
}

impl OpenEnvelope {
    /// constructor
    pub fn new(addresses: Addresses, msg: &[u8]) -> OpenEnvelope {
        OpenEnvelope {
            addresses,
            msg: msg.into(),
        }
    }

    /// seals the envelope
    pub fn seal(self, key: &box_::PrecomputedKey) -> SealedEnvelope {
        let nonce = box_::gen_nonce();
        SealedEnvelope {
            addresses: self.addresses,
            nonce,
            msg: box_::seal_precomputed(&self.msg, &nonce, key),
        }
    }

    /// msg bytes
    pub fn msg(&self) -> &[u8] {
        &self.msg
    }
}

/// Represents an envelope that is open, i.e., its message is not encrypted
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Addresses {
    sender: box_::PublicKey,
    recipient: box_::PublicKey,
}

impl Addresses {
    /// constructor
    pub fn new(sender: box_::PublicKey, recipient: box_::PublicKey) -> Addresses {
        Addresses { sender, recipient }
    }

    /// returns sender's public-key
    pub fn sender_public_key(&self) -> &box_::PublicKey {
        &self.sender
    }

    /// returns recipient's public-key
    pub fn recipient_public_key(&self) -> &box_::PublicKey {
        &self.recipient
    }

    /// precompute the key that can be used to seal the envelope by the sender
    pub fn precompute_sealing_key(
        &self,
        sender_private_key: &box_::SecretKey,
    ) -> box_::PrecomputedKey {
        box_::precompute(&self.recipient, sender_private_key)
    }

    /// precompute the key that can be used to open the envelope by the recipient
    pub fn precompute_opening_key(
        &self,
        recipient_private_key: &box_::SecretKey,
    ) -> box_::PrecomputedKey {
        box_::precompute(&self.sender, recipient_private_key)
    }
}

impl fmt::Display for Addresses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] -> [{}]",
            base58::encode(&self.sender.0),
            base58::encode(&self.recipient.0)
        )
    }
}

/// Message header metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headers {
    msg_type: MessageType,
}

op_ulid! {
    /// Unique message type identifier
    pub MessageType
}

// TODO: track connection timeouts, i.e., if receiving or sending messages takes too long.
/// SealedEnvelope codec
#[derive(Debug)]
pub struct SealedEnvelopeCodec {
    max_msg_size: usize,
    min_msg_size: usize
}

impl Default for SealedEnvelopeCodec {
    fn default() -> SealedEnvelopeCodec {
        SealedEnvelopeCodec {
            max_msg_size: MAX_MSG_SIZE,
            min_msg_size: SEALED_ENVELOPE_MIN_SIZE
        }
    }
}

impl SealedEnvelopeCodec {
    /// constructor
    pub fn new(max_msg_size: usize) -> SealedEnvelopeCodec {
        SealedEnvelopeCodec { max_msg_size, min_msg_size: SEALED_ENVELOPE_MIN_SIZE }
    }
}

impl Encoder for SealedEnvelopeCodec {
    type Item = SealedEnvelope;

    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buf = Vec::with_capacity(item.msg().len() + 256);
        let _ = item
            .encode(&mut buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string().as_str()))?;
        if buf.len() > self.max_msg_size {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "max message size exceeded: {} > {}",
                    buf.len(),
                    self.max_msg_size
                ),
            ));
        }
        dst.extend_from_slice(&buf);
        Ok(())
    }
}

impl Decoder for SealedEnvelopeCodec {
    type Item = SealedEnvelope;

    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let read_to = cmp::min(self.max_msg_size, buf.len());
        if read_to < self.min_msg_size {
            // message is to small - wait for more bytes
            return Ok(None);
        }

        let mut cursor = io::Cursor::new(&buf[..read_to]);
        match SealedEnvelope::decode(&mut cursor) {
            Ok(sealed_envelope) => {
                // drop the bytes that have been decoded
                let _ = buf.split_to(cursor.position() as usize);
                Ok(Some(sealed_envelope))
            },
            Err(err) => {
                if read_to == self.max_msg_size {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "message failed to be decoded within max message size limit: {} : {}",
                            self.max_msg_size,
                            err
                        ),
                    ))
                } else {
                    // we'll try again when more bytes come in
                    Ok(None)
                }
            }
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {

    use super::{base58, Addresses, MessageType, OpenEnvelope, SealedEnvelope, SealedEnvelopeCodec};
    use crate::tests::run_test;
    use exonum_sodiumoxide::crypto::box_;
    use oysterpack_uid::ULID;
    use std::io;
    use tokio::codec::{Decoder, Encoder};

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
    fn secure_envelope() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let addresses = Addresses::new(client_pub_key, server_pub_key);
        let opening_key = addresses.precompute_opening_key(&server_priv_key);
        let sealing_key = addresses.precompute_sealing_key(&client_priv_key);
        let msg = b"some data";
        const FOO: MessageType = MessageType(1866963020838464595588390333368926107);

        run_test("secure_envelope", || {
            info!("addresses: {}", addresses);
            let open_envelope = OpenEnvelope::new(addresses, msg);
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
    }

    #[test]
    fn sealed_envelope_encoding_decoding() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let addresses = Addresses::new(client_pub_key, server_pub_key);
        let opening_key = addresses.precompute_opening_key(&server_priv_key);
        let sealing_key = addresses.precompute_sealing_key(&client_priv_key);

        run_test("sealed_envelope_encoding_decoding", || {
            info!("addresses: {}", addresses);
            let open_envelope = OpenEnvelope::new(addresses.clone(), b"");
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
            assert_eq!(
                sealed_envelope.addresses(),
                sealed_envelope_decoded.addresses()
            );

            sealed_envelope.msg = vec![1];
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
    fn sealed_envelope_codec() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let addresses = Addresses::new(client_pub_key, server_pub_key);
        let opening_key = addresses.precompute_opening_key(&server_priv_key);
        let sealing_key = addresses.precompute_sealing_key(&client_priv_key);

        run_test("sealed_envelope_codec", || {
            let mut codec: SealedEnvelopeCodec = Default::default();
            let mut buf = bytes::BytesMut::new();
            for i in 0..5 {
                let open_envelope = OpenEnvelope::new(addresses.clone(), &vec![i as u8]);
                let mut sealed_envelope = open_envelope.seal(&sealing_key);
                codec.encode(sealed_envelope, &mut buf);
            }

            for i in 0..5 {
                info!("{:?}", codec.decode(&mut buf).unwrap().unwrap())
            }

            // buf is empty
            assert!(codec.decode(&mut buf).unwrap().is_none());

            let open_envelope = OpenEnvelope::new(addresses.clone(), &vec![1]);
            let mut sealed_envelope = open_envelope.seal(&sealing_key);

            let bytes = rmp_serde::to_vec(&sealed_envelope).unwrap();
            let (left, right) = bytes.split_at(super::SEALED_ENVELOPE_MIN_SIZE);
            buf.extend_from_slice(left);
            assert!(codec.decode(&mut buf).unwrap().is_none());
            buf.extend_from_slice(right);
            assert!(codec.decode(&mut buf).unwrap().is_some());
        });

    }
}
