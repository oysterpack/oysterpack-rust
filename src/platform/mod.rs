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
extern crate semver;

use std::fmt;
use std::hash::{Hash, Hasher};

use ::utils::id::Id;

/// Domain is used to group a set of Applications underneath it.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Domain {
    id: DomainId,
    name: DomainName,
}

impl Hash for Domain {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Domain {
    /// DomainID getter
    pub fn id(&self) -> DomainId { self.id }

    /// DomainName getter
    pub fn name(&self) -> &DomainName { &self.name }
}

/// DomainId is the unique identifier for the Domain
pub type DomainId = Id<Domain>;

/// DomainName is the unique name for the Domain.
/// The DomainName has constraints - see [Domain::new()](struct.DomainName.html#method.new)
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    pub fn new(name: &str) -> Result<DomainName, NameError> {
        validate_name(name).map(|name| DomainName(name))
    }

    /// returns the Domain name
    pub fn get(&self) -> &str { &self.0 }
}

fn validate_name(name: &str) -> Result<String, NameError> {
    lazy_static! {
            static ref RE : regex::Regex = regex::Regex::new(r"^[a-z][\w\-]{2,63}$").unwrap();
            static ref STARTS_WITH_ALPHA : regex::Regex = regex::Regex::new(r"^[a-z].*$").unwrap();
        }

    let name = name.trim().to_lowercase();
    if RE.is_match(&name) {
        return Ok(name);
    }
    match name {
        ref name if name.len() < 3 => Err(NameError::TooShort { name: name.to_string(), len: name.len() }),
        ref name if name.len() > 64 => Err(NameError::TooLong { name: name.to_string(), len: name.len() }),
        ref name if !STARTS_WITH_ALPHA.is_match(name) => Err(NameError::StartsWithNonAlpha { name: name.to_string() }),
        name => Err(NameError::Invalid { name })
    }
}

/// App represents an application binary release.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct App {
    // Unique app identifier
    id: AppId,
    // Unique app name
    name: AppName,
    // App version
    version: semver::Version,
}

impl App {
    /// AppId getter
    pub fn id(&self) -> AppId { self.id }

    /// AppName getter
    pub fn name(&self) -> &AppName { &self.name }

    /// Version getter
    pub fn version(&self) -> &semver::Version { &self.version }
}

impl Hash for App {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// App unique identifer
pub type AppId = Id<App>;

/// App unique name
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AppName(String);

impl AppName {
    /// AppName constructor. The name will be trimmed and lowercased before it is validated.
    ///
    /// The name is checked against the following regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    /// - min length = 3
    /// - max length = 64
    /// - only word characters (alphanumeric or “_”) or "-" are allowed
    /// - must start with an alpha char
    pub fn new(name: &str) -> Result<AppName, NameError> {
        validate_name(name).map(|name| AppName(name))
    }

    /// returns the name
    pub fn get(&self) -> &str { &self.0 }
}

impl fmt::Display for AppName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Represents a Domain-App relationship. An app can be deployed into multiple domains.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DomainApp {
    domain: Domain,
    app: App,
}

/// Actors define application functionality.
/// Actors are versioned.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Actor {
    // Unique Actor identifier
    id: ActorId,
    // Unique Actor name
    name: ActorName,
    // Actor version
    version: semver::Version,
    // the Actor crate library
    library: Library,
}

impl Actor {
    /// AppId getter
    pub fn id(&self) -> ActorId { self.id }

    /// AppName getter
    pub fn name(&self) -> &ActorName { &self.name }

    /// Version getter
    pub fn version(&self) -> &semver::Version { &self.version }

    /// The actor crate library
    pub fn library(&self) -> &Library { &self.library }
}

impl Hash for Actor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// Actor unique identifer
pub type ActorId = Id<Actor>;

/// Actor unique name
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActorName(String);

impl ActorName {
    /// ActorName constructor. The name will be trimmed and lowercased before it is validated.
    ///
    /// The name is checked against the following regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    /// - min length = 3
    /// - max length = 64
    /// - only word characters (alphanumeric or “_”) or "-" are allowed
    /// - must start with an alpha char
    pub fn new(name: &str) -> Result<ActorName, NameError> {
        validate_name(name).map(|name| ActorName(name))
    }

    /// returns the name
    pub fn get(&self) -> &str { &self.0 }
}

impl fmt::Display for ActorName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Represents an App-Actor relationship.
/// Applications are composed of actors.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AppActor {
    app: App,
    actor: Actor,
}

/// Represents a crate published on [crates.io](https://crates.io/)
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Library {
    /// Crate library name
    name: LibraryName,
    /// Crate library version
    version: semver::Version,
}

impl Library {
    /// name getter
    pub fn name(&self) -> &LibraryName { &self.name }

    /// version getter
    pub fn version(&self) -> &semver::Version { &self.version }
}

/// Actor unique name
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LibraryName(String);

impl LibraryName {
    /// ActorName constructor. The name will be trimmed and lowercased before it is validated.
    ///
    /// The name is checked against the following regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    /// - min length = 3
    /// - max length = 64
    /// - only word characters (alphanumeric or “_”) or "-" are allowed
    /// - must start with an alpha char
    pub fn new(name: &str) -> Result<ActorName, NameError> {
        validate_name(name).map(|name| ActorName(name))
    }

    /// returns the name
    pub fn get(&self) -> &str { &self.0 }
}

impl fmt::Display for LibraryName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}


/// Name validation errors
#[derive(Fail, Debug)]
pub enum NameError {
    /// Name min length is 3
    #[fail(display = "Name min length is 3 : [{}] length = {}", name, len)]
    TooShort {
        /// name
        name: String,
        /// name length
        len: usize,
    },
    /// Name max length is 64
    #[fail(display = "Name max length is 64 : [{}] length = {}", name, len)]
    TooLong {
        /// name
        name: String,
        /// name length
        len: usize,
    },
    /// Name must start with an alpha char
    #[fail(display = "Name must start with an alpha char : [{}]", name)]
    StartsWithNonAlpha {
        /// name
        name: String
    },
    /// Name must match against regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    #[fail(display = "Name is invalid. It must start with an alpha and the rest can only conist of alphanumeric, '_', or '-' : [{}]", name)]
    Invalid {
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
        if let Err(err @ NameError::TooShort { .. }) = DomainName::new("       ") {
            assert!(format!("{}", err).starts_with("Name min length is "));
        } else {
            panic!("NameError::TooShort error should have been returned")
        }
    }

    #[test]
    fn name_too_short() {
        if let Err(err @ NameError::TooShort { .. }) = DomainName::new("   12   ") {
            assert!(format!("{}", err).starts_with("Name min length is "));
            if let NameError::TooShort { len, .. } = err {
                assert_eq!(2, len);
            }
        } else {
            panic!("NameError::NameTooShort error should have been returned")
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

        if let Err(err @ NameError::TooLong { .. }) = DomainName::new(&name.0) {
            assert!(format!("{}", err).starts_with("Name max length is "));
            if let NameError::TooLong { len, .. } = err {
                assert_eq!(65, len);
            }
        } else {
            panic!("NameError::TooLong error should have been returned")
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
            Err(NameError::Invalid { name }) => assert_eq!("ab c", &name),
            other => panic!("NameError::Invalid error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("-abc") {
            Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("-abc", &name),
            other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("_abc") {
            Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("_abc", &name),
            other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }

        match DomainName::new("1abc") {
            Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("1abc", &name),
            other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
        }
    }
}

