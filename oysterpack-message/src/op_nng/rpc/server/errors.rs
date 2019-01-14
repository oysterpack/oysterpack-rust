/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! Server related errors

//! server errors

use super::*;
use oysterpack_errors::IsError;
use std::fmt;

// TODO: eliminate the boilerplate with a macro, e.g.,
/*
#[op_error(id=1870511279758140964159435436428736321, level=Alert)] // generates the `impl IsError` boilerplate
#[op_nng_error] // generates the `impl fmt::Display` and `impl From<nng::Error>` boilerplate
pub struct SocketCreateError(nng::Error);
*/

pub use crate::op_nng::errors::{
    AioContextCreateError, AioCreateError, AioReceiveError, SocketCreateError, SocketSetOptError,
};

/// Failed to start listener instance
#[derive(Debug)]
pub struct ListenerStartError(nng::Error);

impl ListenerStartError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870510777469481547545613773325104910);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for ListenerStartError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for ListenerStartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to start listener: {}", self.0)
    }
}

impl From<nng::Error> for ListenerStartError {
    fn from(err: nng::Error) -> ListenerStartError {
        ListenerStartError(err)
    }
}

/// Failed to create listener instance
#[derive(Debug)]
pub struct ListenerCreateError(nng::Error);

impl ListenerCreateError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870302624499038905208367552914704572);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for ListenerCreateError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create listener instance: {}", self.0)
    }
}

impl From<nng::Error> for ListenerCreateError {
    fn from(err: nng::Error) -> ListenerCreateError {
        ListenerCreateError(err)
    }
}

/// An error occurred when setting a listener option.
#[derive(Debug)]
pub struct ListenerSetOptError(nng::Error);

impl ListenerSetOptError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870302624499038905208367552914704572);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for ListenerSetOptError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for ListenerSetOptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to set listener option: {}", self.0)
    }
}

impl From<nng::Error> for ListenerSetOptError {
    fn from(err: nng::Error) -> ListenerSetOptError {
        ListenerSetOptError(err)
    }
}
