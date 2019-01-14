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

//! client related errors

use super::*;
use oysterpack_errors::IsError;
use std::fmt;

pub use crate::op_nng::errors::{
    SocketCreateError, SocketRecvError, SocketSendError, SocketSetOptError,
};

/// An error occurred when setting a dialer option.
#[derive(Debug)]
pub struct DialerSetOptError(nng::Error);

impl DialerSetOptError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870617351358933523700534508070132261);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for DialerSetOptError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for DialerSetOptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to set dialer option: {}", self.0)
    }
}

impl From<nng::Error> for DialerSetOptError {
    fn from(err: nng::Error) -> DialerSetOptError {
        DialerSetOptError(err)
    }
}

/// Failed to create dialer instance
#[derive(Debug)]
pub struct DialerCreateError(nng::Error);

impl DialerCreateError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870617814817456819801511817900043129);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for DialerCreateError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for DialerCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create dialer instance: {}", self.0)
    }
}

impl From<nng::Error> for DialerCreateError {
    fn from(err: nng::Error) -> DialerCreateError {
        DialerCreateError(err)
    }
}

/// Failed to start dialer instance
#[derive(Debug)]
pub struct DialerStartError(nng::Error);

impl DialerStartError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870618072331851255202721873004562985);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for DialerStartError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for DialerStartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to start dialer: {}", self.0)
    }
}

impl From<nng::Error> for DialerStartError {
    fn from(err: nng::Error) -> DialerStartError {
        DialerStartError(err)
    }
}
