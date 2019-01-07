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

//! Defines the security model for secure messaging.
//! - domains own sub-domains and services
//! - each service can have multiple service instances, which are identified by unique addresses
//! - subjects represent entities that interact with services
//! - service clients represent relationships between services and subjects
//!   - a service client maps to an address which is used to encrypt messages between a service instance
//!     and the service client
//!   - a service client is dual signed by the subject and the service. It is signed by the subject
//!     to prove that the service client was authorized by the subject - the subject owns the address.
//!     It is signed by the service to prove that service has authorized access to the service from
//!     the service client.
//! - each entity is signed by its owner, which proves ownership and authenticity
//!   - domains are signed by its parent domain
//!   - services are signed by its owning domain
//!   - service instances are signed by its owning service
//!   - service clients are signed by the subject and service

use crate::errors;
use crate::marshal;
use oysterpack_errors::{op_error, Error, ErrorMessage};
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, sign};
use std::{fmt, str::FromStr};

/// public key size is 32 bytes
pub const PUB_KEY_SIZE: u8 = 32;
/// public key size 16 bytes
pub const ULID_SIZE: u8 = 16;

/// Domain ID
#[ulid]
pub struct DomainId(pub u128);

/// Domain that contains the secret key, which can be used for signing purposes
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SigningDomain {
    domain: Domain,
    signing_secret_key: sign::SecretKey,
}

impl SigningDomain {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<SigningDomain, Error> {
        marshal::decode(bytes)
    }

    /// creates a new root domain
    pub fn new_root_domain() -> SigningDomain {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let signing_public_key = DomainSigningKey(signing_public_key);
        let domain = Domain {
            id: DomainId::generate(),
            parent_domain_id: None,
            parent_domain_signature: None,
            signing_public_key,
        };

        SigningDomain {
            domain,
            signing_secret_key,
        }
    }

    /// creates a new child Domain
    pub fn new_child_domain(&self) -> SigningDomain {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let signing_public_key = DomainSigningKey(signing_public_key);
        let id = DomainId::generate();
        let domain = Domain {
            id,
            parent_domain_id: Some(self.domain.id),
            parent_domain_signature: Some(sign::sign_detached(
                &Domain::signing_data(id, &signing_public_key),
                &self.signing_secret_key,
            )),
            signing_public_key,
        };

        SigningDomain {
            domain,
            signing_secret_key,
        }
    }

    /// Generates new signing keys for the Domain. This mean all keys and signatures for the Domain
    /// hierarchy are invalidated. Keys and signatures will need to be regenerated for the entire
    /// domain sub-tree.
    pub fn with_new_keys(self) -> SigningDomain {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let domain = Domain {
            signing_public_key: DomainSigningKey(signing_public_key),
            ..self.domain
        };

        SigningDomain {
            domain,
            signing_secret_key,
        }
    }

    /// Updates the child Domain signature.
    ///
    /// ## Errors
    /// - DomainMustBeChildConstraintError
    pub fn sign_domain(&self, child: &Domain) -> Result<Domain, Error> {
        if self.domain.is_child(&child) {
            Ok(Domain {
                id: child.id,
                parent_domain_id: Some(self.domain.id),
                parent_domain_signature: Some(sign::sign_detached(
                    &Domain::signing_data(child.id, &child.signing_public_key),
                    &self.signing_secret_key,
                )),
                signing_public_key: child.signing_public_key,
            })
        } else {
            Err(op_error!(errors::DomainMustBeChildConstraintError::new(
                self.domain.id,
                child.id,
                "only child domains can be signed"
            )))
        }
    }

    /// constructs a new Domain Service
    pub fn new_service(&self) -> SigningService {
        let id = ServiceId::generate();
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let signing_public_key = ServiceSigningKey(signing_public_key);
        let service = Service {
            id,
            domain_id: self.domain.id,
            domain_signature: sign::sign_detached(
                &Service::signing_data(id, self.domain.id, &signing_public_key.0),
                &self.signing_secret_key,
            ),
            signing_public_key,
        };
        SigningService {
            service,
            signing_secret_key,
        }
    }

    /// Updates the service signature. If the service does not belong to this domain, then the
    ///
    /// ## Errors
    /// -
    pub fn sign_service(&self, service: &Service) -> Result<Service, Error> {
        if self.domain.id == service.domain_id {
            Ok(Service {
                id: service.id,
                domain_id: self.domain.id,
                domain_signature: sign::sign_detached(
                    &Service::signing_data(
                        service.id,
                        self.domain.id,
                        &service.signing_public_key.0,
                    ),
                    &self.signing_secret_key,
                ),
                signing_public_key: service.signing_public_key,
            })
        } else {
            Err(op_error!(errors::ServiceNotOwnedByDomainError::new(
                self.domain.id,
                service.id,
                "a service can only be signed by the domain if it owns it"
            )))
        }
    }

    /// Domain
    pub fn domain(&self) -> &Domain {
        &self.domain
    }
}

/// Domains contain services and sub-domains.
/// - DomainId is permanently assigned to a Domain
/// - crypto keys can be re-issued
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Domain {
    id: DomainId,
    signing_public_key: DomainSigningKey,
    parent_domain_id: Option<DomainId>,
    // (DomainId + signing_public_key) signature
    parent_domain_signature: Option<sign::Signature>,
}

impl Domain {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// uses deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<Domain, Error> {
        marshal::decode(bytes)
    }

    /// Data that is used for signing: DomainId (16 bytes) + signing_public_key (32 bytes)
    pub fn signing_data(id: DomainId, key: &DomainSigningKey) -> [u8; 48] {
        let mut data: [u8; 48] = [0; 48];
        {
            let temp = &mut data[..16];
            temp.copy_from_slice(&id.ulid().to_bytes());
            let temp = &mut data[16..];
            temp.copy_from_slice(&(key.0).0);
        }
        data
    }

    /// checks if the specified Domain is a child of this domain
    pub fn is_child(&self, domain: &Domain) -> bool {
        domain.parent_domain_id.map_or(false, |id| id == self.id)
    }

    /// Verifies the parent domain signature. If this is a root domain, then true is always returned.
    ///
    /// NOTE: to truly verify the domain, you would need to verify all of the ancestor domains up
    /// to the root domain.
    pub fn verify(&self, parent_domain_key: &DomainSigningKey) -> bool {
        self.parent_domain_signature.map_or(true, |signature| {
            sign::verify_detached(
                &signature,
                &Domain::signing_data(self.id, &self.signing_public_key),
                &parent_domain_key.0,
            )
        })
    }

    /// public key used to verify signatures
    pub fn signing_public_key(&self) -> &DomainSigningKey {
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
}

/// Domain signing public key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DomainSigningKey(pub sign::PublicKey);

/// Service signing public key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceSigningKey(pub sign::PublicKey);

/// Subject signing public key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SubjectSigningKey(pub sign::PublicKey);

/// Service ID
#[ulid]
pub struct ServiceId(pub u128);

/// Service that contains the signing secret key
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SigningService {
    service: Service,
    signing_secret_key: sign::SecretKey,
}

impl SigningService {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// uses deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<Domain, Error> {
        marshal::decode(bytes)
    }

    /// returns a new service instance
    pub fn new_service_instance(&self, address: Address) -> ServiceInstance {
        ServiceInstance {
            address,
            service_id: self.service.id,
            // (Address + ServiceId) signature
            service_signature: sign::sign_detached(
                &ServiceInstance::signing_data(address, self.service().id()),
                &self.signing_secret_key,
            ),
        }
    }

    /// Service
    pub fn service(&self) -> &Service {
        &self.service
    }
}

/// Services can only be owned by a single domain, i.e., 1 ServiceId = 1 Service
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Service {
    id: ServiceId,
    domain_id: DomainId,
    // (ServiceId + DomainId + signing_public_key) signature
    domain_signature: sign::Signature,
    signing_public_key: ServiceSigningKey,
}

impl Service {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// uses deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<Domain, Error> {
        marshal::decode(bytes)
    }

    /// Data that is used for signing: ServiceId (16 bytes) + DomainId (16 bytes) + signing_public_key (32 bytes)
    pub fn signing_data(id: ServiceId, domain_id: DomainId, key: &sign::PublicKey) -> [u8; 64] {
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

    /// verifies that the service is owned by the specified domain
    pub fn verify(&self, domain_key: &DomainSigningKey) -> bool {
        sign::verify_detached(
            &self.domain_signature,
            &Service::signing_data(self.id, self.domain_id, &self.signing_public_key.0),
            &domain_key.0,
        )
    }

    /// ServiceId
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// DomainId
    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// signing public key
    pub fn signing_public_key(&self) -> &ServiceSigningKey {
        &self.signing_public_key
    }

    /// Domain signature, which proves that the Domain owns this Service
    pub fn domain_signature(&self) -> &sign::Signature {
        &self.domain_signature
    }
}

/// Each service instance is assigned a unique service address.
/// - a new service address is assigned each time a service instance is started
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceInstance {
    address: Address,
    service_id: ServiceId,
    // (Address + ServiceId) signature
    service_signature: sign::Signature,
}

impl ServiceInstance {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// uses deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<Domain, Error> {
        marshal::decode(bytes)
    }

    /// Data that is signed = Address(32 bytes) + ServiceId(16 bytes)
    pub fn signing_data(address: Address, service_id: ServiceId) -> [u8; 48] {
        let mut data: [u8; 48] = [0; 48];
        {
            let temp = &mut data[..32];
            temp.copy_from_slice(&address.public_key().0);
            let temp = &mut data[32..];
            temp.copy_from_slice(&service_id.ulid().to_bytes());
        }
        data
    }

    /// verifies the signature against the specified key
    ///
    /// NOTE: to fully verify, the service would need to be verified all the way back to the root Domain
    pub fn verify(&self, service_key: &sign::PublicKey) -> bool {
        sign::verify_detached(
            &self.service_signature,
            &ServiceInstance::signing_data(self.address, self.service_id),
            service_key,
        )
    }

    /// Address
    pub fn address(&self) -> Address {
        self.address
    }

    /// ServiceId
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// Service signature, which proves that the Address was issued by the Service
    pub fn service_signature(&self) -> &sign::Signature {
        &self.service_signature
    }
}

/// Subject ID
#[ulid]
pub struct SubjectId(pub u128);

/// Associates a secret key with a subject for signing purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SigningSubject {
    subject: Subject,
    signing_secret_key: sign::SecretKey,
}

impl SigningSubject {
    /// constructor
    pub fn generate() -> SigningSubject {
        let (signing_public_key, signing_secret_key) = sign::gen_keypair();
        let subject = Subject {
            id: SubjectId::generate(),
            signing_public_key: SubjectSigningKey(signing_public_key),
        };
        SigningSubject {
            subject,
            signing_secret_key,
        }
    }

    /// Subject
    pub fn subject(&self) -> &Subject {
        &self.subject
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

/// In a security context, a subject is any entity that requests access to an object.
/// These are generic terms used to denote the thing requesting access and the thing the request is made against.
/// When you log onto an application you are the subject and the application is the object.
/// When someone knocks on your door the visitor is the subject requesting access and your home is the object access is requested of.
///
/// - subjects are assigned addresses via ServiceClient
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Subject {
    id: SubjectId,
    signing_public_key: SubjectSigningKey,
}

impl Subject {
    /// serialize
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        marshal::encode(self)
    }

    /// uses deserialize
    pub fn deserialize(bytes: &[u8]) -> Result<Domain, Error> {
        marshal::decode(bytes)
    }

    /// SubjectId
    pub fn id(&self) -> SubjectId {
        self.id
    }

    /// public key that is used to verify Subject signatures
    pub fn signing_public_key(&self) -> &SubjectSigningKey {
        &self.signing_public_key
    }
}

/// A service client is a subject that is authorized to access a service.
/// It contains a dual signature - it is signed by the subject and by the service
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceClient {
    address: Address,
    subject_id: SubjectId,
    service_id: ServiceId,
    // (Address + SubjectId + ServiceId) is first signed by the Subject, and then the signature is signed by the Service
    subject_signature: sign::Signature,
    // signs the subject signature
    service_signature: sign::Signature,
}

impl ServiceClient {
    /// constructor
    pub fn new(
        address: Address,
        subject: &SigningSubject,
        service: &SigningService,
    ) -> ServiceClient {
        let subject_signature = sign::sign_detached(
            &ServiceClient::signing_data(address, subject.subject.id, service.service.id),
            &subject.signing_secret_key,
        );
        let service_signature =
            sign::sign_detached(&subject_signature.0, &service.signing_secret_key);
        ServiceClient {
            address,
            subject_id: subject.subject.id,
            service_id: service.service.id,
            subject_signature,
            service_signature,
        }
    }

    /// Data that is signed = Address(32 bytes) + SubjectId (16 bytes) + ServiceId(16 bytes)
    pub fn signing_data(
        address: Address,
        subject_id: SubjectId,
        service_id: ServiceId,
    ) -> [u8; 64] {
        let mut data: [u8; 64] = [0; 64];
        {
            let temp = &mut data[..32];
            temp.copy_from_slice(&address.public_key().0);
            let temp = &mut data[32..48];
            temp.copy_from_slice(&subject_id.ulid().to_bytes());
            let temp = &mut data[48..];
            temp.copy_from_slice(&service_id.ulid().to_bytes());
        }
        data
    }

    /// verifies that the ServiceClient was signed by the subject and the service
    pub fn verify(&self, subject_key: &SubjectSigningKey, service_key: &ServiceSigningKey) -> bool {
        sign::verify_detached(
            &self.service_signature,
            &self.subject_signature.0,
            &service_key.0,
        ) && sign::verify_detached(
            &self.subject_signature,
            &ServiceClient::signing_data(self.address, self.subject_id, self.service_id),
            &subject_key.0,
        )
    }

    /// Address
    pub fn address(&self) -> Address {
        self.address
    }

    /// SubjectId
    pub fn subject_id(&self) -> SubjectId {
        self.subject_id
    }

    /// ServiceId
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// Service signature, which authorizes the subject to access the service
    pub fn service_signature(&self) -> &sign::Signature {
        &self.service_signature
    }

    /// Subject signature which proves that the address belongs to the Subject
    pub fn subject_signature(&self) -> &sign::Signature {
        &self.subject_signature
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
    fn domain_hierarchy() {
        let signing_root_domain = SigningDomain::new_root_domain();
        let signing_child_domain = signing_root_domain.new_child_domain();

        let root_domain = signing_root_domain.domain();
        let child_domain = signing_child_domain.domain();
        assert!(root_domain.is_child(&child_domain));
        assert!(!child_domain.is_child(&root_domain));
        assert!(child_domain.verify(root_domain.signing_public_key()));
        let signing_root_domain = signing_root_domain.with_new_keys();
        let root_domain = signing_root_domain.domain();
        assert!(!child_domain.verify(root_domain.signing_public_key()));
        let child_domain = signing_root_domain.sign_domain(child_domain).unwrap();
        assert!(child_domain.verify(root_domain.signing_public_key()));

        let signing_root_domain_2 = SigningDomain::new_root_domain();
        let root_domain_2 = signing_root_domain_2.domain();
        assert!(!child_domain.verify(root_domain_2.signing_public_key()));
        match signing_root_domain_2.sign_domain(&child_domain) {
            Ok(_) => panic!("should have failed because this is not a child domain"),
            Err(err) => assert_eq!(err.id(), errors::DomainMustBeChildConstraintError::ERROR_ID),
        }
        match signing_root_domain_2.sign_domain(root_domain) {
            Ok(_) => panic!("should have failed because this is not a child domain"),
            Err(err) => assert_eq!(err.id(), errors::DomainMustBeChildConstraintError::ERROR_ID),
        }
        match signing_root_domain_2.sign_domain(&root_domain_2) {
            Ok(_) => panic!("should have failed because this is not a child domain"),
            Err(err) => assert_eq!(err.id(), errors::DomainMustBeChildConstraintError::ERROR_ID),
        }

        let grandchild_domain = signing_child_domain.new_child_domain();
        assert!(grandchild_domain
            .domain()
            .verify(signing_child_domain.domain().signing_public_key()));
    }

    #[test]
    fn domain_services() {
        let root_domain = SigningDomain::new_root_domain();
        let root_service = root_domain.new_service();
        assert!(root_service
            .service()
            .verify(root_domain.domain().signing_public_key()));

        let child_domain = root_domain.new_child_domain();
        let child_service = child_domain.new_service();
        assert!(child_service
            .service()
            .verify(child_domain.domain().signing_public_key()));
        assert!(!child_service
            .service()
            .verify(root_domain.domain().signing_public_key()));

        let root_domain = root_domain.with_new_keys();
        assert!(!root_service
            .service()
            .verify(root_domain.domain().signing_public_key()));
        let root_service = root_domain.sign_service(&root_service.service()).unwrap();
        assert!(root_service.verify(root_domain.domain().signing_public_key()));
        assert!(
            !child_service
                .service()
                .verify(root_domain.domain().signing_public_key()),
            "should have not verified because the child service is not owned by the root domain"
        );
        match root_domain.sign_service(&child_service.service()) {
            Ok(_) => panic!(
                "should have failed because the child service is not owned by the root domain"
            ),
            Err(err) => {
                println!("ServiceNotOwnedByDomainError: {}", err);
                assert_eq!(err.id(), errors::ServiceNotOwnedByDomainError::ERROR_ID)
            }
        }
    }

    #[test]
    fn domain_serde() {
        let root = SigningDomain::new_root_domain();
        let child = root.new_child_domain();

        let bytes = root.serialize().unwrap();
        println!("serialized root domain bytes len = {}", bytes.len());
        let root = SigningDomain::deserialize(&bytes).unwrap();
        assert!(child.domain().verify(root.domain().signing_public_key()));

        let bytes = child.serialize().unwrap();
        println!("serialized child domain bytes len = {}", bytes.len());
        let child = SigningDomain::deserialize(&bytes).unwrap();
        assert!(child.domain().verify(root.domain().signing_public_key()));
    }

    #[test]
    fn subject() {
        let root = SigningDomain::new_root_domain();
        let service = root.new_service();
        let subject = SigningSubject::generate();

        let (public_key, _) = box_::gen_keypair();
        let address = Address::from(public_key);
        let service_client = ServiceClient::new(address, &subject, &service);
        assert!(service_client.verify(
            subject.subject().signing_public_key(),
            service.service().signing_public_key()
        ));

        let service2 = root.new_service();
        assert!(!service_client.verify(
            subject.subject().signing_public_key(),
            service2.service().signing_public_key()
        ));
        let subject2 = SigningSubject::generate();
        assert!(!service_client.verify(
            subject2.subject().signing_public_key(),
            service.service().signing_public_key()
        ));
    }

}
