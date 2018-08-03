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

#[allow(dead_code, unused_imports)]
#[macro_use]
extern crate oysterpack_core;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

// define all failures in a single module
pub mod failures {
    #[derive(Debug, Fail)]
    #[fail(display = "Unauthorized Access")]
    pub struct UnauthorizedAccess;
}

// define all error macros in a single module
#[macro_use]
pub mod errors {
    use oysterpack_core::errors::ErrorId;

    pub const ERR_AUTHZ_FAILED: ErrorId = ErrorId(1);

    // This generates a macro named `ErrAuthzFailed`
    error_macro!(ErrAuthzFailed, ERR_AUTHZ_FAILED);
}

use errors::ERR_AUTHZ_FAILED;
use failures::UnauthorizedAccess;
use oysterpack_core::errors::Error;

#[test]
fn failure() {
    let err: Error = ErrAuthzFailed!(UnauthorizedAccess);
    assert_eq!(err.id(), ERR_AUTHZ_FAILED);
    assert_eq!(err.failure().to_string(), UnauthorizedAccess.to_string());
}
