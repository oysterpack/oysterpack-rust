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

/// Defines a new public struct for unique identifiers that need to be defined as constants.
/// Documentation comments are optional.
///
/// `op_const_id!{EventId}` will generate a public struct named `EventId`.
/// - it is defined as a tuple struct
/// - It will have the following methods:
///   - `pub fn id(&self) -> u128`
/// - It will implement the following traits:
///   - Debug,
///   - Copy, Clone,
///   - Ord, PartialOrd,
///   - Eq, PartialEq, Hash,
///   - std::fmt::Display
///   - serde::Serialize, serde::Deserialize
///     - external crates will need to import the macros from `serde_derive`
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate oysterpack_core;
/// #[macro_use]
/// extern crate serde_derive;
///
/// op_const_id! {
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
macro_rules! op_const_id {
    (
        $(#[$outer:meta])*
        $UidName:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
