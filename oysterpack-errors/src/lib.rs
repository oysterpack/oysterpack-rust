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

    #[test]
    #[should_panic]
    fn panic() {
        Panic(PanicId(1)).panic("BOOM");
    }
}
