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

//! Crate dependency domain model

use std::fmt;

/// Represents the kind of dependency
#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Debug, Copy, Serialize, Deserialize)]
pub enum Kind {
    /// Normal compile time dependency
    Normal,
    /// Dependency is used for testing purposes
    Development,
    /// Dependency is used at build time
    Build,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label = match *self {
            Kind::Normal => "Normal",
            Kind::Development => "Development",
            Kind::Build => "Build"
        };
        f.write_str(label)
    }
}

