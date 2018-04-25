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

use self::slog::{Drain, Level, Logger};
use self::slog_json::Json;
use self::slog_async::Async;

use self::actix::prelude::*;
use self::futures::prelude::*;

use actor::ActorMessageResponse;

use std::io::{stderr, Write};

#[cfg(test)]
mod tests;

use std::sync::RwLock;

lazy_static! {
    static ref LOGGER: RwLock<slog::Logger> = {
        let drain = Json::new(stderr())
                .add_default_keys()
                .set_newlines(true)
                .build()
                .fuse();
        let drain = Async::new(drain)
                .chan_size(1024)
                .build()
                .fuse();
        let logger = Logger::root(drain.fuse(), o!());
        RwLock::new(logger)
    };
}

pub const SYSTEM: &'static str = "system";
pub const SYSTEM_SERVICE: &'static str = "system_service";
pub const EVENT: &'static str = "event";
pub const ACTOR_ID: &'static str = "actor_id";

/// Returns the root logger.
///
/// Default logger is configured to log async JSON log events to standard error. The async channel size is 1024.
pub fn root_logger() -> slog::Logger {
    let logger = LOGGER.read().unwrap();
    logger.clone()
}

/// Used to initialize the root logger. This should be initialized at application startup.
pub fn set_root_logger(root_logger: Logger) {
    let mut logger = LOGGER.write().unwrap();
    info!(root_logger, "logging initialized");
    *logger = root_logger;
}
