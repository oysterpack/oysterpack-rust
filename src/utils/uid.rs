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


/// Uid represents a unique identifier
#[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub struct Uid(u64);

impl Uid {
    /// Constructs a new random Uid
    pub fn new() -> Uid {
        let id = uuid::Uuid::new_v4();
        let id = id.simple().to_string();
        let mut s = DefaultHasher::new();
        Hash::hash(&id, &mut s);
        Uid(s.finish())
    }

    /// Constructs a new Uid from the specified id
    pub fn from(id: u64) -> Uid {
        Uid(id)
    }
}

impl fmt::Display for Uid {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uid_hash() {
        use std::collections::HashSet;

        let mut hashes = HashSet::new();
        for _ in 0..10000 {
            assert!(hashes.insert(Uid::new()))
        }
    }

    #[test]
    fn test_display() {
        let id = Uid::new();
        println!("{}", id);
    }

    #[test]
    fn test_from() {
        use std::string::ToString;
        let id1 = Uid::new();
        let id2 = Uid::from(id1.0);
        assert_eq!(id1, id2);
    }
}