// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

extern crate chrono;
extern crate fern;
extern crate syslog;

use self::syslog::{unix_custom, Facility, Severity};

use super::*;

use std::io;

fn init_logging() -> Result<(), fern::InitError> {
    //        fern::Dispatch::new()
    //            .format(|out, message, record| {
    //                out.finish(format_args!(
    //                    "{}[{}][{}] {}",
    //                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.6f]"),
    //                    record.level(),
    //                    record.target(),
    //                    message
    //                ))
    //            })
    //            .level(log::LevelFilter::Warn)
    //            .level_for("oysterpack_actors", log::LevelFilter::Debug)
    //            .chain(io::stdout())
    //            .apply()?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("[{}] {}", record.target(), message))
        }).chain(syslog::unix_custom(
            syslog::Facility::LOG_USER,
            "/run/systemd/journal/syslog",
        )?).level(log::LevelFilter::Warn)
        .level_for("oysterpack_actors", log::LevelFilter::Debug)
        .apply()?;

    Ok(())
}

lazy_static! {
    pub static ref INIT_FERN: Result<(), fern::InitError> = init_logging();
}

pub fn run_test(test: fn() -> ()) {
    let _ = *INIT_FERN;
    test()
}
