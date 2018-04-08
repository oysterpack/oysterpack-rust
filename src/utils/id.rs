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

//! Provides support to generate unique identifiers.

extern crate uuid;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::marker::PhantomData;
use std::cmp::Ordering;

/// Id represents an identifier for some type T. Id(s) are threadsafe.
///
/// # Examples
///
/// ## Defining an Id for a struct
/// ```rust
/// # use oysterpack::Id;
/// struct Domain;
/// type DomainId = Id<Domain>;
/// let id = DomainId::new();
/// ```
/// ## Defining an Id for a trait
/// ```rust
/// # use oysterpack::Id;
/// trait Foo{}
/// // NOTE: Send + Sync must be added to the type def in order to satisfy Id's type constraints.
/// type FooId = Id<Foo + Send + Sync>;
/// let id = FooId::new();
/// ```
///
pub struct Id<T: Send + Sync + ? Sized> {
    id: u64,
    _type: PhantomData<T>,
}

impl<T: Send + Sync + ? Sized> Id<T> {
    /// Constructs a new random unique Id. The odds for collision are the same as for a version 4 UUID.
    /// The algorithm generates a version 4 UUID and then hashes it.
    pub fn new() -> Id<T> {
        let id = uuid::Uuid::new_v4();
        let mut s = DefaultHasher::new();
        Hash::hash(&id, &mut s);
        Id { id: s.finish(), _type: PhantomData }
    }

    /// Constructs a new Id from the specified id
    pub fn from(id: u64) -> Id<T> {
        Id { id: id, _type: PhantomData }
    }

    /// returns the id
    pub fn get(&self) -> u64 { self.id }
}

impl<T: Send + Sync + ? Sized> fmt::Display for Id<T> {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:x}", self.id) }
}

impl<T: Send + Sync + ? Sized> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool { self.id == other.id }
}

impl<T: Send + Sync + ? Sized> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Id<T>) -> Option<Ordering> { Some(self.id.cmp(&other.id)) }
}

impl<T: Send + Sync + ? Sized> Eq for Id<T> {}

impl<T: Send + Sync + ? Sized> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(&other.id) }
}

impl<T: Send + Sync + ? Sized> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl<T: Send + Sync + ? Sized> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { fmt::Display::fmt(self, f) }
}

impl<T: Send + Sync + ? Sized> Copy for Id<T> {}

impl<T: Send + Sync + ? Sized> Clone for Id<T> {
    fn clone(&self) -> Id<T> { *self }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Unique;

    type Uid = Id<Unique>;

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
    fn uid_is_thread_safe() {
        use std::thread;

        let id = Uid::new();
        let t = thread::spawn(move || {
            id
        });

        assert!(t.join().unwrap() == id);
    }

    #[test]
    fn ids_are_threadsafe() {
        use std::thread;

        trait Foo {}
        // traits are not Send or Sync. Send + Sync are added to the type def in order to satisfy Id type constraints.
        type FooId = Id<Foo + Send + Sync>;

        let id = FooId::new();
        let t = thread::spawn(move || {
            id.get()
        });

        // id is still usable here because it implements Copy. The id was copied into the thread
        assert!(t.join().unwrap() == id.get());
    }
}