// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Used to initialize slog

extern crate lazy_static;
extern crate slog;
extern crate slog_async;
extern crate slog_json;

extern crate actix;
extern crate futures;

use self::slog::{Drain, Logger};
use self::slog_json::Json;
use self::slog_async::Async;

use self::actix::prelude::*;
use self::futures::prelude::*;

use actor::ActorMessageResponse;

use std::io::stderr;

use self::actor::*;

#[cfg(test)]
mod tests;

pub const SYSTEM: &'static str = "system";
pub const SYSTEM_SERVICE: &'static str = "system_service";
pub const EVENT: &'static str = "event";
pub const ACTOR_ID: &'static str = "actor_id";

/// Returns the root logger.
pub fn root_logger() -> ActorMessageResponse<slog::Logger> {
    let service = Arbiter::system_registry().get::<Slogger>();
    let request = service.send(GetLogger).map(|result| result.unwrap());
    Box::new(request)
}

pub fn set_root_logger(logger: Logger) -> ActorMessageResponse<()> {
    let service = Arbiter::system_registry().get::<Slogger>();
    let request = service
        .send(SetLogger(logger))
        .map(|result| result.unwrap());
    Box::new(request)
}

mod actor {
    use super::*;

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
            let drain = Json::new(stderr())
                .add_default_keys()
                .set_newlines(true)
                .build()
                .fuse();
            let drain = Async::new(drain).chan_size(1024).build().fuse();
            let logger = Logger::root(drain, o!());
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
