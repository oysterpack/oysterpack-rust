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

//! message codecs

use crate::message::{self, Address, SealedEnvelope};
use bytes::{BufMut, BytesMut};
use std::{cmp, io};
use tokio::codec::{Decoder, Encoder};
use tokio::prelude::*;

// TODO: track connection timeouts, i.e., if receiving or sending messages takes too long.
/// SealedEnvelope codec
#[derive(Debug)]
pub struct SealedEnvelopeCodec {
    max_msg_size: usize,
    min_msg_size: usize,
}

impl Default for SealedEnvelopeCodec {
    fn default() -> SealedEnvelopeCodec {
        SealedEnvelopeCodec {
            max_msg_size: message::MAX_MSG_SIZE,
            min_msg_size: message::SEALED_ENVELOPE_MIN_SIZE,
        }
    }
}

impl SealedEnvelopeCodec {
    /// constructor
    pub fn new(max_msg_size: usize) -> SealedEnvelopeCodec {
        SealedEnvelopeCodec {
            max_msg_size,
            min_msg_size: message::SEALED_ENVELOPE_MIN_SIZE,
        }
    }
}

impl Encoder for SealedEnvelopeCodec {
    type Item = SealedEnvelope;

    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buf = Vec::with_capacity(item.msg().len() + 256);
        item.encode(&mut buf)
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
            }
            Err(err) => {
                if read_to == self.max_msg_size {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "message failed to be decoded within max message size limit: {} : {}",
                            self.max_msg_size, err
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
mod tests {
    use super::*;
    use crate::message::*;
    use crate::tests::run_test;
    use tokio::codec::{Decoder, Encoder};

    #[test]
    fn sealed_envelope_codec() {
        let (client_pub_key, client_priv_key) = box_::gen_keypair();
        let (server_pub_key, server_priv_key) = box_::gen_keypair();

        let (client_addr, server_addr) =
            (Address::from(client_pub_key), Address::from(server_pub_key));
        let opening_key = client_addr.precompute_opening_key(&server_priv_key);
        let sealing_key = server_addr.precompute_sealing_key(&client_priv_key);

        run_test("sealed_envelope_codec", || {
            let mut codec: SealedEnvelopeCodec = Default::default();
            let mut buf = bytes::BytesMut::new();
            for i in 0..5 {
                let open_envelope =
                    OpenEnvelope::new(client_addr.clone(), server_addr.clone(), &vec![i]);
                let mut sealed_envelope = open_envelope.seal(&sealing_key);
                codec.encode(sealed_envelope, &mut buf);
            }

            for i in 0..5 {
                let sealed_envelope = codec.decode(&mut buf).unwrap().unwrap();
                info!(
                    "sealed: {}, msg: {:?}",
                    sealed_envelope,
                    sealed_envelope.msg()
                );
                let open_envelope = sealed_envelope.open(&opening_key).unwrap();
                info!("opened: {}, msg: {:?}", open_envelope, open_envelope.msg());
            }

            // buf is empty
            assert!(codec.decode(&mut buf).unwrap().is_none());

            let open_envelope =
                OpenEnvelope::new(client_addr.clone(), server_addr.clone(), &vec![1]);
            let mut sealed_envelope = open_envelope.seal(&sealing_key);

            let bytes = rmp_serde::to_vec(&sealed_envelope).unwrap();
            let (left, right) = bytes.split_at(SEALED_ENVELOPE_MIN_SIZE);
            buf.extend_from_slice(left);
            assert!(codec.decode(&mut buf).unwrap().is_none());
            buf.extend_from_slice(right);
            assert!(codec.decode(&mut buf).unwrap().is_some());
        });
    }
}
