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

//! # Generic type safe numeric identifiers
//! - ids are associated with a type
//! - provides support to generate unique numeric identifiers.
//! - are serializable, i.e., [Serde](https://docs.rs/serde) compatible
//! - are threadsafe

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_id/0.1.0")]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::marker::PhantomData;
use std::cmp::Ordering;

/// Id represents an identifier for some type T.
///
/// # Examples
///
/// ## Defining an Id for a struct
/// ```rust
/// # use oysterpack_id::Id;
/// struct Domain;
/// type DomainId = Id<Domain>;
/// let id = DomainId::new();
/// ```
/// ## Defining an Id for a trait
/// ```rust
/// # use oysterpack_id::Id;
/// trait Foo{}
/// type FooId = Id<Foo>;
/// let id = FooId::new();
/// ```
#[derive(Serialize, Deserialize)]
pub struct Id<T: ?Sized> {
    id: u64,
    #[serde(skip)]
    _type: PhantomData<T>,
}

impl<T: ?Sized> Id<T> {
    /// Constructs a new random unique Id. The odds for collision are the same as for a version 4 UUID.
    /// The algorithm generates a version 4 UUID and then hashes it.
    pub fn new() -> Id<T> {
        let id = uuid::Uuid::new_v4();
        let mut s = DefaultHasher::new();
        Hash::hash(&id, &mut s);
        Id {
            id: s.finish(),
            _type: PhantomData,
        }
    }

    /// Constructs a new Id from the specified id
    pub fn from(id: u64) -> Id<T> {
        Id {
            id: id,
            _type: PhantomData,
        }
    }

    /// returns the id
    pub fn get(&self) -> u64 {
        self.id
    }
}

impl<T: ?Sized> fmt::Display for Id<T> {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.id)
    }
}

impl<T: ?Sized> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Id<T>) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T: ?Sized> Eq for Id<T> {}

impl<T: ?Sized> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: ?Sized> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: ?Sized> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl<T: ?Sized> Copy for Id<T> {}

impl<T: ?Sized> Clone for Id<T> {
    fn clone(&self) -> Id<T> {
        *self
    }
}

#[cfg(test)]
mod test {
    extern crate bincode;
    extern crate rmp_serde as rmps;
    extern crate serde_cbor;
    extern crate serde_json;

    use super::*;

    struct Unique;
    type Uid = Id<Unique>;

    // how to define an Id for a trait
    trait Foo {}
    type FooId = Id<Foo + Send>;

    // New Ids should be unique
    #[test]
    fn id_hash_uniqueness() {
        use std::collections::HashSet;

        let mut hashes = HashSet::new();
        for _ in 0..100000 {
            assert!(hashes.insert(Uid::new()))
        }
    }

    #[test]
    fn trait_id() {
        let id = FooId::new();
        println!("FooId = {}", id);
    }

    #[test]
    fn display() {
        let id = Uid::new();
        println!("{}", id);
    }

    #[test]
    fn from() {
        let id1 = Uid::new();
        let id2 = Uid::from(id1.get());
        assert_eq!(id1, id2);
    }

    #[test]
    fn to_string() {
        let id = Uid::new();
        assert_eq!(id.to_string(), format!("{}", id));
    }

    #[test]
    fn ordered() {
        let mut ids = vec![Uid::new(), Uid::new(), Uid::new(), Uid::new()];
        ids.sort();
        println!("{:?}", ids);
        let mut id1 = ids.first().unwrap();
        for id2 in ids.iter().skip(1) {
            assert!(id2 > id1);
            id1 = id2;
        }
    }

    #[test]
    fn uid_is_thread_safe() {
        use std::thread;

        let id = Uid::new();
        let t = thread::spawn(move || id);

        assert!(t.join().unwrap() == id);
    }

    #[test]
    fn ids_are_threadsafe() {
        use std::thread;

        trait Foo {}
        // traits are not Send. Send is added to the type def in order to satisfy Id type constraints.
        type FooId = Id<Foo + Send>;

        let id = FooId::new();
        let t = thread::spawn(move || id.get());

        // id is still usable here because it implements Copy. The id was copied into the thread
        assert!(t.join().unwrap() == id.get());
    }

    #[test]
    fn serialization_json() {
        let uid = Uid::new();

        let json = serde_json::to_string(&uid).unwrap();

        println!("{} : {}", &json, json.len());
        let _uid: Uid = serde_json::from_str(&json).expect("JSON deserialization failed");
    }

    #[test]
    fn serialization_bincode() {
        let uid = Uid::new();

        let bytes = bincode::serialize(&uid).unwrap();
        println!("bincode bytes.len() = {}", bytes.len());
        let _uid: Uid = bincode::deserialize(&bytes).expect("bincode deserialization failed");
    }

    #[test]
    fn serialization_cbor() {
        let uid = Uid::new();

        let bytes = serde_cbor::to_vec(&uid).unwrap();
        println!("CBOR bytes.len() = {}", bytes.len());
        let _uid: Uid = serde_cbor::from_slice(&bytes).expect("CBOR deserialization failed");
    }

    #[test]
    fn serialization_msgpack() {
        let uid = Uid::new();

        let bytes = rmps::to_vec(&uid).unwrap();
        println!("rmps bytes.len() = {}", bytes.len());
        let _uid: Uid = rmps::from_slice(&bytes).expect("rmps deserialization failed");
    }

    // Serialization results
    //    bincode bytes.len() = 8
    //    rmps bytes.len() = 10
    //    CBOR bytes.len() = 13
    //    JSON len : 27
    //
    //    bincode is the most efficient in terms of size
}
