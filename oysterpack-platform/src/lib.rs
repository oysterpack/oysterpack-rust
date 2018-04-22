// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # OysterPack Rust Platform Model
//!
//! ## Domain
//! - Domains form a flat namespace, i.e., there are no sub-domains.
//! - Domains are assigned a unique DomainId, which never changes and can never be re-used by any other Domain.
//! - Domain names must be unique across all domains. Domain names can be re-used after a Domain is deleted.
//! - Domains own Apps.
//!
//! ## App
//! - Apps are owned by a single domain.
//! - Apps can be transferred bewteen Domains.
//! - Apps are composed of services
//!
//! ## Service
//! - Services are public functions exposed by a library module.
//! - A service is self-contained in a single library crate.
//!
//! ## AppInstance
//! - Represents a running App instance
//!

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_platform/0.1.0")]

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate oysterpack_id;
extern crate regex;
extern crate semver;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

use semver::Version;

pub use oysterpack_id::Id;

/// Domain
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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
    /// Unique Domain ID
    pub fn id(&self) -> DomainId {
        self.id
    }

    /// Unique Domain name
    pub fn name(&self) -> &DomainName {
        &self.name
    }
}

/// DomainId is the unique identifier for the Domain
pub type DomainId = Id<Domain>;

/// DomainName is the unique name for the Domain.
/// The DomainName has constraints - see [Domain::new()](struct.DomainName.html#method.new)
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DomainName(String);

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
    pub fn get(&self) -> &str {
        &self.0
    }
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

    // provide more helpful error explaining why the name is invalid
    match name {
        ref name if name.len() < 3 => Err(NameError::TooShort {
            name: name.to_string(),
            len: name.len(),
        }),
        ref name if name.len() > 64 => Err(NameError::TooLong {
            name: name.to_string(),
            len: name.len(),
        }),
        ref name if !STARTS_WITH_ALPHA.is_match(name) => Err(NameError::StartsWithNonAlpha {
            name: name.to_string(),
        }),
        name => Err(NameError::Invalid { name }),
    }
}

/// App represents an application binary release.
/// Apps are owned by a single Domain.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct App {
    domain_id: DomainId,
    id: AppId,
    name: AppName,
    version: Version,
    services: HashSet<Service>,
}

impl App {
    /// Returns the owning Domain.
    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// Unique App ID across all domains.
    pub fn id(&self) -> AppId {
        self.id
    }

    /// Unique App name across all domains.
    pub fn name(&self) -> &AppName {
        &self.name
    }

    /// App version
    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    /// App Services
    pub fn services(&self) -> &HashSet<Service> {
        &self.services
    }
}

impl Hash for App {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// App unique identifer
pub type AppId = Id<App>;

/// App unique name
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Services define application functionality.
/// Services are versioned.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Service {
    // Unique Service identifier
    id: ServiceId,
    // Unique Service name
    name: ServiceName,
    // Service version
    version: semver::Version,
}

impl Service {
    /// ServiceId getter
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// ServiceName getter
    pub fn name(&self) -> &ServiceName {
        &self.name
    }

    /// Version getter
    pub fn version(&self) -> &semver::Version {
        &self.version
    }
}

impl Hash for Service {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// Service unique identifer
pub type ServiceId = Id<Service>;

/// Service unique name
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ServiceName(String);

impl ServiceName {
    /// ServiceName constructor. The name will be trimmed and lowercased before it is validated.
    ///
    /// The name is checked against the following regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    /// - min length = 3
    /// - max length = 64
    /// - only word characters (alphanumeric or “_”) or "-" are allowed
    /// - must start with an alpha char
    pub fn new(name: &str) -> Result<ServiceName, NameError> {
        validate_name(name).map(|name| ServiceName(name))
    }

    /// returns the name
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an app instance.
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct AppInstance;
/// AppInstanceId represents a unique id assigned to a new app instance.
pub type AppInstanceId = Id<AppInstance>;

/// Represents a service instance.
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct ServiceInstance;
/// ServiceInstanceId represents a unique id assigned to a new service instance. Use cases:
/// 1. logging
/// 2. metrics
pub type ServiceInstanceId = Id<ServiceInstance>;

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
            other => panic!(
                "NameError::Invalid error should have been returned, but instead received : {:?}",
                other
            ),
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
