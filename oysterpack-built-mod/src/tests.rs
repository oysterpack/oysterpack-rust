// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

use chrono;
use fern;
use log;
use semver;
use serde_json;
use std::io;

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
        info!(
            "RUSTC({}),RUSTDOC({}),RUSTDOC_VERSION({}),NUM_JOBS({})",
            build::RUSTC,
            build::RUSTDOC,
            build::RUSTDOC_VERSION,
            build::NUM_JOBS
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

        let timestamp = ::chrono::DateTime::parse_from_rfc2822(build::BUILT_TIME_UTC)
            .map(|ts| ts.with_timezone(&::chrono::Utc))
            .unwrap();

        assert_eq!(timestamp, build_info.timestamp());

        assert_eq!(build::TARGET, build_info.target().triple().get());
        assert_eq!(build::CFG_ENV, build_info.target().env().get());
        assert_eq!(build::CFG_TARGET_ARCH, build_info.target().arch().get());
        assert_eq!(
            build::CFG_POINTER_WIDTH.parse::<u8>().unwrap(),
            build_info.target().pointer_width().get()
        );
        assert_eq!(build::CFG_ENDIAN, build_info.target().endian().get());
        assert_eq!(build::CFG_OS, build_info.target().os().os());
        assert_eq!(build::CFG_FAMILY, build_info.target().os().family());

        assert_eq!(
            build::CI_PLATFORM,
            build_info.ci_platform().map(|ci| ci.get())
        );

        assert_eq!(build::DEBUG, build_info.compilation().debug());
        assert_eq!(
            build::FEATURES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            *build_info.compilation().features()
        );
        assert_eq!(
            build::OPT_LEVEL.parse::<u8>().unwrap(),
            build_info.compilation().opt_level().get()
        );
        assert_eq!(
            build::RUSTC_VERSION,
            build_info.compilation().rustc_version().get()
        );
        assert_eq!(build::HOST, build_info.compilation().host_triple().get());
        assert_eq!(build::PROFILE, build_info.compilation().profile().get());

        assert_eq!(
            build::GIT_VERSION,
            build_info.git_version().map(|v| v.get())
        );

        assert_eq!(
            semver::Version::parse(build::PKG_VERSION).unwrap(),
            *build_info.package().version()
        );
        assert_eq!(
            build_info.package().version().major,
            build::PKG_VERSION_MAJOR.parse::<u64>().unwrap()
        );
        assert_eq!(
            build_info.package().version().minor,
            build::PKG_VERSION_MINOR.parse::<u64>().unwrap()
        );
        assert_eq!(
            build_info.package().version().patch,
            build::PKG_VERSION_PATCH.parse::<u64>().unwrap()
        );

        match build::PKG_VERSION_PRE {
            "" => assert!(build_info.package().version().pre.is_empty()),
            pre => assert_eq!(
                build_info
                    .package()
                    .version()
                    .pre
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
                pre.split('.')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            ),
        }

        assert_eq!(build::PKG_NAME, build_info.package().name());
        assert_eq!(
            build::PKG_AUTHORS
                .split(':')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .as_slice(),
            build_info.package().authors()
        );
        assert_eq!(build::PKG_HOMEPAGE, build_info.package().homepage());
        assert_eq!(build::PKG_DESCRIPTION, build_info.package().description());
    });
}

#[test]
fn build_consts() {}
