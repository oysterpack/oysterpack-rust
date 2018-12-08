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

//! Standardized Errors

use chrono::{DateTime, Utc};
use oysterpack_events::{
    event::{self, ModuleSource},
    Event, Eventful,
};
use oysterpack_uid::{Domain, DomainULID, TypedULID, ULID};
use std::{fmt, sync::Arc};

/// Converts the Error into an Event&lt;Error&gt;
///
/// ```rust
/// # #[macro_use]
/// # extern crate oysterpack_errors;
/// # use oysterpack_errors::*;
/// # use oysterpack_errors::oysterpack_events::{ Event, Eventful, event::ModuleSource };
///
/// # fn main() {
/// # const FOO_ERR_ID: Id = Id(1863702216415833425137248269790651577);
/// # let err = Error::new(FOO_ERR_ID, Level::Alert, "BOOM", ModuleSource::new(module_path!(), line!()));
/// // converts the Error into an event
/// let err_event = op_error_event!(err);
/// err_event.log();
/// # }
///
/// ```
#[macro_export]
macro_rules! op_error_event {
    ($err:expr) => {
        $err.new_event($crate::oysterpack_events::event::ModuleSource::new(
            module_path!(),
            line!(),
        ))
    };
}

/// Error constructor. The macro can be invoked in the following ways
///
/// ## op_error!((ID, LEVEL), "Error Message")
/// ```rust
/// # #[macro_use]
/// # extern crate oysterpack_errors;
/// # extern crate oysterpack_events;
/// # use oysterpack_errors::*;
/// # use oysterpack_events::{ Event, Eventful, event::ModuleSource };
///
/// # fn main() {
/// pub const FOO_ERR: (Id, Level) = (Id(1863702216415833425137248269790651577), Level::Error);
/// let err = op_error!(FOO_ERR, "BOOM");
/// # }
/// ```
///
/// ## op_error!(ID, LEVEL, "Error Message")
/// ```rust
/// # #[macro_use]
/// # extern crate oysterpack_errors;
/// # extern crate oysterpack_events;
/// # use oysterpack_errors::*;
/// # use oysterpack_events::{ Event, Eventful, event::ModuleSource };
///
/// # fn main() {
/// pub const FOO_ERR: (Id, Level) = (Id(1863702216415833425137248269790651577), Level::Error);
/// let (id, level) = FOO_ERR;
/// let err = op_error!(id, level, "BOOM");
/// # }
/// ```
#[macro_export]
macro_rules! op_error {
    (
    $id_level:expr, $msg:expr
    ) => {
        $crate::error::Error::new(
            $id_level.0,
            $id_level.1,
            $msg.to_string(),
            $crate::oysterpack_events::event::ModuleSource::new(module_path!(), line!()),
        )
    };
    (
    $id:expr, $level:expr, $msg:expr
    ) => {
        $crate::error::Error::new(
            $id,
            $level,
            $msg.to_string(),
            $crate::oysterpack_events::event::ModuleSource::new(module_path!(), line!()),
        )
    };
    ( $error:expr ) => {
        $error.to_error($crate::oysterpack_events::event::ModuleSource::new(
            module_path!(),
            line!(),
        ))
    };
}

/// Errors have the following features:
/// - the error type is identified by its Id
///   - this enables different error types to be documented externally
/// - each error instance is assigned a unique ID
///   - this enables specific errors to be looked up
/// - each error instance is assigned a severity Level.
///   - normally this should be constant across applications
///   - the error instance create timestamp is captured because InstanceId is a ULID
/// - the source code location that created the error is captured
/// - errors can have an optional Error cause specified, i.e., errors can be chained
/// - errors are serializable
/// - errors are cloneable
/// - errors implement the Fail trait
#[derive(Debug, Clone, Serialize, Deserialize, Fail)]
#[fail(display = "{:?}({}:{})[{}] {}", level, id, instance_id, mod_src, msg)]
pub struct Error {
    id: ULID,
    instance_id: InstanceId,
    level: Level,
    msg: String,
    mod_src: ModuleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    cause: Option<Arc<Error>>,
}

impl Error {
    /// Error Domain
    pub const DOMAIN: Domain = Domain("Error");

    /// Constructor
    pub fn new<MSG>(id: Id, level: Level, msg: MSG, mod_src: ModuleSource) -> Error
    where
        MSG: fmt::Display,
    {
        Error {
            id: ULID::from(id.0),
            instance_id: InstanceId::generate(),
            level,
            msg: msg.to_string(),
            mod_src,
            cause: None,
        }
    }

    /// Sets the error cause
    pub fn with_cause(self, cause: Error) -> Error {
        Error {
            cause: Some(Arc::new(cause)),
            ..self
        }
    }

    /// Error ID
    pub fn id(&self) -> Id {
        Id(self.id.into())
    }

    /// Error instance ID
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Error level severity
    pub fn level(&self) -> Level {
        self.level
    }

    /// Error message
    pub fn message(&self) -> &str {
        &self.msg
    }

    /// Where in the source code base the event was created
    pub fn module_source(&self) -> &ModuleSource {
        &self.mod_src
    }

    /// Returns the event timestamp, i.e., when it occurred.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.instance_id.ulid().datetime()
    }

    /// This error may have been caused by another underlying Error
    pub fn cause(&self) -> Option<Arc<Error>> {
        self.cause.as_ref().cloned()
    }

    /// Returns the error cause chain.
    pub fn causes(&self) -> Option<Vec<Arc<Error>>> {
        fn add_cause(mut error_chain: Vec<Arc<Error>>, error: &Arc<Error>) -> Vec<Arc<Error>> {
            error_chain.push(Arc::clone(error));
            match (*error).cause.as_ref() {
                Some(cause) => add_cause(error_chain, cause),
                None => error_chain,
            }
        }

        self.cause
            .as_ref()
            .map(|cause| add_cause(Vec::new(), cause))
    }

    /// Returns the root error cause
    pub fn root_cause(&self) -> Option<Arc<Error>> {
        fn _cause(error: &Arc<Error>) -> Arc<Error> {
            match (*error).cause.as_ref() {
                Some(cause) => _cause(cause),
                None => Arc::clone(error),
            }
        }

        self.cause.as_ref().map(|cause| _cause(cause))
    }
}

impl Eventful for Error {
    /// Event Id
    fn event_id(&self) -> DomainULID {
        DomainULID::from_ulid(Self::DOMAIN, self.id)
    }

    /// Event severity level
    fn event_level(&self) -> event::Level {
        self.level.into()
    }

    /// Converts the Error into an Event.
    /// - Error Id -&gt; Event Id
    /// - Error InstanceId -&gt; Event Instance Id
    /// - Error becomes the Event data
    /// - Error message -> Event message
    ///
    /// NOTE: the Event ModuleSource is where the event was created in the code. The Error ModuleSource
    /// is where the Error was created in the code.
    fn new_event(self, mod_src: ModuleSource) -> Event<Self> {
        Event::from(
            event::InstanceId::from(self.instance_id.ulid()),
            self,
            mod_src,
        )
    }
}

op_ulid! {
    /// Error unique identifier
    pub Id
}

/// Marker type for an Error instance, which is used to define [InstanceId](type.InstanceId.html)
#[allow(missing_debug_implementations)]
pub struct Instance;

/// Error instance IDs are generated for each new Error instance that is created.
pub type InstanceId = TypedULID<Instance>;

/// Error severity level
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Level {
    /// System is unusable.
    /// A panic condition.
    Emergency,
    /// Action must be taken immediately.
    /// A condition that should be corrected immediately.
    Alert,
    /// Critical conditions
    Critical,
    /// Error conditions
    Error,
}

impl Into<event::Level> for Level {
    fn into(self) -> event::Level {
        match self {
            Level::Emergency => event::Level::Emergency,
            Level::Alert => event::Level::Alert,
            Level::Critical => event::Level::Critical,
            Level::Error => event::Level::Error,
        }
    }
}

/// Should be implemented by Error objects
pub trait IsError: fmt::Display {
    /// Error Id
    fn error_id(&self) -> Id;

    /// Error Level
    fn error_level(&self) -> Level;

    /// Converts itself into an Error
    fn to_error(&self, mod_src: ModuleSource) -> Error {
        Error::new(self.error_id(), self.error_level(), &self, mod_src)
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::run_test;
    use oysterpack_uid::ULID;
    use std::time::Duration;

    #[test]
    fn is_error() {
        struct RequestTimeout {
            timeout: Duration,
        }

        impl RequestTimeout {
            pub const ERROR_ID: Id = Id(1865548837704866157621294180822811573);
            pub const ERROR_LEVEL: Level = Level::Error;
        }

        impl IsError for RequestTimeout {
            /// Error Id
            fn error_id(&self) -> Id {
                RequestTimeout::ERROR_ID
            }

            /// Error Level
            fn error_level(&self) -> Level {
                RequestTimeout::ERROR_LEVEL
            }
        }

        impl fmt::Display for RequestTimeout {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Request timed out: {:?}", self.timeout)
            }
        }

        enum AuthErr {
            UnknownSubject,
            InvalidCredentials,
            Unauthorized,
            BadRequest,
            ServerError,
            RequestTimeout(RequestTimeout),
        }

        impl IsError for AuthErr {
            /// Error Id
            fn error_id(&self) -> Id {
                match self {
                    AuthErr::UnknownSubject => Id(1),
                    AuthErr::InvalidCredentials => Id(2),
                    AuthErr::Unauthorized => Id(3),
                    AuthErr::BadRequest => Id(4),
                    AuthErr::ServerError => Id(5),
                    AuthErr::RequestTimeout(err) => err.error_id(),
                }
            }

            /// Error Level
            fn error_level(&self) -> Level {
                match self {
                    AuthErr::UnknownSubject => Level::Alert,
                    AuthErr::InvalidCredentials => Level::Error,
                    AuthErr::Unauthorized => Level::Alert,
                    AuthErr::BadRequest => Level::Error,
                    AuthErr::ServerError => Level::Alert,
                    AuthErr::RequestTimeout(err) => err.error_level(),
                }
            }
        }

        impl fmt::Display for AuthErr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let msg = match self {
                    AuthErr::UnknownSubject => "UnknownSubject".to_string(),
                    AuthErr::InvalidCredentials => "InvalidCredentials".to_string(),
                    AuthErr::Unauthorized => "Unauthorized".to_string(),
                    AuthErr::BadRequest => "BadRequest".to_string(),
                    AuthErr::ServerError => "ServerError".to_string(),
                    AuthErr::RequestTimeout(err) => err.to_string(),
                };
                f.write_str(msg.as_str())
            }
        }

        let err = AuthErr::RequestTimeout(RequestTimeout {
            timeout: Duration::from_millis(100),
        });
        let err: Error = op_error!(err);
        println!("err: {}", err);
        assert_eq!(err.id(), RequestTimeout::ERROR_ID);
        assert_eq!(err.level(), RequestTimeout::ERROR_LEVEL);
    }

    #[test]
    fn error() {
        run_test("error", || {
            let id = ULID::generate();
            let err = op_error!(Id(id.into()), Level::Error, "BOOM");
            error!("{}", err);
            assert_eq!(Id(id.into()), err.id());
            assert_eq!(Level::Error, err.level());
            assert_eq!(err.message(), "BOOM");

            let err2 = op_error!(Id(id.into()), Level::Error, "BOOM");
            error!("{}", err2);

            assert_eq!(err2.id(), err.id());
            assert_eq!(err2.level(), err.level());
            assert_ne!(err2.instance_id(), err.instance_id());

            assert!(err2.cause().is_none());
            assert!(err2.causes().is_none());
            assert!(err2.root_cause().is_none());
        });
    }

    #[test]
    fn error_into_event() {
        run_test("error", || {
            let id = ULID::generate();
            let err = op_error!(Id(id.into()), Level::Error, "BOOM");
            let event: Event<Error> = err.new_event(op_module_source!());
            event.log();
        });
    }

    mod errs {
        use super::*;
        pub const FOO_ERR: (Id, Level) = (Id(1863702216415833425137248269790651577), Level::Error);
        pub const BAR_ERR: (Id, Level) = (Id(1863710844723084375065842092297071588), Level::Alert);
        pub const BAZ_ERR: (Id, Level) =
            (Id(1864734280873114327279151769208160280), Level::Critical);
    }

    #[test]
    fn op_errors_macro() {
        run_test("error", || {
            const FOO_ERR_ID: Id = Id(1863702216415833425137248269790651577);
            const FOO_ERR_LEVEL: Level = Level::Error;
            let err = op_error!(FOO_ERR_ID, FOO_ERR_LEVEL, "BOOM".to_string());
            let event: Event<Error> = err.new_event(op_module_source!());
            event.log();

            let err = op_error!(errs::FOO_ERR, "BOOM!!".to_string());
            let event: Event<Error> = err.new_event(op_module_source!());
            event.log();
        });
    }

    #[test]
    fn error_with_cause() {
        run_test("error_with_cause", || {
            let cause = op_error!(errs::FOO_ERR, "THE MIDDLE CAUSE");
            let cause = cause.with_cause(op_error!(errs::BAZ_ERR, "THE ROOT CAUSE"));
            let err = op_error!(errs::BAR_ERR, "THE TOP LEVEL ERROR");
            let err = err.with_cause(cause);

            match err.causes() {
                None => panic!("There should have been causes"),
                Some(causes) => {
                    assert_eq!(causes.len(), 2);
                    assert_eq!(causes[0].id(), errs::FOO_ERR.0);
                    assert_eq!(causes[1].id(), errs::BAZ_ERR.0);
                }
            }
            assert_eq!(err.root_cause().unwrap().id(), errs::BAZ_ERR.0);

            let err_event = op_error_event!(err);
            err_event.log();
        })
    }

}
