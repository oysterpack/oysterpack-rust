// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

#![allow(warnings)]

use crate::tests::run_test;
use oysterpack_app_metadata::{Build, ContinuousIntegrationPlatform, PackageId};
use semver;
use serde_json;

#[test]
fn build_on_ci_platform() {
    op_build_mod!(build, "build_on_ci_platform.rs");
    run_test("build_on_ci_platform", || {
        let build_md: Build = build::get();
        assert_eq!(
            build_md.ci_platform(),
            Some(ContinuousIntegrationPlatform::new("Travis")).as_ref()
        );
        assert_eq!(build_md.package().dependencies().len(), 17);
        assert!(
            !build_md
                .package()
                .dependencies()
                .contains(build_md.package().id()),
            "Dependencies should exclude itself."
        );
    });
}

#[test]
fn build_no_ci_platform() {
    op_build_mod!(build, "build_no_ci_platform.rs");
    run_test("build_no_ci_platform", || {
        let build_md: Build = build::get();
        assert_eq!(build_md.ci_platform(), None);
    });
}

#[test]
fn build_multi_authors() {
    op_build_mod!(build, "build_multi_authors.rs");
    run_test("build_multi_authors", || {
        let build_md: Build = build::get();
        let authors = build_md.package().authors();
        assert_eq!(authors.len(), 2);
        assert!(authors.contains(&"Alfio Zappala <oysterpack.inc@gmail.com>".to_string()));
        assert!(authors
            .contains(&"Roman Alexander Zappala <roman.a.zappala@oysterpack.com>".to_string()));
    });
}

#[test]
fn build_mod() {
    op_build_mod!(build, "build_no_ci_platform.rs");
    run_test("build_mod", || {
        info!(
            "RUSTC({}),RUSTDOC({}),RUSTDOC_VERSION({}),NUM_JOBS({}),FEATURES_STR({})",
            build::RUSTC,
            build::RUSTDOC,
            build::RUSTDOC_VERSION,
            build::NUM_JOBS,
            build::FEATURES_STR
        );

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
