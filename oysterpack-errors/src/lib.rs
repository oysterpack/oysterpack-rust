// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Panics must be registered and documented within a database.
/// Panics are assigned a unique error id.
///
/// Within application binaries and libraries, Panic(s) should be defined as global constants.
///
/// ```rust
/// use oysterpack_errors::{Panic, PanicId};
///
/// const DATABASE_DOWN : Panic = Panic(PanicId(1));
/// const INVALID_APP_CONFIG : Panic = Panic(PanicId(2));
/// const INVALID_LICENSE : Panic = Panic(PanicId(3));
/// ```
///

#[derive(Copy, Clone, Debug)]
pub struct Panic(pub PanicId);

impl Panic {
    pub fn panic(self, message: &str) {
        panic!("{} {}", self.id().id(), message)
    }

    pub fn id(&self) -> PanicId { self.0 }
}

#[derive(Copy, Clone, Debug)]
pub struct PanicId(pub u64);

impl PanicId {
    pub fn id(&self) -> u64 { self.0 }
}


#[cfg(test)]
mod tests {
    use super::*;

    const DATABASE_DOWN : Panic = Panic(PanicId(1));

    #[test]
    #[should_panic]
    fn panic() {
        Panic(PanicId(1)).panic("BOOM");
    }
}
