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

//! Errors that are shared between the client and server

use oysterpack_errors::IsError;
use std::fmt;

/// Failed to create socket
#[derive(Debug)]
pub struct SocketCreateError(nng::Error);

impl SocketCreateError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870511279758140964159435436428736321);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for SocketCreateError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SocketCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create socket: {}", self.0)
    }
}

impl From<nng::Error> for SocketCreateError {
    fn from(err: nng::Error) -> SocketCreateError {
        SocketCreateError(err)
    }
}

/// An error occurred when setting a socket option.
#[derive(Debug)]
pub struct SocketSetOptError(nng::Error);

impl SocketSetOptError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870511354278148346409496152407634279);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for SocketSetOptError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SocketSetOptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to set socket option: {}", self.0)
    }
}

impl From<nng::Error> for SocketSetOptError {
    fn from(err: nng::Error) -> SocketSetOptError {
        SocketSetOptError(err)
    }
}

/// Failed to send a message on the socket
#[derive(Debug)]
pub struct SocketSendError(nng::Error);

impl SocketSendError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870691045390492837758317571713575234);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for SocketSendError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SocketSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to send message on socket: {}", self.0)
    }
}

impl From<(nng::Message, nng::Error)> for SocketSendError {
    fn from(err: (nng::Message, nng::Error)) -> SocketSendError {
        SocketSendError(err.1)
    }
}

/// Failed to receive a message on the socket
#[derive(Debug)]
pub struct SocketRecvError(nng::Error);

impl SocketRecvError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870691257326561948476799832627658814);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for SocketRecvError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for SocketRecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to receive message on socket: {}", self.0)
    }
}

impl From<nng::Error> for SocketRecvError {
    fn from(err: nng::Error) -> SocketRecvError {
        SocketRecvError(err)
    }
}
