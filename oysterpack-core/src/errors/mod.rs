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

//! OysterPack errors standards:
//!
//! 1. Errors are assigned a unique ErrorId
//! 2. Errors are assigned a severity
//! 3. Errors are documented
//! 4. Errors have context
//! 5. Errors have a timestamp
//!

use chrono::SecondsFormat;
use failure::Fail;
use rusty_ulid::Ulid;
use std::{fmt, time::SystemTime};
use time::system_time;

/// Decorates the Fail cause with an ErrorId, timestamp, and Backtrace
#[derive(Debug, Fail, Clone)]
pub struct Error<T: Fail + Clone> {
    id: ErrorId,
    timestamp: SystemTime,
    #[cause]
    cause: T,
}

impl<T: Fail + Clone> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error[{}][{}][{}]",
            self.id,
            system_time::to_date_time(self.timestamp).to_rfc3339_opts(SecondsFormat::Millis, true),
            self.cause
        )
    }
}

impl<T: Fail + Clone> Error<T> {
    /// Error constructor
    pub fn new(id: ErrorId, cause: T) -> Error<T> {
        Error {
            id,
            timestamp: SystemTime::now(),
            cause,
        }
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
