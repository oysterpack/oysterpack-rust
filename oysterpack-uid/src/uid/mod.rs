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

//! Provides a typesafe [ULID](https://github.com/ulid/spec)

use chrono::{DateTime, Utc};
use rusty_ulid::{self, Ulid};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
};

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

/// Represents a ULID for some type T.
///
/// # Examples
///
/// ## Defining an Id for a struct
/// ```rust
/// # use oysterpack_uid::Uid;
/// struct Domain;
/// type DomainId = Uid<Domain>;
/// let id = DomainId::new();
/// ```
/// ## Defining an Id for a trait
/// ```rust
/// # use oysterpack_uid::Uid;
/// trait Foo{}
/// // traits are not Send. Send is added to the type def in order to satisfy Uid type constraints
/// // in order to be able to send the Uid across threads
/// type FooId = Uid<dyn Foo + Send>;
/// let id = FooId::new();
/// ```
#[derive(Serialize, Deserialize)]
pub struct Uid<T: ?Sized> {
    id: u128,
    #[serde(skip)]
    _type: PhantomData<T>,
}

impl<T: ?Sized> Uid<T> {
    /// New Uid instances are guaranteed to be lexographically sortable if they are created within the same
    /// millisecond. In other words, Uid(s) created within the same millisecond are random.
    pub fn new() -> Uid<T> {
        Uid {
            id: Ulid::new().into(),
            _type: PhantomData,
        }
    }

    /// Creates the next strictly monotonic ULID for the given previous ULID.
    /// Returns None if the random part of the next ULID would overflow.
    pub fn next(previous: Uid<T>) -> Option<Uid<T>> {
        Ulid::next_strictly_monotonic(previous.ulid()).map(|next| Uid {
            id: next.into(),
            _type: PhantomData,
        })
    }

    /// returns the id
    pub fn id(&self) -> u128 {
        self.id
    }

    /// Returns the timestamp of this ULID as a DateTime<Utc>.
    pub fn datetime(&self) -> DateTime<Utc> {
        self.ulid().datetime()
    }

    /// Returns a new ULID with the random part incremented by one.
    /// Returns None if the new ULID overflows.
    pub fn increment(self) -> Option<Uid<T>> {
        Self::next(self)
    }

    fn ulid(&self) -> Ulid {
        Ulid::from(self.id)
    }
}

impl<T: ?Sized> fmt::Display for Uid<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let ulid: Ulid = self.ulid();
        f.write_str(&ulid.to_string())
    }
}

impl<T: ?Sized> PartialEq for Uid<T> {
    fn eq(&self, other: &Uid<T>) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized> PartialOrd for Uid<T> {
    fn partial_cmp(&self, other: &Uid<T>) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: ?Sized> Eq for Uid<T> {}

impl<T: ?Sized> Ord for Uid<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: ?Sized> Hash for Uid<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: ?Sized> Copy for Uid<T> {}

impl<T: ?Sized> Clone for Uid<T> {
    fn clone(&self) -> Uid<T> {
        *self
    }
}

impl<T: ?Sized> fmt::Debug for Uid<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.id)
    }
}

impl<T: ?Sized> From<[u8; 16]> for Uid<T> {
    fn from(bytes: [u8; 16]) -> Self {
        let ulid: Ulid = Ulid::from(bytes);
        Uid {
            id: ulid.into(),
            _type: PhantomData,
        }
    }
}

impl<T: ?Sized> From<u128> for Uid<T> {
    fn from(id: u128) -> Self {
        let ulid: Ulid = Ulid::from(id);
        Uid {
            id: ulid.into(),
            _type: PhantomData,
        }
    }
}

impl<T: ?Sized> From<(u64, u64)> for Uid<T> {
    fn from(id: (u64, u64)) -> Self {
        let ulid: Ulid = Ulid::from(id);
        Uid {
            id: ulid.into(),
            _type: PhantomData,
        }
    }
}

impl<T: ?Sized> FromStr for Uid<T> {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ulid: Ulid = Ulid::from_str(s)?;
        Ok(Uid {
            id: ulid.into(),
            _type: PhantomData,
        })
    }
}

impl<T: ?Sized> From<Uid<T>> for u128 {
    fn from(ulid: Uid<T>) -> Self {
        ulid.id
    }
}

impl<T: ?Sized> From<Uid<T>> for (u64, u64) {
    fn from(ulid: Uid<T>) -> Self {
        (
            (ulid.id >> 64) as u64,
            (ulid.id & 0xFFFF_FFFF_FFFF_FFFF) as u64,
        )
    }
}

/// Types of errors that can occur while trying to decode a string into a ulid
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
