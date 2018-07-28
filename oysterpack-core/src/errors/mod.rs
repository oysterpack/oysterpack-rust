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

use chrono::{DateTime, Utc};
use devops::SourceCodeLocation;
use failure::{Context, Fail};
use rusty_ulid::Ulid;
use std::{fmt, sync::Arc};

#[cfg(test)]
mod tests;

/// Returns devops::SourceCodeLocation, which refers to the source code location where this macro was invoked.
#[macro_export]
macro_rules! failure {
    ($err_id:expr, $fail:expr) => {{
        use devops;
        use errors::Error;
        Error::new($err_id, $fail, src_loc!())
    }};
}

/// Decorates the failure cause with an ErrorId.
/// - cause must implement the `Fail` trait
///   - see https://boats.gitlab.io/failure/fail.html for more details about the `Fail` trait
/// - cause provides the error context. The cause itself may be another Error.
/// - errors are cloneable which enables errors to be sent on multiple channels, e.g., async error logging and tracking
#[derive(Debug, Clone)]
pub struct Error {
    id: ErrorId,
    timestamp: DateTime<Utc>,
    loc: SourceCodeLocation,
    instance: InstanceId,
    failure: ArcFailure,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        Some(self.failure.failure())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ERR[{}]", self.id)?;

        let fail: &Fail = self.failure();
        if let Some(e) = fail.downcast_ref::<Context<Error>>() {
            write!(f, "({})", e.get_context())?;
            // Context will always have a cause, i.e., the underlying Error
            write!(f, "-({})", e.cause().unwrap())
        } else {
            write!(f, "-({})", fail)
        }
    }
}

impl Error {
    /// Error constructor.
    /// The error is logged.
    pub fn new(id: ErrorId, failure: impl Fail, loc: SourceCodeLocation) -> Error {
        let err = Error {
            id,
            instance: InstanceId::new(),
            failure: ArcFailure::new(failure),
            timestamp: Utc::now(),
            loc,
        };
        error!("{}", err);
        err
    }

    /// ErrorId getter
    pub fn id(&self) -> ErrorId {
        self.id
    }

    /// Returns the error cause
    pub fn failure(&self) -> &Fail {
        &self.failure
    }

    /// When the error occurred.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn loc(&self) -> &SourceCodeLocation {
        &self.loc
    }

    // TODO: This needs more structure to depict the error chain. Each error can have its own error chain.
    /// Returns the chain of ErrorId(s) from all chained failures that themselves are an Error.
    /// The first ErrorId will be this Error's ErrorId.
    pub fn error_id_chain(&self) -> Vec<ErrorId> {
        let mut error_ids = vec![self.id];

        let mut fail: &Fail = self;
        while let Some(cause) = fail.cause() {
            if let Some(e) = error_ref(cause) {
                error_ids.push(e.id());
            }
            fail = cause;
        }

        error_ids
    }
}

/// Tries to converts the failure to an Error reference.
///
/// It will succeed for the following cases:
/// 1. failure is an Error
/// 2. the failure type is Context<Error> - the context Error is returned
/// 3. failure is a ArcFailure, where the underlying failure type is an Error
///
pub fn error_ref(failure: &Fail) -> Option<&Error> {
    if let Some(e) = failure.downcast_ref::<Error>() {
        return Some(e);
    }

    if let Some(e) = failure.downcast_ref::<ArcFailure>() {
        return error_ref(e.failure());
    }

    if let Some(e) = failure.downcast_ref::<Context<Error>>() {
        return Some(e.get_context());
    }

    None
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

/// Unique Error Instance ID.
/// This enables a specific error to be searched for withing another context, e.g., searching log events.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InstanceId(pub u128);

impl InstanceId {
    pub fn new() -> InstanceId {
        InstanceId(Ulid::new().into())
    }
}

impl From<u128> for InstanceId {
    fn from(id: u128) -> Self {
        InstanceId(id)
    }
}

impl From<Ulid> for InstanceId {
    fn from(id: Ulid) -> Self {
        InstanceId(id.into())
    }
}

impl fmt::Display for InstanceId {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

/// ArcFailure is a thread-safe reference-counting pointer to an instance of Fail.
/// It provides shared ownership to a Fail instance.
///
/// Invoking clone on ArcFailure produces a new pointer to the same Fail instance in the heap.
/// When the last ArcFailure pointer to a given Fail instance is destroyed, the pointed-to Fail
/// instance is also destroyed.
#[derive(Clone, Debug)]
pub struct ArcFailure(Arc<Fail>);

impl ArcFailure {
    /// Wraps the provided error into an `ArcFailure`.
    /// If the failure already is an ArcFailure, then it will be cloned and returned.
    pub fn new(failure: impl Fail) -> ArcFailure {
        let failure = ArcFailure(Arc::new(failure));
        {
            if let Some(failure) = failure.downcast_ref::<ArcFailure>() {
                return failure.clone();
            }
        }
        failure
    }

    /// Attempts to downcast this `ArcFailure` to a particular `Fail` type by reference.
    ///
    /// If the underlying error is not of type `T`, this will return [`None`](None()).
    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }

    /// Returns a reference to the underlying failure
    pub fn failure(&self) -> &Fail {
        &*self.0
    }
}

impl Fail for ArcFailure {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }
}

impl fmt::Display for ArcFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
