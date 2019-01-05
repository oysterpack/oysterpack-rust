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

//! Defines the security model for secure messaging
//!

use crate::errors;
use oysterpack_errors::{op_error, Error, ErrorMessage};
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, sign};
use std::{fmt, str::FromStr};

/// public key sare 32 bytes
pub const PUB_KEY_SIZE: u8 = 32;

/// Domain ID
#[ulid]
pub struct DomainId(pub u128);

/// Domains contain services and sub-domains.
/// - DomainId is permanently assigned to a Domain
/// - crypto keys can be re-issued
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Domain {
    id: DomainId,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_domain_id: Option<DomainId>,
    // (DomainId + signing_public_key) signature
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_domain_signature: Option<sign::Signature>,
    signing_public_key: sign::PublicKey,
    signing_secret_key: sign::SecretKey,
}

impl Domain {
    /// creates a new root domain
    pub fn new_root_domain() -> Domain {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        Domain {
            id: DomainId::generate(),
            parent_domain_id: None,
            parent_domain_signature: None,
            signing_public_key,
            signing_secret_key,
        }
    }

    /// creates a new child Domain
    pub fn new_child_domain(&self) -> Domain {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let id = DomainId::generate();
        Domain {
            id,
            parent_domain_id: Some(self.id),
            parent_domain_signature: Some(sign::sign_detached(
                &Domain::signing_data(id, &signing_public_key),
                &self.signing_secret_key,
            )),
            signing_public_key,
            signing_secret_key,
        }
    }

    fn signing_data(id: DomainId, key: &sign::PublicKey) -> [u8; 48] {
        let mut data: [u8; 48] = [0; 48];
        {
            let temp = &mut data[..16];
            temp.copy_from_slice(&id.ulid().to_bytes());
            let temp = &mut data[16..];
            temp.copy_from_slice(&key.0);
        }
        data
    }

    /// Verifies the parent domain signature.
    ///
    /// If this is a root domain, then true is always returned.
    pub fn verify(&self, parent_domain_key: &sign::PublicKey) -> bool {
        self.parent_domain_signature.map_or(true, |signature| {
            sign::verify_detached(
                &signature,
                &Domain::signing_data(self.id, &self.signing_public_key),
                parent_domain_key,
            )
        })
    }

    /// public key used to verify signatures
    pub fn signing_public_key(&self) -> &sign::PublicKey {
        &self.signing_public_key
    }

    /// DomainId getter
    pub fn id(&self) -> DomainId {
        self.id
    }

    /// If None is returned, then this is a root Domain
    pub fn parent_domain_id(&self) -> Option<DomainId> {
        self.parent_domain_id
    }

    /// returns true if this is a root domain, i.e., it has no parent
    pub fn is_root_domain(&self) -> bool {
        self.parent_domain_id.is_none()
    }

    /// If this Domain has a parent, then the parent Domain signature is returned.
    /// The parent signs the child to prove that it is the parent's child.
    pub fn parent_domain_signature(&self) -> Option<&sign::Signature> {
        self.parent_domain_signature.as_ref()
    }

    /// constructs a new Domain Service
    pub fn new_service(&self) -> Service {
        let id = ServiceId::generate();
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        Service {
            id,
            domain_id: self.id,
            domain_signature: sign::sign_detached(
                &Service::signing_data(id, self.id, &signing_public_key),
                &self.signing_secret_key,
            ),
            signing_public_key,
            signing_secret_key,
        }
    }
}

/// Service ID
#[ulid]
pub struct ServiceId(pub u128);

/// Services can only be owned by a single domain, i.e., 1 ServiceId = 1 Service
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Service {
    id: ServiceId,
    domain_id: DomainId,
    // (ServiceId + DomainId + signing_public_key) signature
    domain_signature: sign::Signature,
    signing_public_key: sign::PublicKey,
    signing_secret_key: sign::SecretKey,
}

impl Service {
    fn signing_data(id: ServiceId, domain_id: DomainId, key: &sign::PublicKey) -> [u8; 64] {
        let mut data: [u8; 64] = [0; 64];
        {
            let temp = &mut data[..16];
            temp.copy_from_slice(&id.ulid().to_bytes());
            let temp = &mut data[16..32];
            temp.copy_from_slice(&domain_id.ulid().to_bytes());
            let temp = &mut data[32..];
            temp.copy_from_slice(&key.0);
        }
        data
    }
}

/// Each service instance is assigned a unique service address.
/// - a new service address is assigned each time a service instance is started
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceAddress {
    address: Address,
    service_id: ServiceId,
    // (Address + ServiceId) signature
    service_signature: sign::Signature,
}

/// Subject ID
#[ulid]
pub struct SubjectId(pub u128);

/// In a security context, a subject is any entity that requests access to an object.
/// These are generic terms used to denote the thing requesting access and the thing the request is made against.
/// When you log onto an application you are the subject and the application is the object.
/// When someone knocks on your door the visitor is the subject requesting access and your home is the object access is requested of.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Subject {
    id: SubjectId,
    signing_public_key: sign::PublicKey,
    signing_secret_key: sign::SecretKey,
}

impl Subject {
    /// public key that is used to verify Subject signatures
    pub fn signing_public_key(&self) -> &sign::PublicKey {
        &self.signing_public_key
    }

    /// sign the specified data
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        sign::sign(data, &self.signing_secret_key)
    }

    /// sign the specified data and returns the detached signature
    pub fn sign_detached(&self, data: &[u8]) -> sign::Signature {
        sign::sign_detached(data, &self.signing_secret_key)
    }
}

/// Subject address
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SubjectAddress {
    address: Address,
    subject_id: SubjectId,
    // (Addresss + SubjectId) signature
    subject_signature: sign::Signature,
}

/// Subject address
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceClientAddress {
    subject_address: SubjectAddress,
    // SubjectAddress.subject_signature is signed by the service
    service_signature: sign::Signature,
}

/// A signed address is signed by a trusted third party
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SignedAddress {
    address: Address,
    signer: sign::PublicKey,
    signature: sign::Signature,
}

impl SignedAddress {
    /// Address getter
    pub fn address(&self) -> Address {
        self.address
    }

    /// verifies the signature
    pub fn verify(&self) -> bool {
        sign::verify_detached(&self.signature, &(self.address.0).0, &self.signer)
    }

    /// returns the signer's public key
    pub fn signer(&self) -> sign::PublicKey {
        self.signer
    }

    /// returns the signature
    pub fn signature(&self) -> sign::Signature {
        self.signature
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

    /// computes an intermediate key that can be used to encrypt / decrypt data
    pub fn precompute_key(&self, secret_key: &box_::SecretKey) -> box_::PrecomputedKey {
        box_::precompute(&self.0, secret_key)
    }

    /// signs the address using the specified key pair
    pub fn sign(&self, public_key: sign::PublicKey, secret_key: &sign::SecretKey) -> SignedAddress {
        SignedAddress {
            address: *self,
            signer: public_key,
            signature: sign::sign_detached(&(self.0).0, secret_key),
        }
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

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn signed_public_key() {
        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let (client_signing_public_key, client_signing_private_key) =
            sodiumoxide::crypto::sign::gen_keypair();
        let (server_signing_public_key, server_signing_private_key) =
            sodiumoxide::crypto::sign::gen_keypair();

        // client public key has multiple signatures:
        // - self signed by the client
        // - server signed - which means it was authorized by the server
        let self_signed_client_public_key =
            sodiumoxide::crypto::sign::sign(&client_public_key.0, &client_signing_private_key);
        println!(
            "self_signed_client_public_key.len() = {}",
            self_signed_client_public_key.len()
        );
        println!("{:?}", self_signed_client_public_key);
        let server_signed_client_public_key = sodiumoxide::crypto::sign::sign(
            &self_signed_client_public_key,
            &server_signing_private_key,
        );
        println!(
            "server_signed_client_public_key.len() = {}",
            server_signed_client_public_key.len()
        );
        println!("{:?}", server_signed_client_public_key);
        let verified_client_public_key = sodiumoxide::crypto::sign::verify(
            &server_signed_client_public_key,
            &server_signing_public_key,
        )
        .unwrap();
        println!(
            "verified_client_public_key.len() = {}",
            verified_client_public_key.len()
        );
        println!("{:?}", verified_client_public_key);
        let verified_client_public_key = sodiumoxide::crypto::sign::verify(
            &verified_client_public_key,
            &client_signing_public_key,
        )
        .unwrap();
        println!(
            "verified_client_public_key.len() = {}",
            verified_client_public_key.len()
        );
        println!("{:?}", verified_client_public_key);
        assert_eq!(&verified_client_public_key, &client_public_key.0);
    }

    #[test]
    fn signed_address() {
        let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
        let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();

        let (client_signing_public_key, client_signing_private_key) =
            sodiumoxide::crypto::sign::gen_keypair();
        let (server_signing_public_key, server_signing_private_key) =
            sodiumoxide::crypto::sign::gen_keypair();

        let client_addr = Address::from(client_public_key);
        let signed_client_addr =
            client_addr.sign(server_signing_public_key, &server_signing_private_key);
        assert!(signed_client_addr.verify(), "failed to verify signed addr");
    }

    #[test]
    fn domain_new_child_domain() {
        let root_domain = Domain::new_root_domain();
        let child_domain = root_domain.new_child_domain();
        assert!(child_domain.verify(root_domain.signing_public_key()))
    }

}
