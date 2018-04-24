// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate slog;

extern crate actix;
extern crate futures;

use super::*;

#[test]
fn root_logger() {
    let mut sys = System::new("sys");
    let test = super::root_logger().map(|logger| {
        info!(logger, "root_logger SUCCESS #1");
        logger
    });
    match sys.run_until_complete(test) {
        Ok(logger) => info!(logger, "root_logger SUCCESS #2"),
        Err(err) => panic!(err),
    }
}

#[test]
fn set_root_logger() {
    // TODO: write log to a string buffer to inspect

    let mut sys = System::new("sys");
    let init_root_logger = super::root_logger()
        .and_then(|logger| {
            let logger = logger.new(o!(ACTOR_ID => 456));
            super::set_root_logger(logger)
        })
        .and_then(|_| {
            super::root_logger().map(|logger| {
                info!(logger, "set_root_logger SUCCESS #1");
                logger
            })
        });
    match sys.run_until_complete(init_root_logger) {
        Ok(logger) => info!(logger, "set_root_logger SUCCESS #2"),
        Err(err) => panic!(err),
    }
}
