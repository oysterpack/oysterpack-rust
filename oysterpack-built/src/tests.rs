// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate chrono;
extern crate semver;

use self::chrono::{DateTime, ParseResult, Utc};
use self::semver::{SemVerError, Version};

use build;

#[test]
fn build_info() {
    println!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
    println!(
        "This is version {}{}, built for {} by {}.",
        parse_pkg_version(build::PKG_VERSION).unwrap(),
        build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
        build::TARGET,
        build::RUSTC_VERSION
    );
    println!(
        "I was built with profile \"{}\", features \"{}\" on {}",
        build::PROFILE,
        build::FEATURES_STR,
        parse_datetime(build::BUILT_TIME_UTC).unwrap()
    );
}

/// Parses timestamps that are formatted according to **RFC 2822**, `Sat, 21 Jul 2018 15:59:46 GMT`.
///
/// [built](https://crates.io/crates/built) formats timestamps using RFC 2822:
/// - `BUILT_TIME_UTC`
fn parse_datetime(ts: &str) -> ParseResult<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(ts).map(|ts| ts.with_timezone(&Utc))
}

/// Parses versions according to semver format.
///
/// [built](https://crates.io/crates/built) formats the `PKG_VERSION` using semver.
fn parse_pkg_version(ver: &str) -> Result<Version, SemVerError> {
    Version::parse(ver)
}
