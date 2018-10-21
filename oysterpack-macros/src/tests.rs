// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit test support

use chrono;
use fern;
use log;
use std::io;

pub const MODULE_NAME: &str = "oysterpack_macros";

fn init_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.6f]"),
                record.level(),
                record.target(),
                message
            ))
        }).level(log::LevelFilter::Warn)
        .level_for(MODULE_NAME, log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()?;

    Ok(())
}

lazy_static! {
    pub static ref INIT_FERN: Result<(), fern::InitError> = init_logging();
}

pub fn run_test<F: FnOnce() -> ()>(test: F) {
    let _ = *INIT_FERN;
    test()
}

#[test]
fn compiles() {
    run_test(|| {
        info!("it compiles :)")
    });
}
