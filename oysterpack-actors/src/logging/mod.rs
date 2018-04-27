// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Actor system slog integration
//! [slog-rs](https://docs.rs/crate/slog/) is used as the logging framework.
//!
//! ## Why Slog ?
//! - because it provides structured logging
//! - because there is support for logging JSON
//! - because there is support for type safe logging

extern crate lazy_static;
extern crate slog;
extern crate slog_extlog;

extern crate actix;
extern crate futures;

use self::slog::Logger;

use self::actix::prelude::*;
use self::futures::prelude::*;

use actor::ActorMessageResponse;
use CRATE_LOG_NAME;

use self::system::*;

#[cfg(test)]
mod tests;

pub const SYSTEM: &'static str = "system";
pub const SYSTEM_SERVICE: &'static str = "system_service";
pub const EVENT: &'static str = "event";
pub const ACTOR_ID: &'static str = "actor_id";

/// Returns the root logger for the actor system.
///
/// # Panics
/// Panics if this method is not run within the context of a running actor system.
pub fn system_logger() -> ActorMessageResponse<slog::Logger> {
    let service = Arbiter::system_registry().get::<Slogger>();
    let request = service.send(GetLogger).map(|result| result.unwrap());
    Box::new(request)
}

/// Used to initialize the root logger for the actor system.
///
/// # Panics
/// Panics if this method is not run within the context of a running actor system.
pub fn set_system_logger(logger: Logger) -> ActorMessageResponse<()> {
    let service = Arbiter::system_registry().get::<Slogger>();
    let request = service
        .send(SetLogger(logger))
        .map(|result| result.unwrap());
    Box::new(request)
}

mod system {
    use super::*;

    extern crate slog_async;
    extern crate slog_json;

    use self::slog::Drain;
    use slog_extlog::stats;
    use self::slog_json::Json;
    use self::slog_async::Async;

    use std::io::{stderr, Write};

    /// Type alias used for Result Error types that should never result in an Error
    type Never = ();

    pub struct Slogger {
        logger: Logger,
    }

    impl Actor for Slogger {
        type Context = Context<Self>;
    }

    impl SystemService for Slogger {
        fn service_started(&mut self, _: &mut Context<Self>) {
            self.logger = self.logger.new(o!(SYSTEM => Arbiter::name()));
        }
    }

    impl Default for Slogger {
        fn default() -> Self {
            //            fn async_json_drain<W: Write + Send + 'static>(
            //                writer: W,
            //                chan_size: usize,
            //            ) -> slog_async::Async {
            //                let drain = Json::new(writer)
            //                    .add_default_keys()
            //                    .set_newlines(true)
            //                    .build()
            //                    .fuse();
            //                let drain = Async::new(drain).chan_size(chan_size).build();
            //                drain
            //            }
            //
            //            let drain = async_json_drain(stderr(), 1024);
            //            let logger = Logger::root(drain.fuse(), o!());

            let logger = slog::Logger::root(
                ::std::sync::Mutex::new(slog_json::Json::default(stderr())).map(slog::Fuse),
                o!(),
            );
            Slogger { logger }
        }
    }

    impl Supervised for Slogger {}

    #[derive(Debug)]
    pub struct GetLogger;

    impl Message for GetLogger {
        type Result = Result<Logger, Never>;
    }

    impl Handler<GetLogger> for Slogger {
        type Result = Result<Logger, Never>;

        fn handle(&mut self, _: GetLogger, _: &mut Self::Context) -> Self::Result {
            Ok(self.logger.clone())
        }
    }

    #[derive(Debug)]
    pub struct SetLogger(pub Logger);

    impl Message for SetLogger {
        type Result = Result<(), Never>;
    }

    impl Handler<SetLogger> for Slogger {
        type Result = Result<(), Never>;

        fn handle(&mut self, msg: SetLogger, _: &mut Self::Context) -> Self::Result {
            self.logger = msg.0;
            Ok(())
        }
    }
}
