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

/// macro that generates a new type for a String
macro_rules! op_tuple_struct_string {
    (
        $(#[$outer:meta])*
        $name:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name (String);

        impl $name {
            /// TargetTriple constructor
            pub fn new(value: &str) -> $name {
                $name(value.to_string())
            }

            /// get the underlying value
            pub fn get(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

/// macro that generates a new type where the underlying value implements Copy
macro_rules! op_tuple_struct_copy {
    (
        $(#[$outer:meta])*
        $name:ident($T:ty)
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name ($T);

        impl $name {
            /// TargetTriple constructor
            pub fn new(value: $T) -> $name {
                $name(value)
            }

            /// get the underlying value
            pub fn get(&self) -> $T {
                self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}
