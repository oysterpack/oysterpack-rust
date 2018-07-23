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

use failure::{Context, Fail};
use rusty_ulid::Ulid;
use std::{fmt, ops::Deref, sync::Arc};

#[cfg(test)]
mod tests;

/// Decorates the failure cause with an ErrorId.
/// - cause must implement the `Fail` trait
///   - see https://boats.gitlab.io/failure/fail.html for more details about the `Fail` trait
/// - cause provides the error context. The cause itself may be another Error.
/// - errors are cloneable which enables errors to be sent on multiple channels, e.g., async error logging and tracking
#[derive(Debug, Fail, Clone)]
pub struct Error {
    id: ErrorId,
    #[cause]
    cause: SharedFailure,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ERR[{}]", self.id)?;

        let fail: &Fail = self.cause();
        if let Some(e) = fail.downcast_ref::<Context<Error>>() {
            write!(f, "-({})", e.get_context())?;
            // Context will always have a cause, i.e., the underlying Error
            write!(f, "-({})", e.cause().unwrap())
        } else {
            write!(f, "-({})", fail)
        }
    }
}

impl Error {
    /// Error constructor
    pub fn new(id: ErrorId, cause: impl Fail) -> Error {
        Error {
            id,
            cause: SharedFailure::new(cause),
        }
    }

    /// ErrorId getter
    pub fn id(&self) -> ErrorId {
        self.id
    }

    /// Returns the error cause
    pub fn cause(&self) -> &Fail {
        &(*self.cause)
    }

    /// Returns the chain of ErrorId(s) from all chained failures that themselves are an Error.
    /// The first ErrorId will be this Error's ErrorId.
    pub fn error_id_chain(&self) -> Vec<ErrorId> {
        let mut error_ids = vec![self.id];

        let mut fail: &Fail = self;
        while let Some(cause) = fail.cause() {
//            if let Some(e) = error_ref(cause) {
//                error_ids.push(e.id());
//            }
            if let Some(e) = cause.downcast_ref::<SharedFailure>() {
                if let Some(e) = e.downcast_ref::<Error>() {
                    error_ids.push(e.id());
                } else if let Some(e) = e.downcast_ref::<Context<Error>>() {
                    error_ids.push(e.get_context().id());
                }
            } else if let Some(e) = cause.downcast_ref::<Error>() {
                error_ids.push(e.id());
            } else if let Some(e) = cause.downcast_ref::<Context<Error>>() {
                error_ids.push(e.get_context().id());
            }

            fail = cause;
        }

        error_ids
    }
}

// TODO: BUG: stack overflow
/// Downcasts the failure to an Error, or returns None
pub fn error_ref(failure: &Fail) -> Option<&Error> {
    match failure.downcast_ref::<Error>() {
        Some(e) => Some(&e),
        None => match failure.downcast_ref::<SharedFailure>() {
            Some(e) => error_ref(&(*e)),
            None => match failure.downcast_ref::<Context<Error>>() {
                Some(e) => Some(e.get_context()),
                None => None,
            },
        },
    }
}

/// Unique Error ID
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ErrorId(pub u128);

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

/// Failure that can be cloned and shared across thread boundaries.
/// It can be derefenced to get to the underlying failure.
#[derive(Clone, Debug)]
pub struct SharedFailure(Arc<Fail>);

impl SharedFailure {
    /// Wraps the provided error into a `SharedFailure`.
    pub fn new<T: Fail>(err: T) -> SharedFailure {
        SharedFailure(Arc::new(err))
    }

    /// Attempts to downcast this `SharedFailure` to a particular `Fail` type by reference.
    ///
    /// If the underlying error is not of type `T`, this will return [`None`](None()).
    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl Fail for SharedFailure {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }
}

impl fmt::Display for SharedFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error> for SharedFailure {
    fn from(err: Error) -> SharedFailure {
        SharedFailure(Arc::new(err))
    }
}

impl Deref for SharedFailure {
    type Target = Fail;

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}
