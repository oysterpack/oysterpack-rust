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
//! - Apps can be transferred between Domains.
//! - Apps are composed of services
//!
//! ## Service
//! - Services are designed as Actors via [Actix](https://docs.rs/crate/actix)
//! - Tne Service interface is defined by the Actor supported messages.
//! - A service is self-contained in a single library crate.
//!
//! ## AppInstance
//! - Represents a running App instance
//!

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_platform/0.1.0")]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate oysterpack_id;
extern crate regex;
extern crate semver;

use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use semver::Version;

pub use oysterpack_id::Id;

#[cfg(test)]
mod tests;

/// Domain represents a logical domain used to provide structure, organization, and ownership.
/// For example, [App](struct.App.html)(s) are owned by a Domain.
///
/// The DomainId identifies the Domain. The DomainName is a user friendly for the Domain, i.e.,
/// mapped to the DomainId. The DomainName can be changed, as long as it is globally unique amongst
/// all domain names, but the DomainId is permanent.
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

    /// Unique Domain name.
    /// DomainName is a user friendly name mapped to the DomainId.
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
        static ref RE: regex::Regex = regex::Regex::new(r"^[a-z][\w\-]{2,63}$").unwrap();
        static ref STARTS_WITH_ALPHA: regex::Regex = regex::Regex::new(r"^[a-z].*$").unwrap();
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
    /// App constructor
    pub fn new(
        domain_id: DomainId,
        id: AppId,
        name: AppName,
        version: Version,
        services: HashSet<Service>,
    ) -> Result<App, AppInstanceError> {
        if services.is_empty() {
            Err(AppInstanceError::AppHasNoServices)
        } else {
            Ok(App {
                domain_id,
                id,
                name,
                version,
                services,
            })
        }
    }

    /// Returns the owning Domain.
    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// Unique App ID across all domains.
    pub fn id(&self) -> AppId {
        self.id
    }

    /// Unique App name across all domains.
    /// The App name is a user friendly name mapped to the AppId.
    pub fn name(&self) -> &AppName {
        &self.name
    }

    /// App version.
    ///
    /// App id and version form a unique constraint on App.
    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    /// The set of services that compose the app.
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
    /// Service constructor
    pub fn new(id: ServiceId, name: ServiceName, version: semver::Version) -> Service {
        Service { id, name, version }
    }

    /// ServiceId
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// ServiceName must be unique across all services.
    pub fn name(&self) -> &ServiceName {
        &self.name
    }

    /// Service id and version form a unique constraint on Service.
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppInstance {
    app: App,
    instance_id: AppInstanceId,
}
/// AppInstanceId represents a unique id assigned to a new app instance.
pub type AppInstanceId = Id<AppInstance>;

impl AppInstance {
    /// AppInstance constructor
    pub fn new(app: App, instance_id: AppInstanceId) -> AppInstance {
        AppInstance { app, instance_id }
    }
}

/// Represents a service instance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceInstance {
    service: Service,
    instance_id: ServiceInstanceId,
}

/// ServiceInstanceId represents a unique id assigned to a new service instance. Use cases:
/// 1. logging
/// 2. metrics
pub type ServiceInstanceId = Id<ServiceInstance>;

impl ServiceInstance {
    /// ServiceInstance constructor
    pub fn new(service: Service) -> ServiceInstance {
        ServiceInstance {
            service,
            instance_id: ServiceInstanceId::new(),
        }
    }
}

impl fmt::Display for ServiceInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}][{}][{}][{}]",
            self.service.name(),
            self.service.version(),
            self.service.id(),
            self.instance_id
        )
    }
}

impl ServiceInstance {
    /// Service getter
    pub fn service(&self) -> &Service {
        &self.service
    }

    /// ServiceInstanceId getter
    pub fn instance_id(&self) -> ServiceInstanceId {
        self.instance_id
    }
}

/// Name validation errors
#[derive(Fail, Debug)]
pub enum NameError {
    /// Name min length is 3
    #[fail(
        display = "Name min length is 3 : [{}] length = {}",
        name,
        len
    )]
    TooShort {
        /// name
        name: String,
        /// name length
        len: usize,
    },
    /// Name max length is 64
    #[fail(
        display = "Name max length is 64 : [{}] length = {}",
        name,
        len
    )]
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
        name: String,
    },
    /// Name must match against regex :
    /// ```text
    /// ^[a-z][\w\-]{2,63}$
    /// ```
    #[fail(
        display = "Name is invalid. It must start with an alpha and the rest can only conist of alphanumeric, '_', or '-' : [{}]",
        name
    )]
    Invalid {
        /// name
        name: String,
    },
}

/// AppInstanceBuilder errors
#[derive(Fail, Debug)]
pub enum AppInstanceError {
    /// App must have at 1 service defined.
    #[fail(display = "App must have at 1 service defined.")]
    AppHasNoServices,
}
