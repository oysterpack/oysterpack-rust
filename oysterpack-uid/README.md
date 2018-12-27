Provides macros for its [oysterpack_uid](https://crates.io/crates/oysterpack_uid) sister crate.
This crate is exported by [oysterpack_uid](https://crates.io/crates/oysterpack_uid).

## For Example
<pre>
use oysterpack_uid::macros::ulid;

#[ulid]
pub struct UserId(pub u128);
</pre>

### Produces the following code
<pre>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct UserId(pub u128);

impl From<UserId> for oysterpack_uid::ULID {
    fn from(ulid: UserId) -> oysterpack_uid::ULID {
        ulid.0.into()
    }
}

impl From<oysterpack_uid::ULID> for UserId{
    fn from(ulid: oysterpack_uid::ULID) -> UserId{
        UserId(ulid.into())
    }
}

impl From<oysterpack_uid::DomainULID> for UserId{
    fn from(ulid: oysterpack_uid::DomainULID) -> UserId{
        UserId(ulid.ulid().into())
    }
}

impl UserId{
    /// returns the ID as a ULID
    pub fn ulid(&self) -> oysterpack_uid::ULID {
        self.0.into()
    }
}

impl std::fmt::Display for UserId{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ulid: oysterpack_uid::ULID = self.0.into();
        f.write_str(ulid.to_string().as_str())
    }
}

</pre>