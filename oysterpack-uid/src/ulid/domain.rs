/*
 * Copyright 2018 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! Provides support for domain scoped ULID(s), that are scoped by the code, i.e., not enforced by the compiler.

use crate::ULID;

use crate::TypedULID;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a generic form of TypedULID&lt;T&gt;, i.e., it's a ULID for the specified domain.
///
/// TypedULID&lt;T&gt; is a typed ULID. However, there are use cases where we want to erase the type and have
/// a generic. An example use case is tagging events with UID(s) for different domains.
///
/// TypedULID&lt;T&gt; can be converted to a [DomainULID](ulid/struct.DomainULID.html) automatically if the
/// TypedULID type T implements [HasDomain](ulid/trait.HasDomain.html).
///
/// ## Example DomainULID generator function
/// ```rust
/// # extern crate oysterpack_uid;
/// # use oysterpack_uid::*;
///
/// fn new_request_id() -> DomainULID {
///     const DOMAIN: Domain = Domain("Request");
///     DomainULID::generate(DOMAIN)
/// }
///
/// # fn main() {
/// let request_id =  new_request_id();
/// assert_eq!(request_id.domain(), "Request");
/// # }
///
/// ```
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainULID {
    domain: String,
    id: ULID,
}

impl DomainULID {
    /// Constructs a new ULID as DomainULID
    pub fn generate(domain: Domain) -> DomainULID {
        DomainULID {
            domain: domain.to_string(),
            id: ULID::generate(),
        }
    }

    /// Associates the Domain to the ULID
    pub fn from_ulid<T: Into<ULID>>(domain: Domain, id: T) -> DomainULID {
        DomainULID {
            domain: domain.to_string(),
            id: id.into(),
        }
    }

    /// Getter for the TypedULID domain
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the id formatted as a [ULID](https://github.com/ulid/spec), e.g., 01CAT3X5Y5G9A62FH1FA6T9GVR
    pub fn ulid(&self) -> ULID {
        self.id
    }
}

impl fmt::Display for DomainULID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.domain, self.id)
    }
}

impl<T: HasDomain> From<TypedULID<T>> for DomainULID {
    fn from(uid: TypedULID<T>) -> Self {
        DomainULID::from_ulid(T::DOMAIN, uid.ulid())
    }
}

/// Domain ID is used to define constants
///
/// ```rust
/// # use oysterpack_uid::*;
/// const FOO_ID: DomainId = DomainId(Domain("Foo"),1866919584682221951251731635731565689);
/// let foo_id: DomainULID = FOO_ID.as_domain_ulid();
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DomainId(pub Domain, pub u128);

impl DomainId {
    /// returns the id as a DomainULID
    pub fn as_domain_ulid(&self) -> DomainULID {
        crate::DomainULID::from_ulid(self.0, self.1)
    }

    /// returns the ID's ULID
    pub fn ulid(&self) -> ULID {
        self.1.into()
    }

    /// Domain getter
    pub fn domain(&self) -> Domain {
        self.0
    }
}

impl Into<DomainULID> for DomainId {
    fn into(self) -> DomainULID {
        self.as_domain_ulid()
    }
}

impl std::fmt::Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_domain_ulid().fmt(f)
    }
}

/// Models the domain used by [DomainULID](ulid/struct.DomainULID.html).
///
/// Domain(s) are static and are defined as consts.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Domain(pub &'static str);

impl Domain {
    /// Returns the domain name
    pub fn name(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl AsRef<str> for Domain {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Meant to be implemented by domain types to associate the Domain with the type.
///
///
/// ## Example
/// ```rust
/// extern crate oysterpack_uid;
/// use oysterpack_uid::*;
///
/// struct User;
///
/// impl HasDomain for User {
///     const DOMAIN: Domain = Domain("User");
/// }
///
/// type UserId = TypedULID<User>;
///
/// fn main() {
///     let id : DomainULID = UserId::generate().into();
///     assert_eq!(id.domain(), User::DOMAIN.name());
/// }
///
/// ```
pub trait HasDomain {
    /// Domain
    const DOMAIN: Domain;
}

#[cfg_attr(tarpaulin, skip)]
#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::run_test;
    use serde_json;
    use std::{cmp::Ordering, str::FromStr};

    #[derive(Debug)]
    struct User;

    impl HasDomain for User {
        const DOMAIN: Domain = Domain("User");
    }

    #[test]
    fn domain_ulid() {
        let id: DomainULID = DomainULID::generate(User::DOMAIN);
        assert_eq!(id.domain(), User::DOMAIN.name());
        println!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
        println!("{:?} => {}", id, id);
        const DOMAIN_FOO: Domain = Domain("Foo");
        let id: DomainULID = DomainULID::from_ulid(DOMAIN_FOO, ULID::generate());
        println!("DomainULID: {}", serde_json::to_string_pretty(&id).unwrap());
        let id = DomainULID::from_ulid(User::DOMAIN, ULID::generate());
    }

    #[test]
    fn domain() {
        run_test("domain", || {
            const USERS: Domain = Domain("users");
            assert_eq!(USERS.as_ref(), "users");
            assert_eq!(USERS.as_ref(), USERS.name());
        });
    }

}
