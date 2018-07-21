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

//! # OysterPack Built provides the ability to gather information about the crate's cargo build.

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built/0.1.0")]

extern crate built;
extern crate chrono;
extern crate semver;

use chrono::{DateTime, ParseResult, Utc};
use semver::{SemVerError, Version};

pub mod build;

pub use build::write_app_built_file;
pub use build::write_library_built_file;

#[cfg(test)]
mod tests;

/// Parses timestamps that are formatted according to **RFC 2822**, `Sat, 21 Jul 2018 15:59:46 GMT`.
///
/// [built](https://crates.io/crates/built) formats timestamps using RFC 2822.
pub fn parse_datetime(ts: &str) -> ParseResult<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(ts).map(|ts| ts.with_timezone(&Utc))
}

/// Parses versions according to semver format.
///
/// [built](https://crates.io/crates/built) formats the `PKG_VERSION` using semver.
pub fn parse_pkg_version(ver: &str) -> Result<Version, SemVerError> {
    Version::parse(ver)
}
