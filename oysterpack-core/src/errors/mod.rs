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

//! OysterPack error standards:
//!
//! - Errors are assigned a unique ErrorId
//! - Errors are assigned a severity
//! - Errors are documented
//! - Errors have context
//! - Errors are timestamped
//! - Errors are tracked against crates in 2 ways :
//!   1. the binary crate - within which app the error occurred
//!   2. the library crate - the error was produced by which library
//!

use failure::Fail;
use rusty_ulid::Ulid;
use std::fmt;

/// Decorates the failure cause with an ErrorId.
/// - cause must implement the `Fail` trait
///   - see https://boats.gitlab.io/failure/fail.html for more details about the `Fail` trait
/// - cause provides the error context. The cause itself may be another Error.
/// - errors are cloneable which enables errors to be sent on multiple channels, e.g., async error logging and tracking
#[derive(Debug, Fail, Clone)]
pub struct Error<E: Fail + Clone> {
    id: ErrorId,
    #[cause]
    cause: E,
}

impl<T: Fail + Clone> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error[{}][{}]", self.id, self.cause)
    }
}

impl<T: Fail + Clone> Error<T> {
    /// Error constructor
    pub fn new(id: ErrorId, cause: T) -> Error<T> {
        Error { id, cause }
    }

    /// ErrorId getter
    pub fn id(&self) -> ErrorId {
        self.id
    }

    /// Returns the error cause
    pub fn cause(&self) -> &T {
        &self.cause
    }
}

/// Unique Error ID
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ErrorId(u128);

impl ErrorId {
    pub fn new(id: u128) -> ErrorId {
        ErrorId(id)
    }

    pub fn value(&self) -> u128 {
        self.0
    }
}

impl From<u128> for ErrorId {
    fn from(id: u128) -> Self {
        ErrorId(id)
    }
}

impl From<Ulid> for ErrorId {
    fn from(id: Ulid) -> Self {
        ErrorId(id.into())
    }
}

impl fmt::Display for ErrorId {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
