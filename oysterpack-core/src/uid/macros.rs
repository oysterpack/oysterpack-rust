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

/// Defines a new public struct for a universally unique identifier that is randomly generated.
/// Documentation comments are optional.
///
/// # Examples
///
/// ```rust
/// #[macro_use]
/// extern crate oysterpack_core;
///
/// op_id! {
///   /// EventId comments can be specified.
///   EventId
/// }
///
/// fn main() {
///    /// EventId has been defined above
///    let id = EventId::new();
///    println!("{}", id);
/// }
///
/// ```
#[macro_export]
macro_rules! op_id {
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
                $UidName($crate::uid::uid())
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
#[macro_export]
macro_rules! op_const_id {
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
