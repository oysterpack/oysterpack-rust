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

//! message errors

use super::{Addresses, SessionId};
use exonum_sodiumoxide::crypto::{box_, sign};
use oysterpack_errors::{Id, IsError, Level};
use std::fmt;

/// Indicates that a SealedEnvelope failed to be open.
#[derive(Debug, Clone)]
pub struct SealedEnvelopeOpenFailed<'a>(pub &'a super::SealedEnvelope);

impl<'a> SealedEnvelopeOpenFailed<'a> {
    /// Error ID(01CY9EP7BMKBRBA56Y13DEHXSQ)
    pub const ERR_ID: Id = Id(1867014431750359479243220706658940727);
    /// Error level
    pub const ERR_LEVEL: Level = Level::Error;
}

impl IsError for SealedEnvelopeOpenFailed<'_> {
    fn error_id(&self) -> Id {
        Self::ERR_ID
    }

    fn error_level(&self) -> Level {
        Self::ERR_LEVEL
    }
}

impl fmt::Display for SealedEnvelopeOpenFailed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed to open SealedEnvelope: {}, nonce: {}, msg.len: {}",
            self.0.addresses(),
            crate::message::base58::encode(&self.0.nonce().0),
            self.0.msg().len()
        )
    }
}

/// Message related errors
#[derive(Debug)]
pub enum MessageError<'a> {
    /// The key is not a valid public-key
    InvalidAddress(&'a [u8]),
    /// the sender's public-key is unknown
    UnknownSender(&'a box_::PublicKey),
    /// the sender is forbidden, i.e., has been blocked
    ForbiddenSender(&'a box_::PublicKey),
    /// the recipient's public-key is unknown.
    UnknownRecipient(&'a box_::PublicKey),
    /// Payment is required
    SenderPaymentRequired(&'a box_::PublicKey),
    /// Decoding error
    DecodingError(DecodingError),
    /// Encoding error
    EncodingError(EncodingError),
    /// Invalid signature
    InvalidSignature(&'a sign::PublicKey),
    /// The session ID length was not 16 bytes, i.e., 128 bits
    InvalidSessionIdLength {
        /// sender address
        from: &'a sign::PublicKey,
        /// invalid session id length that was found in the message
        len: usize,
    },
    /// Session ID is not valid
    InvalidSessionId {
        /// sender address
        from: &'a sign::PublicKey,
        /// session ID
        session_id: SessionId,
    },
    /// Hash digest should be 64 bytes - SHA-512 is used
    InvalidDigestLength {
        /// sender address
        from: &'a sign::PublicKey,
        /// invalid hash digest length that was found in the message
        len: usize,
    },
    /// Data failed a checksum, i.e., its hash did not match
    ChecksumFailed {
        /// sender address
        from: &'a sign::PublicKey,
        /// session ID
        session_id: SessionId,
    },
    /// Decryption failed
    DecryptionFailed {
        /// sender address
        from: &'a sign::PublicKey,
        /// session ID
        session_id: SessionId,
    },
}

impl IsError for MessageError<'_> {
    fn error_id(&self) -> Id {
        match self {
            MessageError::InvalidAddress(_) => Id(1867021926897034296478877125570412391), // 01CY9MKDWMKXTPSJ039G78YFV7
            MessageError::UnknownSender(_) => Id(1867021982369811891428717464289628214), // 01CY9MMTPJM448K523NX3EJM1P
            MessageError::ForbiddenSender(_) => Id(1867021999800795039526843952808187927), // 01CY9MN8S56D56RYBZ1CV4VP0Q
            MessageError::UnknownRecipient(_) => Id(1867022017488503270046354574569684513), // 01CY9MNQ2C44SEMA2XJRVCHFH1
            MessageError::SenderPaymentRequired(_) => Id(1867022032677743826660887081515750864), // 01CY9MP3B0BZZPHD16AW96R5EG
            MessageError::DecodingError(_) => Id(1867031190663367090363230915571813628), // 01CY9VX93CH1C1BJKRAV2T237W
            MessageError::EncodingError(_) => Id(1867031212491238890411696131253995837), // 01CY9VXTQM3ZYPE2JXFF83829X
            MessageError::InvalidSignature(_) => Id(1867172076936046708942329759845274008), // 01CYDB1R45VRVQF03RHZZ2PMCR
            MessageError::InvalidSessionIdLength { .. } => {
                Id(1867172923315771044142605233334059560)
            } // 01CYDBQ3TJRH8GZSXKT4C0T0H8
            MessageError::InvalidDigestLength { .. } => Id(1867177124319618698988963972315043675), // 01CYDF15BZPFRPVX0HG16PENTV
            MessageError::ChecksumFailed { .. } => Id(1867178063957932565585764976839329233), // 01CYDFRWD29J3TP97F47WS71EH
            MessageError::DecryptionFailed { .. } => Id(1867178498353912205043500705741562600), // 01CYDG3V9Y7D0XYEJKCH3MGRQ8
            MessageError::InvalidSessionId { .. } => Id(1867222419281185803983256937238975084), // 01CYEJRJB9R7Q902QW2HT5TMKC
        }
    }

    /// trigger an Alert, for anything security related
    fn error_level(&self) -> Level {
        match self {
            MessageError::InvalidAddress(_) => Level::Error,
            MessageError::UnknownSender(_) => Level::Error,
            MessageError::ForbiddenSender(_) => Level::Alert,
            MessageError::UnknownRecipient(_) => Level::Error,
            // trigger an Alert because sender did not send payment
            MessageError::SenderPaymentRequired(_) => Level::Alert,
            MessageError::DecodingError(_) => Level::Error,
            // Mark this as critical because encoding should never fail besides IO errors
            MessageError::EncodingError(_) => Level::Critical,
            MessageError::InvalidSignature(_) => Level::Alert,
            MessageError::InvalidSessionIdLength { .. } => Level::Alert,
            MessageError::InvalidDigestLength { .. } => Level::Alert,
            MessageError::ChecksumFailed { .. } => Level::Alert,
            MessageError::DecryptionFailed { .. } => Level::Alert,
            MessageError::InvalidSessionId { .. } => Level::Error,
        }
    }
}

impl fmt::Display for MessageError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageError::InvalidAddress(address) => write!(
                f,
                "Invalid address: {}",
                crate::message::base58::encode(address)
            ),
            MessageError::UnknownSender(address) => write!(
                f,
                "Unknown sender: {}",
                crate::message::base58::encode(&address.0)
            ),
            MessageError::ForbiddenSender(address) => write!(
                f,
                "Forbidden sender: {}",
                crate::message::base58::encode(&address.0)
            ),
            MessageError::UnknownRecipient(address) => write!(
                f,
                "Unknown recipient: {}",
                crate::message::base58::encode(&address.0)
            ),
            MessageError::SenderPaymentRequired(address) => write!(
                f,
                "Sender payment is required: {}",
                crate::message::base58::encode(&address.0)
            ),
            MessageError::DecodingError(err) => write!(f, "Failed to decode: {}", err),
            MessageError::EncodingError(err) => write!(f, "Failed to encode: {}", err),
            MessageError::InvalidSignature(address) => write!(
                f,
                "Invalid signature from: {}",
                crate::message::base58::encode(&address.0)
            ),
            MessageError::InvalidSessionIdLength { from, len } => write!(
                f,
                "Session ID len should be 16 but was ({}) - from: {}",
                len,
                crate::message::base58::encode(&from.0)
            ),
            MessageError::InvalidDigestLength { from, len } => write!(
                f,
                "Digest len should be 64 but was ({}) - from: {}",
                len,
                crate::message::base58::encode(&from.0)
            ),
            MessageError::ChecksumFailed { from, session_id } => write!(
                f,
                "Checksum failed: from: {}, session_id: {}",
                crate::message::base58::encode(&from.0),
                session_id
            ),
            MessageError::DecryptionFailed { from, session_id } => write!(
                f,
                "Decrption failed - from: {}, session_id: {}",
                crate::message::base58::encode(&from.0),
                session_id
            ),
            MessageError::InvalidSessionId { from, session_id } => write!(
                f,
                "Invalid session ID [{}] from: {}",
                session_id,
                crate::message::base58::encode(&from.0)
            ),
        }
    }
}

/// Defines the type of Decoding Error
#[derive(Debug)]
pub enum DecodingError {
    /// SealedEnvelope failed to be decoded
    InvalidSealedEnvelope(rmp_serde::decode::Error),
    /// SealedSignedMessage failed to be decoded
    InvalidSealedSignedMessage(rmp_serde::decode::Error),
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Defines the type of Encoding Error
#[derive(Debug)]
pub enum EncodingError {
    /// SealedEnvelope failed to be encoded
    InvalidSealedEnvelope(rmp_serde::encode::Error),
    /// SealedSignedMessage failed to be encoded
    InvalidSealedSignedMessage(rmp_serde::encode::Error),
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}
