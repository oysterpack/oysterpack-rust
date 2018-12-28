Provides support for universally unique identifiers that confirm to the [ULID spec](https://github.com/ulid/spec).

## Features
- ULID generation via [ULID](ulid/struct.ULID.html)
- ULIDs can be associated with a domain. Example domains are user ids, request ids, application ids, service ids, etc.
  - [DomainULID](ulid/struct.DomainULID.html) and [Domain](ulid/struct.Domain.html)
    - domain is defined by code, i.e., [Domain](ulid/struct.Domain.html) is used to define domain names as constants
    - [DomainULID](ulid/struct.DomainULID.html) scopes [ULID](ulid/struct.ULID.html)(s) to a [Domain](ulid/struct.Domain.html)
  - [DomainId](ulid/struct.DomainId.html) can be used to define constants, which can then be converted into DomainULID
  - u128 or ULID tuple structs marked with a `#[ulid]` attribute
- ULIDs are thread safe, i.e., they can be sent across threads
- ULIDs are lightweight and require no heap allocation
- ULIDs are serializable via [serde](https://crates.io/crates/serde)

### Generating ULIDs
```rust
use oysterpack_uid::*;
let id = ULID::generate();
```
### Generating ULID constants
```rust
use oysterpack_uid::{
    ULID, Domain, DomainULID, macros::{ulid, domain}
};
use serde::{Serialize, Deserialize};

#[domain(Foo)]
#[ulid]
pub struct FooId(u128);

const FOO_ID: FooId = FooId(1866910953065622895350834727020862173);
let ulid: ULID = FOO_ID.into();
let ulid: DomainULID = FOO_ID.into();
```

### Generating DomainULIDs
```rust
use oysterpack_uid::*;
const DOMAIN: Domain = Domain("Foo");
let id = DomainULID::generate(DOMAIN);
```

### Generating DomainULID constants via DomainId
```rust
use oysterpack_uid::*;
const FOO: Domain = Domain("Foo");
pub const FOO_EVENT_ID: DomainId = DomainId(FOO, 1866921270748045466739527680884502485);
let domain_ulid = FOO_EVENT_ID.as_domain_ulid();
```