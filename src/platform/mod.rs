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

//! Defines the platform model.

extern crate chrono;

use ::utils::id::Id;
use std::fmt;


/// Domain is used to group a set of Applications underneath it.
///
pub struct Domain {
    id: DomainId,
    name: DomainName,
    created_on: chrono::DateTime<chrono::Utc>,
}

/// Domain Id
pub type DomainId = Id<Domain>;

/// Domain Name
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DomainName(String);

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl DomainName {
    /// DomainName constructor. name will be trimmed.
    ///
    /// # Panics
    /// Panics if name is blank.
    pub fn new(name: &str) -> DomainName {
        let name = name.trim();
        assert!(name.len() > 0,"name cannot be blank");
        DomainName(name.to_string())
    }

    /// returns the Domain name
    pub fn get(&self) -> &str { &self.0 }
}

/// DomainName Errors
pub enum DomainNameError {
    /// Domain names cannot be blank
    BlankName
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn domain_name_get() {
        let name = DomainName::new("OysterPack");
    }
}

