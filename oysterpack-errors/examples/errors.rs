// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error best practices.
//! - all errors should implement IsError.
//! - all errors should be defined within a submodule named errors
//! - type safe enums are used as errors
//!   - enum variants simply wrap error types that implement te `IsError` trait
//! - errors can be reused from other error modules by wrapping them into your domain specific errors

#[macro_use]
extern crate oysterpack_errors;

pub mod server {

    pub mod errors {
        use std::fmt;

        use oysterpack_errors::{Id, IsError, Level};

        pub struct InternalServerError(String);

        impl fmt::Display for InternalServerError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Internal server error: {}", self.0)
            }
        }

        impl InternalServerError {
            const ERROR_ID: Id = Id(1865551845355628987320990301934080894);
            const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for InternalServerError {
            /// Error Id
            fn error_id(&self) -> Id {
                Self::ERROR_ID
            }

            /// Error Level
            fn error_level(&self) -> Level {
                Self::ERROR_LEVEL
            }
        }

    }
}

pub mod client {

    pub mod errors {
        use std::{fmt, time::Duration};

        use oysterpack_errors::{Id, IsError, Level};

        pub struct BadRequest;

        impl fmt::Display for BadRequest {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Bad request")
            }
        }

        impl BadRequest {
            const ERROR_ID: Id = Id(1865551572052646964972568445250428269);
            const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for BadRequest {
            fn error_id(&self) -> Id {
                Self::ERROR_ID
            }

            fn error_level(&self) -> Level {
                Self::ERROR_LEVEL
            }
        }

        pub struct RequestTimeout {
            timeout: Duration,
        }

        impl RequestTimeout {
            pub const ERROR_ID: Id = Id(1865548837704866157621294180822811573);
            pub const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for RequestTimeout {
            fn error_id(&self) -> Id {
                RequestTimeout::ERROR_ID
            }

            fn error_level(&self) -> Level {
                RequestTimeout::ERROR_LEVEL
            }
        }

        impl fmt::Display for RequestTimeout {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Request timed out: {:?}", self.timeout)
            }
        }
    }
}

pub mod auth {
    pub mod errors {
        use crate::{client::errors::*, server::errors::*};
        use std::fmt;

        use oysterpack_errors::{Id, IsError, Level};

        pub enum AuthErr {
            UnknownSubject(UnknownSubject),
            InvalidCredentials(InvalidCredentials),
            Unauthorized(Unauthorized),
            BadRequest(BadRequest),
            ServerError(InternalServerError),
            RequestTimeout(RequestTimeout),
        }

        impl IsError for AuthErr {
            fn error_id(&self) -> Id {
                match self {
                    AuthErr::UnknownSubject(err) => err.error_id(),
                    AuthErr::InvalidCredentials(err) => err.error_id(),
                    AuthErr::Unauthorized(err) => err.error_id(),
                    AuthErr::BadRequest(err) => err.error_id(),
                    AuthErr::ServerError(err) => err.error_id(),
                    AuthErr::RequestTimeout(err) => err.error_id(),
                }
            }

            fn error_level(&self) -> Level {
                match self {
                    AuthErr::UnknownSubject(err) => err.error_level(),
                    AuthErr::InvalidCredentials(err) => err.error_level(),
                    AuthErr::Unauthorized(err) => err.error_level(),
                    AuthErr::BadRequest(err) => err.error_level(),
                    AuthErr::ServerError(err) => err.error_level(),
                    AuthErr::RequestTimeout(err) => err.error_level(),
                }
            }
        }

        impl fmt::Display for AuthErr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let msg = match self {
                    AuthErr::UnknownSubject(err) => err.to_string(),
                    AuthErr::InvalidCredentials(err) => err.to_string(),
                    AuthErr::Unauthorized(err) => err.to_string(),
                    AuthErr::BadRequest(err) => err.to_string(),
                    AuthErr::ServerError(err) => err.to_string(),
                    AuthErr::RequestTimeout(err) => err.to_string(),
                };
                f.write_str(msg.as_str())
            }
        }

        pub struct UnknownSubject;

        impl fmt::Display for UnknownSubject {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Unknown subject")
            }
        }

        impl UnknownSubject {
            const ERROR_ID: Id = Id(1865552198292831176435293190788686559);
            const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for UnknownSubject {
            fn error_id(&self) -> Id {
                Self::ERROR_ID
            }

            fn error_level(&self) -> Level {
                Self::ERROR_LEVEL
            }
        }

        pub struct InvalidCredentials;

        impl fmt::Display for InvalidCredentials {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Bad credentials")
            }
        }

        impl InvalidCredentials {
            const ERROR_ID: Id = Id(1865552281942821052229733799712616785);
            const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for InvalidCredentials {
            fn error_id(&self) -> Id {
                Self::ERROR_ID
            }

            fn error_level(&self) -> Level {
                Self::ERROR_LEVEL
            }
        }

        pub struct Unauthorized;

        impl fmt::Display for Unauthorized {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Unauthorized")
            }
        }

        impl Unauthorized {
            const ERROR_ID: Id = Id(1865552335081195403758565194192915548);
            const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for Unauthorized {
            fn error_id(&self) -> Id {
                Self::ERROR_ID
            }

            fn error_level(&self) -> Level {
                Self::ERROR_LEVEL
            }
        }
    }

    pub struct LoginRequest {
        pub user: String,
        pub password: String,
    }

    pub struct User;

    pub fn login(request: LoginRequest) -> Result<User, errors::AuthErr> {
        if request.user.len() == 0 || request.password.len() == 0 {
            return Err(errors::AuthErr::BadRequest(
                crate::client::errors::BadRequest,
            ));
        }

        if request.user != "alfio.zappala" {
            return Err(errors::AuthErr::UnknownSubject(errors::UnknownSubject));
        }

        if request.password != "secret" {
            return Err(errors::AuthErr::InvalidCredentials(
                errors::InvalidCredentials,
            ));
        }

        Ok(User)
    }
}

fn main() {
    // application code works with domain specific errors
    match auth::login(auth::LoginRequest {
        user: "alfio.zappala".to_string(),
        password: "invalid_password".to_string(),
    }) {
        Ok(_) => println!("LOGIN SUCCESS"),
        Err(err) => {
            // convert the domain specific error into a generic Error, which can be reported in a generic manner
            let err = op_error!(err);
            eprintln!("LOGIN FAILED: {}", err);
        }
    }
}
