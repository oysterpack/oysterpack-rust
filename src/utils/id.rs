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

//! The *uid* module provides support to generate unique identifiers.

extern crate uuid;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::marker::PhantomData;
use std::cmp::Ordering;

/// Id represents an identifier for some type T
///
/// # Example
///```rust
/// struct Domain;
/// type DomainId = Id<Domain>;
/// let id = DomainId::new();
///
pub struct Id<T> {
    id: u64,
    _type: PhantomData<T>,
}

impl<T> Id<T> {
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

impl<T> fmt::Display for Id<T> {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:x} = {0}", self.id) }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool { self.id == other.id }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Id<T>) -> Option<Ordering> { Some(self.id.cmp(&other.id)) }
}

impl<T> Eq for Id<T> {}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(&other.id) }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { fmt::Display::fmt(self, f) }
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Id<T> { *self }
}


#[cfg(test)]
mod test {
    use super::*;

    struct Domain;

    type DomainId = Id<Domain>;

    #[test]
    fn test_uid_hash() {
        use std::collections::HashSet;

        let mut hashes = HashSet::new();
        for _ in 0..10000 {
            assert!(hashes.insert(DomainId::new()))
        }
    }

    #[test]
    fn test_display() {
        let id = DomainId::new();
        println!("{}", id);
    }

    #[test]
    fn test_from() {
        let id1 = DomainId::new();
        let id2 = DomainId::from(id1.get());
        assert_eq!(id1, id2);
    }
}