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

extern crate failure;
extern crate chrono;
extern crate lazy_static;
extern crate regex;

use std::fmt;
use failure::Fail;

use ::utils::id::Id;

/// Domain is used to group a set of Applications underneath it.
///
pub struct Domain {
    id: DomainId,
    name: DomainName,

    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: chrono::DateTime<chrono::Utc>,
    deleted_on: chrono::DateTime<chrono::Utc>,

    enabled: bool,

    parent: Option<DomainId>,
    children: Vec<DomainId>,
}

impl Domain {
    /// DomainID getter
    pub fn id(&self) -> DomainId { self.id }

    /// DomainName getter
    pub fn name(&self) -> &DomainName { &self.name }

    /// When the Domain was created
    pub fn created_on(&self) -> chrono::DateTime<chrono::Utc> { self.created_on }

    /// Is the Domain enabled
    pub fn enabled(&self) -> bool { self.enabled }

    /// Parent DomainID
    pub fn parent_id(&self) -> Option<DomainId> { self.parent }

    /// Children DomainID(s)
    pub fn children_ids(&self) -> &[DomainId] { &self.children[..] }
}

/// DomainId is the unique identifier for the Domain
pub type DomainId = Id<Domain>;

/// DomainName is the unique name for the Domain.
/// The DomainName has constraints - see [Domain::new()](struct.DomainName.html#method.new)
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DomainName(String);

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl DomainName {
    /// DomainName constructor. The name will be trimmed and lowercased before it is validated.
    ///
    /// The name is checked against the following regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    /// - min length = 3
    /// - max length = 64
    /// - only word characters (alphanumeric or “_”) or "-" are allowed
    /// - must start with an alpha char
    pub fn new(name: &str) -> Result<DomainName, DomainNameError> {
        lazy_static! {
            static ref RE : regex::Regex = regex::Regex::new(r"^[a-z][\w\-]{2,63}$").unwrap();
            static ref STARTS_WITH_ALPHA : regex::Regex = regex::Regex::new(r"^[a-z].*$").unwrap();
        }

        let name = name.trim().to_lowercase();
        if RE.is_match(&name) {
            return Ok(DomainName(name));
        }
        match name {
            ref name if name.len() < 3 => Err(DomainNameError::NameTooShort { name: name.to_string(), len: name.len() }),
            ref name if name.len() > 64 => Err(DomainNameError::NameTooLong { name: name.to_string(), len: name.len() }),
            ref name if !STARTS_WITH_ALPHA.is_match(name) => Err(DomainNameError::StartsWithNonAlpha { name: name.to_string() }),
            name => Err(DomainNameError::InvalidName { name })
        }
    }

    /// returns the Domain name
    pub fn get(&self) -> &str { &self.0 }
}


/// DomainName Errors
#[derive(Fail, Debug)]
pub enum DomainNameError {
    /// DomainName min length is 3
    #[fail(display = "DomainName min length is 3 : [{}] length = {}", name, len)]
    NameTooShort {
        /// name
        name: String,
        /// name length
        len: usize,
    },
    /// DomainName max length is 64
    #[fail(display = "DomainName max length is 64 : [{}] length = {}", name, len)]
    NameTooLong {
        /// name
        name: String,
        /// name length
        len: usize,
    },
    /// DomainName must start with an alpha char
    #[fail(display = "DomainName must start with an alpha char : [{}]", name)]
    StartsWithNonAlpha {
        /// name
        name: String
    },
    /// DomainName must match against regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    #[fail(display = "DomainName is invalid. It must start with an alpha and the rest can only conist of alphanumeric, '_', or '-' : [{}]", name)]
    InvalidName {
        /// name
        name: String
    },
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn domain_name_get() {
        let domain_name = "   OysterPack   ";
        let name = DomainName::new(domain_name).unwrap();
        assert_eq!(name.get(), &domain_name.trim().to_lowercase());
    }

    #[test]
    fn blank_domain_name() {
        if let Err(err @ DomainNameError::NameTooShort { .. }) = DomainName::new("       ") {
            assert!(format!("{}", err).starts_with("DomainName min length is "));
        } else {
            panic!("DomainNameError::BalnkName error should have been returned")
        }
    }

    #[test]
    fn name_too_short() {
        if let Err(err @ DomainNameError::NameTooShort { .. }) = DomainName::new("   12   ") {
            assert!(format!("{}", err).starts_with("DomainName min length is "));
            if let DomainNameError::NameTooShort { len, .. } = err {
                assert_eq!(2, len);
            }
        } else {
            panic!("DomainNameError::NameTooShort error should have been returned")
        }
    }

    #[test]
    fn name_too_long() {
        let name = vec!['a'; 65];
        let name = name.iter().fold(("".to_string(), 0), |mut s, c| {
            s.0.insert(s.1, *c);
            s.1 += 1;
            s
        });

        if let Err(err @ DomainNameError::NameTooLong { .. }) = DomainName::new(&name.0) {
            assert!(format!("{}", err).starts_with("DomainName max length is "));
            if let DomainNameError::NameTooLong { len, .. } = err {
                assert_eq!(65, len);
            }
        } else {
            panic!("DomainNameError::NameTooLong error should have been returned")
        }
    }

    #[test]
    fn valid_names() {
        // min length = 3
        let name = DomainName::new("aBc").unwrap();
        assert_eq!("abc", name.get());
        // alphanumeric and _ are allowed
        let name = DomainName::new("abc_DEF_123-456").unwrap();
        assert_eq!("abc_def_123-456", name.get());

        // max length = 64
        let name = vec!['a'; 64];
        let name = name.iter().fold(("".to_string(), 0), |mut s, c| {
            s.0.insert(s.1, *c);
            s.1 += 1;
            s
        });
        assert_eq!(64, name.0.len());
        DomainName::new(&name.0).unwrap();
    }

    #[test]
    fn invalid_names() {
        match DomainName::new("aB c") {
            Err(DomainNameError::InvalidName { name }) => assert_eq!("ab c", &name),
            other => panic!("DomainNameError::InvalidName error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("-abc") {
            Err(DomainNameError::StartsWithNonAlpha { name }) => assert_eq!("-abc", &name),
            other => panic!("DomainNameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("_abc") {
            Err(DomainNameError::StartsWithNonAlpha { name }) => assert_eq!("_abc", &name),
            other => panic!("DomainNameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("1abc") {
            Err(DomainNameError::StartsWithNonAlpha { name }) => assert_eq!("1abc", &name),
            other => panic!("DomainNameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }
    }
}

