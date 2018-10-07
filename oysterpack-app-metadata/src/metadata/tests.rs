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
use tests::*;

//#[test]
//fn build_info() {
//    run_test(|| {
//        info!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
//        info!(
//            "This is version {}{}, built for {} by {}.",
//            build::PKG_VERSION,
//            build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
//            build::TARGET,
//            build::RUSTC_VERSION
//        );
//        info!(
//            "I was built with profile \"{}\", features \"{}\" on {}",
//            build::PROFILE,
//            build::FEATURES_STR,
//            build::BUILT_TIME_UTC
//        );
//        info!(
//            "RUSTC({}),RUSTDOC({}),RUSTDOC_VERSION({}),NUM_JOBS({})",
//            build::RUSTC,
//            build::RUSTDOC,
//            build::RUSTDOC_VERSION,
//            build::NUM_JOBS
//        );
//    });
//}

//pub mod build {
//    use metadata::{self, *};
//
//    // The file has been placed there by the build script.
//    include!(concat!(env!("OUT_DIR"), "/built.rs"));
//
//    /// Collects the build-time info to construct a new Build instance
//    pub fn get() -> Build {
//        let mut builder = BuildBuilder::new();
//        builder.timestamp(
//            ::chrono::DateTime::parse_from_rfc2822(BUILT_TIME_UTC)
//                .map(|ts| ts.with_timezone(&::chrono::Utc))
//                .unwrap(),
//        );
//        builder.target(
//            TargetTriple::new(TARGET),
//            TargetEnv::new(CFG_ENV),
//            TargetOperatingSystem::new(CFG_FAMILY.to_string(), CFG_OS.to_string()),
//            TargetArchitecture::new(CFG_TARGET_ARCH),
//            Endian::new(CFG_ENDIAN),
//            PointerWidth::new(CFG_POINTER_WIDTH.parse().unwrap()),
//        );
//        if let Some(ci) = CI_PLATFORM {
//            builder.ci_platform(ContinuousIntegrationPlatform::new(ci));
//        }
//        if let Some(git_version) = GIT_VERSION {
//            builder.git_version(GitVersion::new(git_version));
//        }
//        builder.compilation(
//            DEBUG,
//            FEATURES.iter().map(|feature| feature.to_string()).collect(),
//            CompileOptLevel::new(OPT_LEVEL.parse().unwrap()),
//            RustcVersion::new(RUSTC_VERSION),
//            TargetTriple::new(HOST),
//            BuildProfile::new(PROFILE),
//        );
//        let dependencies: Vec<metadata::PackageId> =
//            ::serde_json::from_str(DEPENDENCIES_JSON).unwrap();
//        builder.package(
//            PKG_NAME.to_string(),
//            PKG_AUTHORS
//                .split(':')
//                .map(|author| author.to_string())
//                .collect(),
//            PKG_DESCRIPTION.to_string(),
//            ::semver::Version::parse(PKG_VERSION).unwrap(),
//            PKG_HOMEPAGE.to_string(),
//            dependencies,
//        );
//        builder.build()
//    }
//}
//
//#[test]
//fn build_get() {
//    run_test(|| {
//        let build_info = build::get();
//        info!("build_info: {:?}", build_info);
//
//        let build_info_json = serde_json::to_string_pretty(&build_info).unwrap();
//        info!("build_info_json: {}", build_info_json);
//        let build_info2 = serde_json::from_str(&build_info_json).unwrap();
//        assert_eq!(build_info, build_info2);
//
//        let timestamp = ::chrono::DateTime::parse_from_rfc2822(build::BUILT_TIME_UTC)
//            .map(|ts| ts.with_timezone(&::chrono::Utc))
//            .unwrap();
//
//        assert_eq!(timestamp, build_info.timestamp());
//
//        assert_eq!(build::TARGET, build_info.target().triple().get());
//        assert_eq!(build::CFG_ENV, build_info.target().env().get());
//        assert_eq!(build::CFG_TARGET_ARCH, build_info.target().arch().get());
//        assert_eq!(
//            build::CFG_POINTER_WIDTH.parse::<u8>().unwrap(),
//            build_info.target().pointer_width().get()
//        );
//        assert_eq!(build::CFG_ENDIAN, build_info.target().endian().get());
//        assert_eq!(build::CFG_OS, build_info.target().os().os());
//        assert_eq!(build::CFG_FAMILY, build_info.target().os().family());
//
//        assert_eq!(
//            build::CI_PLATFORM,
//            build_info.ci_platform().map(|ci| ci.get())
//        );
//
//        assert_eq!(build::DEBUG, build_info.compilation().debug());
//        assert_eq!(
//            build::FEATURES
//                .iter()
//                .map(|s| s.to_string())
//                .collect::<Vec<String>>(),
//            *build_info.compilation().features()
//        );
//        assert_eq!(
//            build::OPT_LEVEL.parse::<u8>().unwrap(),
//            build_info.compilation().opt_level().get()
//        );
//        assert_eq!(
//            build::RUSTC_VERSION,
//            build_info.compilation().rustc_version().get()
//        );
//        assert_eq!(build::HOST, build_info.compilation().host_triple().get());
//        assert_eq!(build::PROFILE, build_info.compilation().profile().get());
//
//        assert_eq!(
//            build::GIT_VERSION,
//            build_info.git_version().map(|v| v.get())
//        );
//
//        assert_eq!(
//            semver::Version::parse(build::PKG_VERSION).unwrap(),
//            *build_info.package().version()
//        );
//        assert_eq!(
//            build_info.package().version().major,
//            build::PKG_VERSION_MAJOR.parse::<u64>().unwrap()
//        );
//        assert_eq!(
//            build_info.package().version().minor,
//            build::PKG_VERSION_MINOR.parse::<u64>().unwrap()
//        );
//        assert_eq!(
//            build_info.package().version().patch,
//            build::PKG_VERSION_PATCH.parse::<u64>().unwrap()
//        );
//
//        match build::PKG_VERSION_PRE {
//            "" => assert!(build_info.package().version().pre.is_empty()),
//            pre => assert_eq!(
//                build_info
//                    .package()
//                    .version()
//                    .pre
//                    .iter()
//                    .map(|v| v.to_string())
//                    .collect::<Vec<String>>(),
//                pre.split('.')
//                    .map(|s| s.to_string())
//                    .collect::<Vec<String>>()
//            ),
//        }
//
//        assert_eq!(build::PKG_NAME, build_info.package().name());
//        assert_eq!(
//            build::PKG_AUTHORS
//                .split(':')
//                .map(|s| s.to_string())
//                .collect::<Vec<String>>()
//                .as_slice(),
//            build_info.package().authors()
//        );
//        assert_eq!(build::PKG_HOMEPAGE, build_info.package().homepage());
//        assert_eq!(build::PKG_DESCRIPTION, build_info.package().description());
//    });
//}
