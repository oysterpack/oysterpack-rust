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
use serde_json;
use std::io;

use super::*;

op_build_mod!();

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
        .level_for(build::PKG_NAME, log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()?;

    Ok(())
}

lazy_static! {
    pub static ref INIT_FERN: Result<(), fern::InitError> = init_logging();
}

/// Used to run tests. It ensures that logging has been initialized.
pub fn run_test<F: FnOnce() -> ()>(test: F) {
    let _ = *INIT_FERN;
    test()
}

#[test]
fn build_info() {
    run_test(|| {
        info!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
        info!(
            "This is version {}{}, built for {} by {}.",
            build::PKG_VERSION,
            build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
            build::TARGET,
            build::RUSTC_VERSION
        );
        info!(
            "I was built with profile \"{}\", features \"{}\" on {}",
            build::PROFILE,
            build::FEATURES_STR,
            build::BUILT_TIME_UTC
        );
    });
}

#[test]
fn build_get() {
    run_test(|| {
        let build_info = build::get();
        info!("build_info: {:?}", build_info);

        let build_info_json = serde_json::to_string_pretty(&build_info).unwrap();
        info!("build_info_json: {}", build_info_json);
        let build_info2 = serde_json::from_str(&build_info_json).unwrap();
        assert_eq!(build_info, build_info2);

        //TODO: check that build info is mapped coorectly
    });
}
