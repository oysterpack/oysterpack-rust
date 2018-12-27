Provides support for universally unique identifiers that confirm to the [ULID spec](https://github.com/ulid/spec).

You can generate ULIDs as String or u128.
You can convert ULIDs between String and u128.

```
use oysterpack_uid::{
    ulid,
    ulid_u128,
    ulid_u128_into_string,
    ulid_str_into_u128
};

// generates a new ULID as a string
let id_str = ulid();
// generates a new ULID as u128
let id_u128 = ulid_u128();

// conversions between string and u128 ULIDs
let ulid_str = ulid_u128_into_string(id_u128);
assert_eq!(ulid_str_into_u128(&ulid_str).unwrap(), id_u128);
```

You can define type safe ULID based unique identifiers ([Uid](https://docs.rs/oysterpack_uid/latest/oysterpack_uid/uid/struct.Uid.html)):

### Uid for structs
```rust
use oysterpack_uid::Uid;
struct User;
type UserId = Uid<User>;
let id = UserId::new();
```

### Uid for traits
```rust
use oysterpack_uid::Uid;
trait Foo{}
TypedULID
TypedULID
type FooId = Uid<dyn Foo + Send + Sync>;
let id = FooId::new();
```
By default, Uid<T> is serializable via serde. If serialization is not needed then you can opt out by
including the dependency with default features disabled : `default-features = false`.