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

//! message related errors

use crate::security;
use oysterpack_errors::{ErrorMessage, IsError};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use std::fmt;

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
    SealedEnvelope,
    BytesMessage,
    SigningDomain,
    Domain,
    SigningService,
    Service,
}

/// nng:Message related error
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BincodeSerializeError(pub(crate) Scope, pub(crate) ErrorMessage);

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
        write!(
            f,
            "`{:?}` bincode deserialization failed: {}",
            self.0, self.1
        )
    }
}

/// bincode deserialization related error
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BincodeDeserializeError(pub(crate) Scope, pub(crate) ErrorMessage);

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
        write!(
            f,
            "`{:?}` bincode deserialization failed: {}",
            self.0, self.1
        )
    }
}

/// snappy decompression error
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnappyDecompressError(pub(crate) ErrorMessage);

impl SnappyDecompressError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1869668521103431848013804724538751291);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for SnappyDecompressError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SnappyDecompressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "snappy decompression failed: {}", self.0)
    }
}

/// The Domain must be a child.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainMustBeChildConstraintError {
    parent_id: security::DomainId,
    invalid_child_id: security::DomainId,
    msg: ErrorMessage,
}

impl DomainMustBeChildConstraintError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1869845396788067888655278226736804733);
    /// Level::Error
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;

    /// constructor
    pub fn new(
        parent_id: security::DomainId,
        invalid_child_id: security::DomainId,
        msg: &str,
    ) -> DomainMustBeChildConstraintError {
        DomainMustBeChildConstraintError {
            parent_id,
            invalid_child_id,
            msg: ErrorMessage(msg.to_string()),
        }
    }
}

impl IsError for DomainMustBeChildConstraintError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for DomainMustBeChildConstraintError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: domain {} is not a child of {}",
            self.msg, self.invalid_child_id, self.parent_id
        )
    }
}

/// The service is not owned by the domain
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceNotOwnedByDomainError {
    domain_id: security::DomainId,
    service_id: security::ServiceId,
    msg: ErrorMessage,
}

impl ServiceNotOwnedByDomainError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1869868324958312490751247039544724835);
    /// Level::Error
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;

    /// constructor
    pub fn new(
        domain_id: security::DomainId,
        service_id: security::ServiceId,
        msg: &str,
    ) -> ServiceNotOwnedByDomainError {
        ServiceNotOwnedByDomainError {
            domain_id,
            service_id,
            msg: ErrorMessage(msg.to_string()),
        }
    }
}

impl IsError for ServiceNotOwnedByDomainError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for ServiceNotOwnedByDomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: domain {} does not own service {}",
            self.msg, self.domain_id, self.service_id,
        )
    }
}

/// The Domain must be a child.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SigningSecretKeyDoesNotMatchPublicKeyError(pub sign::PublicKey);

impl SigningSecretKeyDoesNotMatchPublicKeyError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1869854140619665551699628258648836837);
    /// Level::Error
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
}

impl IsError for SigningSecretKeyDoesNotMatchPublicKeyError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SigningSecretKeyDoesNotMatchPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid secret key for public key : {}",
            bs58::encode(&(self.0).0).into_string()
        )
    }
}
