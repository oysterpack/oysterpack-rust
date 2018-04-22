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

#[cfg(test)]
mod tests;

use std::sync::RwLock;

lazy_static!{
    static ref LOGGER: RwLock<Option<slog::Logger>> = RwLock::new(None);
}

pub const SYSTEM: &'static str = "system";
pub const SYSTEM_SERVICE: &'static str = "system_service";
pub const EVENT: &'static str = "event";

/// Initializes slog for Actors.
///
/// The specified logger will be used as the root logger for all Actors
pub fn init(root_logger: slog::Logger) {
    let mut logger = LOGGER.write().unwrap();
    info!(root_logger, "logging initialized");
    *logger = Some(root_logger);
}

/// Returns the root Actor logger.
///
/// If None is returned, then it means the logger has not yet been initialized via init().
pub fn logger() -> Option<slog::Logger> {
    let logger = LOGGER.read().unwrap();
    match *logger {
        None => None,
        Some(ref logger) => Some(logger.clone()),
    }
}
