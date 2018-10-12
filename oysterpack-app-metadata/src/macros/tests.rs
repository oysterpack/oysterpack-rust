// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

#![allow(warnings)]

use metadata::{Build, ContinuousIntegrationPlatform, PackageId};
use tests::*;

#[test]
fn build_on_ci_platform() {
    op_build_mod!(build, "build_on_ci_platform.rs");
    run_test(|| {
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
    run_test(|| {
        let build_md: Build = build::get();
        assert_eq!(build_md.ci_platform(), None);
    });
}

#[test]
fn build_multi_authors() {
    op_build_mod!(build, "build_multi_authors.rs");
    run_test(|| {
        let build_md: Build = build::get();
        let authors = build_md.package().authors();
        assert_eq!(authors.len(), 2);
        assert!(authors.contains(&"Alfio Zappala <oysterpack.inc@gmail.com>".to_string()));
        assert!(
            authors
                .contains(&"Roman Alexander Zappala <roman.a.zappala@oysterpack.com>".to_string())
        );
    });
}
