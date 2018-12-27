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

//! Provides the [ULID](https://github.com/ulid/spec) functionality.

use byteorder::ByteOrder;
use chrono::{DateTime, Utc};
use failure::Fail;
use rusty_ulid::{self, Ulid};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, str::FromStr};

pub(crate) mod domain;

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

/// Converts a ULID u128 representation into a String
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

    /// encodes itself as bytes in big-endian order, i.e., network byte order
    pub fn to_bytes(&self) -> [u8; 16] {
        let (left, right): (u64, u64) = self.0.into();
        let id = [left, right];
        let mut bytes: [u8; 16] = [0; 16];
        byteorder::NetworkEndian::write_u64_into(&id, &mut bytes);
        bytes
    }

    /// Returns a new ULID with the random part incremented by one.
    /// Overflowing the random part generates a new ULID, i.e., with a new timestamp portion.
    ///
    /// ## Use Cases
    /// 1. In case of collision, use increment as a cheap method to generate a new ULID
    /// 2. An alternative cheaper (faster) way to generate ULID(s), with the following trade offs:
    ///    - ULID(s) will be sequential until they overflow - depending on the use case, that may be
    ///      and advantage or disadvantage.
    ///    - higher probablity of collision when multiple ULIDs have the same exact timestamp
    ///      component are used to produce new ULID(s) via incrementing a base ULID. The longer the ULID(s)
    ///      are incremented, the higher the chance that they will collide with other ULID(s). Thus, it
    ///      depends on the use case and context.
    ///
    /// ## Notes
    /// Based on benchmarks, producing a new ULID via `increment` is about 10x faster vs `generate`.
    pub fn increment(self) -> ULID {
        let prev = self.0;
        let ulid = self.0.increment();
        if ulid < prev {
            Self::generate()
        } else {
            ULID(ulid)
        }
    }
}

impl fmt::Display for ULID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
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

impl From<[u8; 16]> for ULID {
    fn from(id: [u8; 16]) -> Self {
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
        let ulid: u128 = (*self).into();
        serializer.serialize_u128(ulid)
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
                formatter.write_str("ULID as u128")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID(Ulid::from(u128::from(value))))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ULID::from(value))
            }
        }

        deserializer.deserialize_u128(ULIDVisitor)
    }
}

/// Types of errors that can occur while trying to decode a string into a ulid_str
#[derive(Debug, Fail)]
pub enum DecodingError {
    #[fail(display = "invalid length")]
    /// The length of the parsed string does not conform to requirements.
    InvalidLength,
    /// The parsed string contains a character that is not allowed in a [crockford Base32](https://crockford.com/wrmg/base32.html) string.
    #[fail(display = "invalid char: '{}'", _0)]
    InvalidChar(char),
    /// Parsing the string overflowed the result value bits
    #[fail(display = "overflow")]
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

#[cfg_attr(tarpaulin, skip)]
#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use serde_json;
    use std::{cmp::Ordering, str::FromStr};

    #[test]
    fn test_uid_hash_uniqueness() {
        use std::collections::HashSet;
        let count = 100000;

        let mut hashes = HashSet::new();
        for _ in 0..count {
            assert!(hashes.insert(ULID::generate()))
        }
    }

    #[test]
    fn uid_str() {
        let id = ULID::generate();
        let id_str = id.to_string();
        let id2 = ULID::from_str(&id_str).unwrap();
        assert_eq!(id, id2);

        let (ulid_str_part_1, ulid_str_part_2) = id_str.split_at(id_str.len() / 2);
        match ULID::from_str(ulid_str_part_1) {
            Ok(_) => panic!("Should have failed"),
            Err(err @ DecodingError::InvalidLength) => println!("Invalid ULID: {}", err),
            Err(err) => panic!("Failed because of some other reason: {}", err),
        }

        // Crockford's Base32 encoding (https://crockford.com/wrmg/base32.html) is used. This alphabet excludes the letters I, L, O, and U to avoid confusion and abuse.
        // When decoding, upper and lower case letters are accepted, and i and l will be treated as 1 and o will be treated as 0.
        let mut ulid = crate::ulid_str();
        ulid.remove(ulid.len() - 1);
        ulid.insert(ulid.len(), 'U');
        println!("invalid ulid: {}", ulid);
        match ULID::from_str(&ulid) {
            Ok(ulid) => panic!("Should have failed: {}", ulid),
            Err(err @ DecodingError::InvalidChar(_)) => println!("Invalid ULID: {}", err),
            Err(err) => panic!("Failed because of some other reason: {}", err),
        }

        ulid.remove(ulid.len() - 1);
        ulid.insert(ulid.len(), 'I');
        println!("invalid ulid: {}", ulid);
        assert!(ULID::from_str(&ulid).unwrap().to_string().ends_with("1"));

        ulid.remove(ulid.len() - 1);
        ulid.insert(ulid.len(), 'L');
        println!("invalid ulid: {}", ulid);
        assert!(ULID::from_str(&ulid).unwrap().to_string().ends_with("1"));

        ulid.remove(ulid.len() - 1);
        ulid.insert(ulid.len(), 'O');
        println!("invalid ulid: {}", ulid);
        assert!(ULID::from_str(&ulid).unwrap().to_string().ends_with("0"));

        // Technically, a 26-character Base32 encoded string can contain 130 bits of information,
        // whereas a ULID must only contain 128 bits. Therefore, the largest valid ULID encoded in
        // Base32 is 7ZZZZZZZZZZZZZZZZZZZZZZZZZ, which corresponds to an epoch time of 281474976710655 or 2 ^ 48 - 1.
        match ULID::from_str("8ZZZZZZZZZZZZZZZZZZZZZZZZZ") {
            Ok(_) => panic!("Should have failed"),
            Err(err @ DecodingError::DataTypeOverflow) => println!("Invalid ULID: {}", err),
            Err(err) => panic!("Failed because of some other reason: {}", err),
        }
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
        let id = ULID::generate();
        let id_u128: u128 = id.into();
        assert_eq!(id, ULID::from(id_u128));
    }

    #[test]
    fn uid_ordered() {
        use std::thread;
        let mut id = ULID::generate();
        for _ in 0..10 {
            thread::sleep_ms(1);
            let temp = ULID::generate();
            assert!(temp > id);
            id = temp;
        }
    }

    #[test]
    fn uid_next() {
        let mut id = ULID::generate();
        for _ in 0..1000 {
            let temp = id.clone().increment();
            assert!(temp > id);
            id = temp;
        }
    }

    #[test]
    fn uid_is_thread_safe() {
        use std::thread;

        let id = ULID::generate();
        let t = thread::spawn(move || id);
        assert!(t.join().unwrap() == id);
    }

    #[test]
    fn uid_serde() {
        let id = ULID::generate();
        let id_bytes = bincode::serialize(&id).unwrap();
        assert_eq!(
            id_bytes.len(),
            16,
            "ULID should be serialized as 128 bits, i.e., 16 bytes * 8 = 128"
        );
        let id_u128: u128 = id.into();
        println!("({}) bytes.len = {}", id_u128, id_bytes.len());
        let id2: ULID = bincode::deserialize(&id_bytes).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn ulid_functions() {
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
        let id_1: ULID = ULID::generate();
        let id_2: ULID = ULID::generate();
        assert!(id_1 != id_2);

        let id_bytes = bincode::serialize(&id_2).unwrap();
        let id_3: ULID = bincode::deserialize(&id_bytes).unwrap();
        assert_eq!(id_3, id_2);
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
