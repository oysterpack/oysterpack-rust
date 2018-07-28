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

//! Provides support to generate universally unique identifiers.

use rusty_ulid::{new_ulid_string, Ulid};

/// Defines a new public struct for a universally unique identifier that is randomly generated.
/// Documentation comments are optional.
/// - `oysterpack_core::uid` needs to be in scope for this macro to function
#[macro_export]
macro_rules! uid {
    (
        $(#[$outer:meta])*
        $UidName:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct $UidName(u128);

        impl $UidName {
            /// constructs a new universally unique identifier
            pub fn new() -> $UidName {
                $UidName(::uid())
            }

            /// returns the id
            pub fn id(&self) -> u128 {
                self.0
            }
        }

        impl ::std::fmt::Display for $UidName {
            /// Displays the id in lower hex format
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{:x}", self.0)
            }
        }
    };
}

/// Defines a new public struct for unique identifiers that need to be defined as constants.
/// Documentation comments are optional.
/// - `oysterpack_core::uid` needs to be in scope for this macro to function
#[macro_export]
macro_rules! uid_const {
    (
        $(#[$outer:meta])*
        $UidName:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct $UidName(pub u128);

        impl $UidName {

            /// returns the id
            pub fn id(&self) -> u128 {
                self.0
            }
        }

        impl ::std::fmt::Display for $UidName {
            /// Displays the id in lower hex format
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{:x}", self.0)
            }
        }
    };
}

/// Returns a universally unique ddentifier
pub fn uid() -> u128 {
    Ulid::new().into()
}

/// Returns a universally unique lexicographically sortable identifier :
/// - 128-bit compatibility with UUID
/// - 1.21e+24 unique ULIDs per millisecond
/// - Lexicographically sortable!
/// - Canonically encoded as a 26 character string, as opposed to the 36 character UUID
/// - Uses Crockford's base32 for better efficiency and readability (5 bits per character)
/// - Case insensitive
/// - No special characters (URL safe)
pub fn ulid() -> String {
    new_ulid_string()
}
