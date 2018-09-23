// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit test support

use build;

#[test]
fn build_info() {
    println!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
    println!(
        "This is version {}{}, built for {} by {}.",
        build::PKG_VERSION,
        build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
        build::TARGET,
        build::RUSTC_VERSION
    );
    println!(
        "I was built with profile \"{}\", features \"{}\" on {}",
        build::PROFILE,
        build::FEATURES_STR,
        build::BUILT_TIME_UTC
    );
}
