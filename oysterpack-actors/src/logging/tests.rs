// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate slog;
extern crate sloggers;

use super::*;

#[test]
fn init_logger() {
    // When logging has not yet been initialized
    // Then None is returned
    assert!(super::logger().is_none());

    // Given logging is initialized
    init(new_logger());
    // Then the logger is stored
    assert!(super::logger().is_some());
    // And the logger can be retrieved
    match super::logger() {
        Some(logger) => {
            info!(logger, "LOGGING HAS BEEN INITIALIZED");
            info!(logger, "SUCCESS");
        }
        None => panic!("Logger should have been initialized"),
    }
}

/// Creates a new logger for testing purposes
pub(crate) fn new_logger() -> slog::Logger {
    use self::sloggers::Build;
    use self::sloggers::terminal::{Destination, TerminalLoggerBuilder};
    use self::sloggers::types::{Format, Severity};

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stderr);
    builder.format(Format::Full);

    builder.build().unwrap()
}
