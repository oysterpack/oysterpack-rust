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
//! It is meant to be used in conjunction with [oysterpack_built](https://crates.io/crates/oysterpack_built).
//! [oysterpack_built](https://crates.io/crates/oysterpack_built) generates the application build
//! metadata source file. This crate is used to load the application build netadata via the
//! [op_build_mod](macro.op_build_mod.html) macro.
//!
//! ## Application Build Metadata Domain Model
//! ![uml](ml.svg)

#![deny(missing_docs, missing_debug_implementations, warnings)]
#![doc(html_root_url = "https://docs.rs/oysterpack_app_metadata/0.3.2")]

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;

#[macro_use]
mod macros;

pub mod metadata;

pub use crate::metadata::Build;
pub use crate::metadata::BuildProfile;
pub use crate::metadata::Compilation;
pub use crate::metadata::CompileOptLevel;
pub use crate::metadata::ContinuousIntegrationPlatform;
pub use crate::metadata::Endian;
pub use crate::metadata::GitVersion;
pub use crate::metadata::Package;
pub use crate::metadata::PackageId;
pub use crate::metadata::PointerWidth;
pub use crate::metadata::RustcVersion;
pub use crate::metadata::Target;
pub use crate::metadata::TargetArchitecture;
pub use crate::metadata::TargetEnv;
pub use crate::metadata::TargetOperatingSystem;
pub use crate::metadata::TargetTriple;

#[cfg(test)]
op_tests_mod!();
