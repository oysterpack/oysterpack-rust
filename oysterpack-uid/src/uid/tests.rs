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

//! unit tests
#![allow(warnings)]

use super::Uid;
use std::{cmp::Ordering, str::FromStr};
use tests::run_test;

struct O;

type Oid = Uid<O>;

trait Foo {}

type FooId = Uid<dyn Foo + Send>;

// New Ids should be unique
#[test]
fn uid_hash_uniqueness() {
    run_test(|| {
        use std::collections::HashSet;
        let count = 100000;
        info!("id_hash_uniqueness: {}", count);

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(Oid::new()))
        }
    });
}

#[test]
fn uid_str() {
    run_test(|| {
        let id = FooId::new();
        let id_str = id.to_string();
        info!("uid_str: {}", id_str);
        let id2 = FooId::from_str(&id_str).unwrap();
        assert_eq!(id, id2);
    });
}

#[test]
fn uid_eq() {
    let id = FooId::new();
    let id_u128: u128 = id.into();
    assert_eq!(id, Uid::from(id_u128));
}

#[test]
fn uid_ordered() {
    use std::thread;
    let mut id = FooId::new();
    for _ in 0..10 {
        thread::sleep_ms(1);
        let temp = FooId::new();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_next() {
    let mut id = FooId::new();
    for _ in 0..1000 {
        let temp = id.clone().increment().unwrap();
        assert!(temp > id);
        id = temp;
    }
}

#[test]
fn uid_is_thread_safe() {
    use std::thread;

    let id = FooId::new();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);

    let id = Oid::new();
    let t = thread::spawn(move || id);
    assert!(t.join().unwrap() == id);
}

pub mod Errors {
    /// Typsafe Id
    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
    pub struct Id(u128);

    impl Id {
        /// returns the id
        pub fn id(&self) -> u128 {
            self.0
        }
    }

    impl ::std::fmt::Display for Id {
        /// Displays the id in lower hex format
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f,"{}",self.0)
        }
    }

    impl From<u128> for Id {
        fn from(id: u128) -> Self {
            Id(id)
        }
    }

    // ID_1
    pub const ID_1: Id = Id(1);
    //
    pub const ID_2: Id = Id(2);
    pub const ID_3: Id = Id(3);

    pub enum Ids {
        ID_1,
        ID_2,
        ID_3
    }

    impl Ids {
        pub fn id(&self) -> Id {
            match self {
                Ids::ID_1 => ID_1,
                Ids::ID_2 => ID_2,
                Ids::ID_3 => ID_3,
            }
        }
    }

    impl Ids {
        pub fn from(id: u128) -> Option<Self> {
            match id {
                1 => Some(Ids::ID_1),
                2 => Some(Ids::ID_2),
                3 => Some(Ids::ID_3),
                _ => None
            }
        }
    }


    impl ::std::fmt::Display for Ids {
        /// Displays the id in lower hex format
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            let (label, id) = match self {
                Ids::ID_1 => ("ID_1",ID_1),
                Ids::ID_2 => ("ID_2",ID_2),
                Ids::ID_3 => ("ID_3",ID_3),
            };
            write!(f, "{}({})", label, id)
        }
    }



}

//op_ids! {
//    Errors(u128) {
//        /// ID_1
//        ID_1 = 1,
//        /// ID_2
//        ID_2 = 2,
//        /// ID_3
//        ID_3 = 3
//    }
//}
