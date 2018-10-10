// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

use semver;
use serde_json;

use super::PackageId;
use tests::*;

op_build_mod!();

#[test]
fn parsing_dependencies_graphviz_dot_into_package_ids() {
    let dot = r#"
    digraph {
    0 [label="oysterpack_app_template=0.1.0"]
    1 [label="log=0.4.5"]
    2 [label="serde=1.0.79"]
    3 [label="oysterpack_app_metadata=0.1.0"]
    4 [label="serde_derive=1.0.79"]
    5 [label="fern=0.5.6"]
    6 [label="semver=0.9.0"]
    7 [label="chrono=0.4.6"]
    8 [label="serde_json=1.0.31"]
    9 [label="ryu=0.2.6"]
    10 [label="itoa=0.4.3"]
    11 [label="num-integer=0.1.39"]
    12 [label="time=0.1.40"]
    13 [label="num-traits=0.2.6"]
    14 [label="libc=0.2.43"]
    15 [label="semver-parser=0.7.0"]
    16 [label="proc-macro2=0.4.19"]
    17 [label="syn=0.15.6"]
    18 [label="quote=0.6.8"]
    19 [label="unicode-xid=0.1.0"]
    20 [label="cfg-if=0.1.5"]
    0 -> 1
    0 -> 2
    0 -> 3
    0 -> 4
    0 -> 5
    0 -> 6
    0 -> 7
    0 -> 8
    8 -> 2
    8 -> 9
    8 -> 10
    7 -> 11
    7 -> 2
    7 -> 12
    7 -> 13
    12 -> 14
    11 -> 13
    6 -> 15
    6 -> 2
    5 -> 1
    4 -> 16
    4 -> 17
    4 -> 18
    18 -> 16
    17 -> 19
    17 -> 18
    17 -> 16
    16 -> 19
    3 -> 2
    3 -> 7
    3 -> 6
    3 -> 4
    1 -> 20
}"#;

    run_test(|| {
        let mut package_ids: Vec<PackageId> = dot
            .lines()
            .filter(|line| !line.contains("->") && line.contains("["))
            .skip(1)
            .map(|line| {
                let line = &line[line.find('"').unwrap() + 1..];
                let line = &line[..line.find('"').unwrap()];
                let tokens: Vec<&str> = line.split("=").collect();
                PackageId::new(
                    tokens.get(0).unwrap().to_string(),
                    semver::Version::parse(tokens.get(1).unwrap()).unwrap(),
                )
            }).collect();
        package_ids.sort();
        let package_ids: Vec<String> = package_ids.iter().map(|id| id.to_string()).collect();
        info!("package_ids : {}", package_ids.join("\n"));
    });
}

#[test]
fn build_mod() {
    run_test(|| {
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
