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

//! Provides the [ULID](https://github.com/ulid/spec) functionality.

use chrono::{DateTime, Utc};
use rusty_ulid::{self, Ulid};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
};

/// Returns a new ULID encoded as a String.
pub fn ulid_str() -> String {
    rusty_ulid::generate_ulid_string()
}

/// Returns a new ULID encoded as u128
pub fn ulid_u128() -> u128 {
    rusty_ulid::Ulid::generate().into()
}

/// Converts a ULID string representation into u128
pub fn ulid_str_into_u128(ulid: &str) -> Result<u128, DecodingError> {
    rusty_ulid::Ulid::from_str(ulid)
        .map(|ulid| ulid.into())
        .map_err(DecodingError::from)
}

/// Converts a ULID u128 representation into String
pub fn ulid_u128_into_string(ulid: u128) -> String {
    rusty_ulid::Ulid::from(ulid).to_string()
}

/// Provides the core ULID functionality.
///
/// ```rust
/// # extern crate oysterpack_uid;
/// # use oysterpack_uid::*;
/// let id = ULID::generate();
///
///
/// // Get the ULID creation timestamp
/// let datetime = id.datetime();
///
/// // ULID provides a bunch of useful conversion
/// let id: u128 = ULID::generate().into();
/// let id = ULID::from(id);
/// let (id_1, id_2): (u64, u64) = ULID::generate().into();
///
/// let id3: ULID = "01CVG2MP5HJ45SRJTRRHRQ3RJ0".parse().unwrap();
///
/// // ULIDs are passed by value, i.e., copied
///
/// fn foo(id: ULID) {
///   println!("{}", id);
/// }
///
/// foo(id3);
/// foo(id3);
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ULID(Ulid);

impl ULID {
    /// Constructor which generates new ULID
    pub fn generate() -> ULID {
        ULID(Ulid::generate())
    }

    /// Returns the timestamp of this ULID as a DateTime<Utc>.
    pub fn datetime(&self) -> DateTime<Utc> {
        self.0.datetime()
    }
}

impl fmt::Display for ULID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl FromStr for ULID {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        rusty_ulid::Ulid::from_str(s)
            .map(ULID)
            .map_err(DecodingError::from)
    }
}

impl From<[u8; 16]> for ULID {
    fn from(bytes: [u8; 16]) -> Self {
        ULID(Ulid::from(bytes))
    }
}

impl From<u128> for ULID {
    fn from(id: u128) -> Self {
        ULID(Ulid::from(id))
    }
}

impl From<(u64, u64)> for ULID {
    fn from(id: (u64, u64)) -> Self {
        ULID(Ulid::from(id))
    }
}

impl From<ULID> for u128 {
    fn from(ulid: ULID) -> Self {
        ulid.0.into()
    }
}

impl From<ULID> for (u64, u64) {
    fn from(ulid: ULID) -> Self {
        ulid.0.into()
    }
}

impl Serialize for ULID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ULID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ULIDVisitor;

        impl<'de> Visitor<'de> for ULIDVisitor {
            type Value = ULID;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("ULID")
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            #[inline]
            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(value)))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value
                    .parse()
                    .map_err(|_| de::Error::invalid_type(de::Unexpected::Str(value), &"a ULID"))
            }
        }

        deserializer.deserialize_str(ULIDVisitor)
    }
}

/// A TypedULID represents a domain specific ULID, where the domain is defined and enforced by the
/// type system.
///
/// ## How to define a TypedULID for structs
/// ```rust
/// # use oysterpack_uid::TypedULID;
/// struct Foo;
/// type FooId = TypedULID<Foo>;
/// let id = FooId::generate();
/// ```
/// ## How to define TypedULID for traits
/// ```rust
/// # use oysterpack_uid::TypedULID;
/// trait Foo{}
/// // traits are not Send. Send is added to the type def in order to satisfy TypedULID type constraints
/// // in order to be able to send the TypedULID across threads
/// type FooId = TypedULID<dyn Foo + Send + Sync>;
/// let id = FooId::generate();
/// ```
///
/// TypedULID&lt;T&gt; can be converted to a [DomainULID](struct.DomainULID.html) automatically if the
/// TypedULID type T implements [HasDomain](trait.HasDomain.html).
///
/// ```rust
/// # use oysterpack_uid::*;
/// struct Foo;
///
/// impl HasDomain for Foo {
///     const DOMAIN: Domain = Domain("Foo");
/// }
///
/// type FooId = TypedULID<Foo>;
/// let id = FooId::generate();
/// let id: DomainULID = id.into();
/// assert_eq!(id.domain(), Foo::DOMAIN.name());
///
/// ```
pub struct TypedULID<T: 'static + ?Sized> {
    id: ULID,
    _type: PhantomData<T>,
}

impl<T: 'static + ?Sized> TypedULID<T> {
    /// New TypedULID instances are guaranteed to be lexographically sortable if they are created within the same
    /// millisecond. In other words, TypedULID(s) created within the same millisecond are random.
    pub fn generate() -> TypedULID<T> {
        TypedULID {
            id: ULID::generate(),
            _type: PhantomData,
        }
    }

    /// Creates the next strictly monotonic ULID for the given previous ULID.
    /// Returns None if the random part of the next ULID would overflow.
    pub fn next(previous: TypedULID<T>) -> Option<TypedULID<T>> {
        Ulid::next_strictly_monotonic(previous.id.0).map(|next| TypedULID {
            id: ULID(next),
            _type: PhantomData,
        })
    }

    /// returns the ulid
    pub fn ulid(&self) -> ULID {
        self.id
    }

    /// Returns a new ULID with the random part incremented by one.
    /// Returns None if the new ULID overflows.
    pub fn increment(self) -> Option<TypedULID<T>> {
        Self::next(self)
    }
}

impl<T: 'static> Serialize for TypedULID<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de, T: 'static> Deserialize<'de> for TypedULID<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UidVisitor<T: 'static>(PhantomData<&'static T>);

        impl<'de, T: 'static> Visitor<'de> for UidVisitor<T> {
            type Value = TypedULID<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("ULID")
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TypedULID::from(ULID::from(u128::from(value))))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TypedULID::from(ULID::from(u128::from(value))))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TypedULID::from(ULID::from(u128::from(value))))
            }

            #[inline]
            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TypedULID::from(ULID::from(value)))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TypedULID::from_str(value)
                    .map_err(|_| de::Error::invalid_type(de::Unexpected::Str(value), &"a ULID"))
            }
        }

        deserializer.deserialize_str(UidVisitor(PhantomData))
    }
}

impl<T: 'static + ?Sized> fmt::Display for TypedULID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(&self.id.to_string())
    }
}

impl<T: 'static + ?Sized> PartialEq for TypedULID<T> {
    fn eq(&self, other: &TypedULID<T>) -> bool {
        self.id == other.id
    }
}

impl<T: 'static + ?Sized> PartialOrd for TypedULID<T> {
    fn partial_cmp(&self, other: &TypedULID<T>) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: 'static + ?Sized> Eq for TypedULID<T> {}

impl<T: 'static + ?Sized> Ord for TypedULID<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: 'static + ?Sized> Hash for TypedULID<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: 'static + ?Sized> Copy for TypedULID<T> {}

impl<T: 'static + ?Sized> Clone for TypedULID<T> {
    fn clone(&self) -> TypedULID<T> {
        *self
    }
}

impl<T: 'static + ?Sized> fmt::Debug for TypedULID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.id)
    }
}

impl<T: 'static + ?Sized> From<[u8; 16]> for TypedULID<T> {
    fn from(bytes: [u8; 16]) -> Self {
        TypedULID {
            id: bytes.into(),
            _type: PhantomData,
        }
    }
}

impl<T: 'static + ?Sized> From<u128> for TypedULID<T> {
    fn from(id: u128) -> Self {
        TypedULID {
            id: id.into(),
            _type: PhantomData,
        }
    }
}

impl<T: 'static + ?Sized> From<(u64, u64)> for TypedULID<T> {
    fn from(id: (u64, u64)) -> Self {
        TypedULID {
            id: id.into(),
            _type: PhantomData,
        }
    }
}

impl<T: 'static + ?Sized> From<ULID> for TypedULID<T> {
    fn from(id: ULID) -> Self {
        TypedULID {
            id,
            _type: PhantomData,
        }
    }
}

impl<T: 'static + ?Sized> From<TypedULID<T>> for u128 {
    fn from(ulid: TypedULID<T>) -> Self {
        ulid.id.into()
    }
}

impl<T: 'static + ?Sized> From<TypedULID<T>> for ULID {
    fn from(ulid: TypedULID<T>) -> Self {
        ulid.id
    }
}

impl<T: 'static + ?Sized> From<TypedULID<T>> for (u64, u64) {
    fn from(ulid: TypedULID<T>) -> Self {
        ulid.id.into()
    }
}

impl<T: 'static + ?Sized> FromStr for TypedULID<T> {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: Ulid = Ulid::from_str(s)?;
        Ok(TypedULID {
            id: ULID(id),
            _type: PhantomData,
        })
    }
}

/// Types of errors that can occur while trying to decode a string into a ulid_str
#[derive(Debug)]
pub enum DecodingError {
    /// The length of the parsed string does not conform to requirements.
    InvalidLength,
    /// The parsed string contains a character that is not allowed in a [crockford Base32](https://crockford.com/wrmg/base32.html) string.
    InvalidChar(char),
    /// Parsing the string overflowed the result value bits
    DataTypeOverflow,
}

impl From<rusty_ulid::crockford::DecodingError> for DecodingError {
    fn from(err: rusty_ulid::crockford::DecodingError) -> Self {
        match err {
            rusty_ulid::crockford::DecodingError::InvalidLength => DecodingError::InvalidLength,
            rusty_ulid::crockford::DecodingError::InvalidChar(c) => DecodingError::InvalidChar(c),
            rusty_ulid::crockford::DecodingError::DataTypeOverflow => {
                DecodingError::DataTypeOverflow
            }
        }
    }
}

/// Represents a generic form of TypedULID&lt;T&gt;, i.e., it's a ULID for the specified domain.
///
/// TypedULID&lt;T&gt; is a typed ULID. However, there are use cases where we want to erase the type and have
/// a generic. An example use case is tagging events with UID(s) for different domains.
///
/// TypedULID&lt;T&gt; can be converted to a [DomainULID](ulid/struct.DomainULID.html) automatically if the
/// TypedULID type T implements [HasDomain](ulid/trait.HasDomain.html).
///
/// ## Example DomainULID generator function
/// ```rust
/// # extern crate oysterpack_uid;
/// # use oysterpack_uid::*;
///
/// fn new_request_id() -> DomainULID {
///     const DOMAIN: Domain = Domain("Request");
///     DomainULID::generate(&DOMAIN)
/// }
///
/// # fn main() {
/// let request_id =  new_request_id();
/// assert_eq!(request_id.domain(), "Request");
/// # }
///
/// ```
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainULID {
    domain: String,
    id: ULID,
}

impl DomainULID {
    /// Constructs a new ULID as DomainULID
    pub fn generate(domain: &Domain) -> DomainULID {
        DomainULID {
            domain: domain.to_string(),
            id: ULID::generate(),
        }
    }

    /// Associates the Domain to the ULID
    pub fn from_ulid<T: Into<ULID>>(domain: Domain, id: T) -> DomainULID {
        DomainULID {
            domain: domain.to_string(),
            id: id.into(),
        }
    }

    /// Getter for the TypedULID domain
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the id formatted as a [ULID](https://github.com/ulid/spec), e.g., 01CAT3X5Y5G9A62FH1FA6T9GVR
    pub fn ulid(&self) -> ULID {
        self.id
    }
}

impl fmt::Display for DomainULID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.domain, self.id)
    }
}

impl<T: HasDomain> From<TypedULID<T>> for DomainULID {
    fn from(uid: TypedULID<T>) -> Self {
        DomainULID::from_ulid(T::DOMAIN, uid.id)
    }
}

/// Domain ID is used to define constants
///
/// ```rust
/// # use oysterpack_uid::*;
/// const FOO_ID: DomainId = DomainId(Domain("Foo"),1866919584682221951251731635731565689);
/// let foo_id: DomainULID = FOO_ID.as_domain_ulid();
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DomainId(pub Domain, pub u128);

impl DomainId {

    /// returns the id as a DomainULID
    pub fn as_domain_ulid(&self) -> DomainULID {
        crate::DomainULID::from_ulid(self.0, self.1)
    }

    /// returns the ID's ULID
    pub fn ulid(&self) -> ULID {
        self.1.into()
    }

    /// Domain getter
    pub fn domain(&self) -> Domain {
        self.0
    }
}

impl Into<DomainULID> for DomainId {

    fn into(self) -> DomainULID {
        self.as_domain_ulid()
    }
}

impl std::fmt::Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_domain_ulid().to_string().as_str())
    }
}

/// Models the domain used by [DomainULID](ulid/struct.DomainULID.html).
///
/// Domain(s) are static and are defined as consts.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Domain(pub &'static str);

impl Domain {
    /// Returns the domain name
    pub fn name(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl AsRef<str> for Domain {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Meant to be implemented by domain types to associate the Domain with the type.
///
///
/// ## Example
/// ```rust
/// extern crate oysterpack_uid;
/// use oysterpack_uid::*;
///
/// struct User;
///
/// impl HasDomain for User {
///     const DOMAIN: Domain = Domain("User");
/// }
///
/// type UserId = TypedULID<User>;
///
/// fn main() {
///     let id : DomainULID = UserId::generate().into();
///     assert_eq!(id.domain(), User::DOMAIN.name());
/// }
///
/// ```
pub trait HasDomain {
    /// Domain
    const DOMAIN: Domain;
}

#[cfg(test)]
mod tests;
