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

//! Provides support for compiler enforced typed ULID(s)

use crate::{DecodingError, ULID};

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

    /// returns the ulid
    pub fn ulid(&self) -> ULID {
        self.id
    }

    /// Returns a new ULID with the random part incremented by one.
    /// Overflowing the random part generates a new TypedULID, i.e., with a new timestamp portion.
    pub fn increment(self) -> TypedULID<T> {
        TypedULID {
            id: self.id.increment(),
            _type: PhantomData,
        }
    }
}

impl<T: 'static> Serialize for TypedULID<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ulid: u128 = (*self).into();
        serializer.serialize_u128(ulid)
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
                formatter.write_str("TypedULID as u128")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(TypedULID::from(u128::from(value)))
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(TypedULID::from(u128::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(TypedULID::from(u128::from(value)))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(TypedULID::from(value))
            }
        }

        deserializer.deserialize_u128(UidVisitor(PhantomData))
    }
}

impl<T: 'static + ?Sized> fmt::Display for TypedULID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.id.fmt(f)
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
            id: ULID::from_ulid(id),
            _type: PhantomData,
        })
    }
}

#[cfg_attr(tarpaulin, skip)]
#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::run_test;
    use crate::*;
    use serde_json;
    use std::{cmp::Ordering, str::FromStr};

    #[derive(Debug)]
    struct User;

    impl HasDomain for User {
        const DOMAIN: Domain = Domain("User");
    }

    type UserId = TypedULID<User>;

    trait Foo {}

    type FooId = TypedULID<dyn Foo + Send + Sync>;

    // New Ids should be unique
    #[test]
    fn test_uid_hash_uniqueness() {
        use std::collections::HashSet;
        let count = 100000;

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(UserId::generate()))
        }
    }

    #[test]
    fn uid_str() {
        run_test("uid_str", || {
            let id = FooId::generate();
            let id_str = id.to_string();
            info!("uid_str: {}", id_str);
            let id2 = FooId::from_str(&id_str).unwrap();
            assert_eq!(id, id2);
        });
    }

    #[test]
    fn ulid_str_u128_conversions() {
        let ulid_str = crate::ulid_str();
        let ulid_128 = crate::ulid_str_into_u128(&ulid_str).unwrap();
        let ulid = ULID::from_str(&ulid_str).unwrap();
        assert_eq!(ulid.to_string(), ulid_str);
        assert_eq!(ulid_128, ulid.into());

        let ulid_128 = crate::ulid_u128();
        let ulid_str = crate::ulid_u128_into_string(ulid_128);
        assert_eq!(ulid_128, crate::ulid_str_into_u128(&ulid_str).unwrap());
    }

    #[test]
    fn uid_eq() {
        let id = FooId::generate();
        let id_u128: u128 = id.into();
        assert_eq!(id, TypedULID::from(id_u128));
    }

    #[test]
    fn uid_ordered() {
        use std::thread;
        let mut id = FooId::generate();
        for _ in 0..10 {
            thread::sleep_ms(1);
            let temp = FooId::generate();
            assert!(temp > id);
            id = temp;
        }
    }

    #[test]
    fn uid_next() {
        let mut id = FooId::generate();
        for _ in 0..1000 {
            let temp = id.clone().increment();
            assert!(temp > id);
            id = temp;
        }
    }

    #[test]
    fn uid_is_thread_safe() {
        use std::thread;

        let id = FooId::generate();
        let t = thread::spawn(move || id);
        assert!(t.join().unwrap() == id);

        let id = UserId::generate();
        let t = thread::spawn(move || id);
        assert!(t.join().unwrap() == id);
    }

    #[test]
    fn typed_ulid_serde() {
        pub struct Foo;
        let id = TypedULID::<Foo>::generate();
        let id_bytes = bincode::serialize(&id).unwrap();
        let id2: TypedULID<Foo> = bincode::deserialize(&id_bytes).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn ulid_functions() {
        run_test("ulid_functions", || {
            use std::collections::HashSet;
            let count = 100000;

            let mut hashes = HashSet::new();
            for _ in 0..count {
                assert!(hashes.insert(ulid_str()))
            }

            for uid in hashes {
                let uid_u128 = ulid_str_into_u128(&uid).unwrap();
                let uid2 = ulid_u128_into_string(uid_u128);
                assert_eq!(uid, uid2);
            }

            assert!(ulid_str_into_u128("INVALID").is_err());
        });
    }

    #[test]
    fn domain_uid() {
        run_test("domain_uid", || {
            let id: DomainULID = UserId::generate().into();
            assert_eq!(id.domain(), User::DOMAIN.name());
            info!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
            info!("{:?} => {}", id, id);
            const DOMAIN_FOO: Domain = Domain("Foo");
            let id = FooId::generate();
            let id: DomainULID = DomainULID::from_ulid(DOMAIN_FOO, id.ulid());
            info!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
            let id = DomainULID::from_ulid(User::DOMAIN, ULID::generate());
        })
    }

    #[test]
    fn ulid_increment() {
        let mut ulid = super::ULID::generate();
        for _ in 0..1_000_000 {
            let prev = ulid;
            ulid = ulid.increment();
            assert!(ulid > prev);
        }

        let previous_ulid = ULID::from(0x0000_0000_0000_FFFF_FFFF_FFFF_FFFF_FFFF);
        let (datetime_1, n_1): (u64, u64) = previous_ulid.into();
        println!("(datetime_1, n_1) = ({}, {})", datetime_1, n_1);
        let next = previous_ulid.increment();
        let (datetime_2, n_2): (u64, u64) = next.into();
        println!("(datetime_2, n_2) = ({}, {})", datetime_2, n_2);
        assert_ne!(datetime_2, datetime_1);
        assert_ne!(n_2, 0);
        assert_ne!(n_2, n_1);

        let ulid = ULID::generate();
        let ulid2 = ulid.increment();
        assert_eq!(ulid.datetime(), ulid2.datetime());
    }

    #[test]
    fn ulid_generate() {
        run_test("ULID", || {
            let id_1: ULID = ULID::generate();
            let id_2: ULID = ULID::generate();
            assert!(id_1 != id_2);

            let id_bytes = bincode::serialize(&id_2).unwrap();
            let id_3: ULID = bincode::deserialize(&id_bytes).unwrap();
            assert_eq!(id_3, id_2);

            let foo_id = FooId::generate();
            let foo_ulid: ULID = foo_id.into();
            assert_eq!(foo_id.ulid(), foo_ulid);
        });
    }

    #[test]
    fn domain() {
        run_test("domain", || {
            const USERS: Domain = Domain("users");
            assert_eq!(USERS.as_ref(), "users");
            assert_eq!(USERS.as_ref(), USERS.name());
        });
    }

    #[test]
    fn const_ulid() {
        op_ulid! { FooId }
        const FOO_ID: FooId = FooId(1866907549525959787301297812082244355);
        let id = FOO_ID.to_string();
        let ulid: ULID = id.parse().unwrap();
        assert_eq!(ulid, FOO_ID.ulid());
        let foo_ulid: ULID = FOO_ID.into();
        assert_eq!(ulid, foo_ulid);
        run_test("const_ulid", || {
            info!("{}", serde_json::to_string(&FOO_ID).unwrap());
        })
    }

    #[test]
    fn to_bytes() {
        let ulid1 = ULID::generate();
        let bytes = ulid1.to_bytes();
        let ulid2 = ULID::from(bytes);
        println!("ulid1({}) ulid2({})", ulid1, ulid2);
        assert_eq!(ulid1, ulid2);
    }

}
