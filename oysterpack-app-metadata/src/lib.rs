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

//! Defines the application metadata domain model.
//!
//! The [op_build_mod!](macro.op_build_mod.html) macro is used in conjunction with [oysterpack_built](https://crates.io/crates/oysterpack_built)

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_app_metadata/0.1.0")]

extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate serde_json;

#[macro_use]
mod macros;

pub mod metadata;

pub use metadata::Build;
pub use metadata::BuildBuilder;
pub use metadata::BuildProfile;
pub use metadata::Compilation;
pub use metadata::CompileOptLevel;
pub use metadata::ContinuousIntegrationPlatform;
pub use metadata::Endian;
pub use metadata::GitVersion;
pub use metadata::Package;
pub use metadata::PointerWidth;
pub use metadata::RustcVersion;
pub use metadata::Target;
pub use metadata::TargetArchitecture;
pub use metadata::TargetEnv;
pub use metadata::TargetOperatingSystem;
pub use metadata::TargetTriple;

#[cfg(test)]
mod tests;
