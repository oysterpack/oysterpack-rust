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

/// Failed to create new asynchronous I/O handle
#[derive(Debug)]
pub struct AioCreateError(nng::Error);

impl AioCreateError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870510443603468311033495279443790945);
    /// Level::Alert
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
}

impl IsError for AioCreateError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for AioCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create new aio handle: {}", self.0)
    }
}

impl From<nng::Error> for AioCreateError {
    fn from(err: nng::Error) -> AioCreateError {
        AioCreateError(err)
    }
}

/// Aio receive operation failed
#[derive(Debug)]
pub struct AioReceiveError(nng::Error);

impl AioReceiveError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870374078796088086815067802169113773);
    /// Level::Error
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
}

impl IsError for AioReceiveError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for AioReceiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Aio receive operation failed: {}", self.0)
    }
}

impl From<nng::Error> for AioReceiveError {
    fn from(err: nng::Error) -> AioReceiveError {
        AioReceiveError(err)
    }
}

/// Failed to create new socket context
#[derive(Debug)]
pub struct AioContextCreateError(nng::Error);

impl AioContextCreateError {
    /// Error Id
    pub const ERROR_ID: oysterpack_errors::Id =
        oysterpack_errors::Id(1870374278155759380545373361718947172);
    /// Level::Error
    pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
}

impl IsError for AioContextCreateError {
    fn error_id(&self) -> oysterpack_errors::Id {
        Self::ERROR_ID
    }

    fn error_level(&self) -> oysterpack_errors::Level {
        Self::ERROR_LEVEL
    }
}

impl fmt::Display for AioContextCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create new socket context: {}", self.0)
    }
}

impl From<nng::Error> for AioContextCreateError {
    fn from(err: nng::Error) -> AioContextCreateError {
        AioContextCreateError(err)
    }
}
