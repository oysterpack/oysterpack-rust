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

//! macros

/// Used to define ULID constants in a type safe manner.
///
/// The new type implements : Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize
///
/// For portability purposes, when the u128 ID is serialized as [u64; 2] because most serializer / deserializers
/// don't natively support u128, e.g., JSON, MessagePack.
///
/// ```rust
///  #[macro_use]
///  extern crate oysterpack_uid;
///
///  use oysterpack_uid::ULID;
///
///  op_ulid! {
///     /// Foo ID
///     pub FooId
/// }
///
///  pub const FOO_ID: FooId = FooId(1866910953065622895350834727020862173);
///
///  fn main() {
///     let ulid: ULID = FOO_ID.into();
///     let ulid_str = FOO_ID.to_string();
///     assert_eq!(ulid, ulid_str.parse::<ULID>().unwrap());
///     assert_eq!(ulid, FOO_ID.ulid());
///  }
/// ```
///
#[macro_export]
macro_rules! op_ulid {
    (
    $(#[$outer:meta])*
    $struct_vis:vis $Name:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        $struct_vis struct $Name(pub u128);

        op_tt_as_item! {
            impl From<$Name> for $crate::ULID {
                fn from(ulid: $Name) -> $crate::ULID {
                    ulid.0.into()
                }
            }
        }

        op_tt_as_item! {
            impl From<$crate::ULID> for $Name {
                fn from(ulid: $crate::ULID) -> $Name {
                    $Name(ulid.into())
                }
            }
        }

        op_tt_as_item! {
            impl From<$crate::DomainULID> for $Name {
                fn from(ulid: $crate::DomainULID) -> $Name {
                    $Name(ulid.ulid().into())
                }
            }
        }

        op_tt_as_item! {
            impl std::fmt::Display for $Name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    let ulid: $crate::ULID = self.0.into();
                    f.write_str(ulid.to_string().as_str())
                }
            }
        }

        op_tt_as_item! {
            impl $Name {
                /// returns the ID as a ULID
                pub fn ulid(&self) -> $crate::ULID {
                    self.0.into()
                }
            }
        }

        op_tt_as_item! {
            impl std::str::FromStr for $Name {
                type Err = $crate::ulid::DecodingError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    $crate::rusty_ulid::Ulid::from_str(s)
                        .map(|ulid| $Name(ulid.into()))
                        .map_err($crate::ulid::DecodingError::from)
                }
            }
        }


        op_tt_as_item! {
            impl serde::Serialize for $Name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeTuple;
                    let mut tup = serializer.serialize_tuple(2)?;
                    let (v1,v2) = ((self.0 >> 64) as u64, (self.0 & 0xFFFF_FFFF_FFFF_FFFF) as u64);
                    tup.serialize_element(&v1)?;
                    tup.serialize_element(&v2)?;
                    tup.end()
                }
            }
        }

        op_tt_as_item! {
            impl<'de> serde::Deserialize<'de> for $Name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct ULIDVisitor;

                    impl<'de> serde::de::Visitor<'de> for ULIDVisitor {
                        type Value = $Name;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(stringify!($Name))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where A: serde::de::SeqAccess<'de>{
                            let (v1, v2): (u64, u64) = (seq.next_element().unwrap().unwrap(), seq.next_element().unwrap().unwrap());
                            Ok($Name(u128::from(v1) << 64 | u128::from(v2)))
                        }
                    }

                    deserializer.deserialize_seq(ULIDVisitor)
                }
            }

        }

    };
}
