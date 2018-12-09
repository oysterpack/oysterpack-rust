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

//! message errors

use super::Addresses;
use exonum_sodiumoxide::crypto::box_;
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
        }
    }

    fn error_level(&self) -> Level {
        match self {
            MessageError::InvalidAddress(_) => Level::Error,
            MessageError::UnknownSender(_) => Level::Error,
            // trigger an Alert, because this is security related
            MessageError::ForbiddenSender(_) => Level::Alert,
            MessageError::UnknownRecipient(_) => Level::Error,
            // trigger an Alert because sender did not send payment
            MessageError::SenderPaymentRequired(_) => Level::Alert,
            MessageError::DecodingError(_) => Level::Error,
            // Mark this as critical because encoding should never fail besides IO errors
            MessageError::EncodingError(_) => Level::Critical,
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
        }
    }
}

/// Defines the type of Decoding Error
#[derive(Debug)]
pub enum DecodingError {
    /// SealedEnvelope failed to be decoded
    InvalidSealedEnvelope(rmp_serde::decode::Error),
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
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}
