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

//! uid macros

/// Generates a new type for an unsigned int.
/// - it is defined as a public tuple struct
/// - documentation comments are optional.
/// - it will have the following methods:
///   - `pub fn id(&self) -> u128`
/// - it will implement the following traits:
///   - Debug,
///   - Copy, Clone,
///   - Ord, PartialOrd,
///   - Eq, PartialEq, Hash,
///   - std::fmt::Display
///   - serde::Serialize, serde::Deserialize
///     - external crates will need to import the macros from `serde`
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate oysterpack_uid;
/// extern crate serde;
///
/// op_int_type! {
///   /// Error ID
///   ErrorId
/// }
///
/// const ERR_1: ErrorId = ErrorId(1);
///
/// fn main() {
///    let err_id = ERR_1.id();
///    assert_eq!(ERR_1, ErrorId(err_id));
/// }
/// ```
#[macro_export]
macro_rules! op_int_type {
    (
        $(#[$outer:meta])*
        $Name:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct $Name(pub u128);

        impl $Name {

            /// returns the ID
            pub fn id(&self) -> u128 {
                self.0
            }
        }

        impl ::std::fmt::Display for $Name {
            /// Displays the id in lower hex format
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $Name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_u128(self.0)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $Name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {

                struct IdVisitor;

                impl<'de> ::serde::de::Visitor<'de> for IdVisitor {
                    type Value = $Name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("u128")
                    }

                    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        Ok($Name(u128::from(value)))
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        Ok($Name(u128::from(value)))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        Ok($Name(u128::from(value)))
                    }

                    #[inline]
                    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        Ok($Name(value))
                    }
                }

                deserializer.deserialize_u128(IdVisitor)
            }
        }
    };
}
