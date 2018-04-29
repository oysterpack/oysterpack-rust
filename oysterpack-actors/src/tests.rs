// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! POC tests

extern crate chrono;
extern crate fern;

use super::*;

use std::io;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.6f]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("oysterpack_actors", log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()?;
    Ok(())
}

lazy_static! {
 pub static ref INIT_FERN : Result<(), fern::InitError> = setup_logger();
}

pub fn run_test(test: fn() -> ()) {
    let _ = *INIT_FERN;
    test()
}
