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

// TODO: write log to a string buffer to inspect

#[test]
fn root_logger() {
    let logger = super::root_logger();
    info!(logger, "root_logger SUCCESS #1");
    warn!(logger, "root_logger SUCCESS #1");
}

#[test]
fn set_root_logger() {
    super::set_root_logger(super::root_logger().new(o!(ACTOR_ID => 456)));
    let logger = super::root_logger();
    info!(logger, "set_root_logger SUCCESS #1");
    warn!(logger, "set_root_logger SUCCESS #1");
}
