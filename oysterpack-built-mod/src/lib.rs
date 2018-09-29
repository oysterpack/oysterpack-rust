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

//! This module provides a macro that will generate a public module that contains build-time info
//! that was generated via [oysterpack_built](https://crates.io/crates/oysterpack_built)

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built_mod/0.2.0")]

extern crate chrono;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
#[cfg(test)]
extern crate log;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate fern;
#[cfg(test)]
extern crate serde_json;

use chrono::{DateTime, Utc};
use std::fmt::{self, Display, Formatter};

/// Generate a public module named `build` which includes build-time info generated via
/// [oysterpack_built](https://crates.io/crates/oysterpack_built)
#[macro_export]
macro_rules! op_build_mod {
    () => {
        /// provides build-time information
        pub mod build {
            // The file has been placed there by the build script.
            include!(concat!(env!("OUT_DIR"), "/built.rs"));

            /// Collects the build-time info to construct a Build instance
            pub fn get() -> $crate::Build {
                let mut builder = $crate::BuildBuilder::new();
                builder.timestamp(BUILT_TIME_UTC);
                builder.target(
                    $crate::TargetTriple(TARGET.to_string()),
                    $crate::TargetEnv(CFG_ENV.to_string()),
                    $crate::TargetOperatingSystem {
                        family: CFG_FAMILY.to_string(),
                        os: CFG_OS.to_string(),
                    },
                    $crate::TargetArchitecture(CFG_TARGET_ARCH.to_string()),
                    $crate::Endian(CFG_ENDIAN.to_string()),
                    $crate::PointerWidth(CFG_POINTER_WIDTH.parse().unwrap()),
                );
                if let Some(ci) = CI_PLATFORM.map(|ci_platform| ci_platform.to_string()) {
                    builder.ci_platform($crate::ContinuousIntegrationPlatform(ci));
                }
                if let Some(git_version) = GIT_VERSION.map(|git_version| git_version.to_string()) {
                    builder.git_version($crate::GitVersion(git_version));
                }
                builder.compilation(
                    DEBUG,
                    FEATURES.iter().map(|feature| feature.to_string()).collect(),
                    $crate::CompileOptLevel(OPT_LEVEL.parse().unwrap()),
                    $crate::RustcVersion(RUSTDOC_VERSION.to_string()),
                    $crate::TargetTriple(HOST.to_string()),
                    $crate::BuildProfile(PROFILE.to_string()),
                );
                builder.package($crate::Package {
                    name: PKG_NAME.to_string(),
                    authors: PKG_AUTHORS
                        .split(':')
                        .map(|author| author.to_string())
                        .collect(),
                    description: PKG_DESCRIPTION.to_string(),
                    version: $crate::semver::Version::parse(PKG_VERSION).unwrap(),
                });
                builder.build()
            }
        }
    };
}

/// Build contains the crate's build-time information.
/// It collects the build-time info produced via [oysterpack_built](https://crates.io/crates/oysterpack_built)
/// into a type safe struct.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Build {
    timestamp: DateTime<Utc>,
    target: Target,
    ci_platform: Option<ContinuousIntegrationPlatform>,
    compilation: Compilation,
    git_version: Option<GitVersion>,
    package: Package,
}

impl Build {
    /// When the crate was built
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// The compile target platform
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// The Continuous Integration platform detected during compilation.
    pub fn ci_platform(&self) -> Option<&ContinuousIntegrationPlatform> {
        self.ci_platform.as_ref()
    }

    /// Compilation info
    pub fn compilation(&self) -> &Compilation {
        &self.compilation
    }

    /// If the crate was compiled from within a git-repository, GIT_VERSION contains HEAD's tag.
    /// The short commit id is used if HEAD is not tagged.
    pub fn git_version(&self) -> Option<&GitVersion> {
        self.git_version.as_ref()
    }

    /// Crate package info
    pub fn package(&self) -> &Package {
        &self.package
    }
}

/// Used to build new Build instances.
#[derive(Debug)]
pub struct BuildBuilder {
    timestamp: Option<DateTime<Utc>>,
    target: Option<Target>,
    ci_platform: Option<ContinuousIntegrationPlatform>,
    compilation: Option<Compilation>,
    git_version: Option<GitVersion>,
    package: Option<Package>,
}

impl BuildBuilder {
    /// Constructs a new builder
    pub fn new() -> BuildBuilder {
        BuildBuilder {
            timestamp: None,
            target: None,
            ci_platform: None,
            compilation: None,
            git_version: None,
            package: None,
        }
    }

    /// Set when the crate was compiled.
    /// - timestamp must be formatted per RFC2822
    pub fn timestamp(&mut self, timestamp: &str) {
        let timestamp = DateTime::parse_from_rfc2822(timestamp)
            .map(|ts| ts.with_timezone(&Utc))
            .unwrap();
        self.timestamp = Some(timestamp);
    }

    /// Set compile target info
    pub fn target(
        &mut self,
        triple: TargetTriple,
        env: TargetEnv,
        os: TargetOperatingSystem,
        arch: TargetArchitecture,
        endian: Endian,
        pointer_width: PointerWidth,
    ) {
        self.target = Some(Target {
            triple,
            env,
            os,
            arch,
            endian,
            pointer_width,
        });
    }

    /// The Continuous Integration platform detected during compilation
    pub fn ci_platform(&mut self, ci: ContinuousIntegrationPlatform) {
        self.ci_platform = Some(ci);
    }

    /// Compilation info
    pub fn compilation(
        &mut self,
        debug: bool,
        features: Vec<String>,
        opt_level: CompileOptLevel,
        rustc_version: RustcVersion,
        host: TargetTriple,
        profile: BuildProfile,
    ) {
        self.compilation = Some(Compilation {
            debug,
            features,
            opt_level,
            rustc_version,
            host,
            profile,
        });
    }

    /// Set the GIT project version
    pub fn git_version(&mut self, ver: GitVersion) {
        self.git_version = Some(ver)
    }

    /// Set package info
    pub fn package(&mut self, package: Package) {
        self.package = Some(package)
    }

    /// Produces the Build.
    ///
    /// # Panics
    /// If the Build failed to be constructed
    pub fn build(self) -> Build {
        Build {
            timestamp: self.timestamp.unwrap(),
            target: self.target.unwrap(),
            ci_platform: self.ci_platform,
            compilation: self.compilation.unwrap(),
            git_version: self.git_version,
            package: self.package.unwrap(),
        }
    }
}

/// Platforms are identified by their “target triple” which is the string to inform the compiler
/// what kind of output should be produced.
///
/// The target triple has the general format `<arch><sub>-<vendor>-<sys>-<abi>`, where:
/// - `arch`
///   - x86, arm, thumb, mips, etc.
///   - On UNIXy systems, you can find this with the command uname -m.
/// - `sub`
///   - for example on ARM: v5, v6m, v7a, v7m, etc.
/// - `vendor`
///   - pc, apple, nvidia, ibm, etc.
///   - On linux: usually unknown. On windows: pc. On OSX/iOS: apple
/// - `sys`
///   - none, linux, win32, darwin, cuda, etc.
///   - On UNIXy systems, you can find this with the command uname -s
/// - `abi`
///   - eabi, gnu, android, macho, elf, etc.
///   - On Linux, this refers to the libc implementation which you can find out with ldd --version.
///   - Mac and *BSD systems don't provide multiple ABIs, so this field is omitted.
///   - On Windows, AFAIK there are only two ABIs: gnu and msvc.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetTriple(String);

impl Display for TargetTriple {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Target environment - corresponds to the abi part of the target triple
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetEnv(String);

impl Display for TargetEnv {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Target architecture
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetArchitecture(String);

impl Display for TargetArchitecture {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// endianness
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Endian(String);

impl Display for Endian {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// pointer width
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PointerWidth(u8);

impl Display for PointerWidth {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The target operating system
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetOperatingSystem {
    family: String,
    os: String,
}

impl TargetOperatingSystem {
    /// Operating system family
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Operating system name
    pub fn os(&self) -> &str {
        &self.os
    }
}

/// Compile target info
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Target {
    triple: TargetTriple,
    env: TargetEnv,
    os: TargetOperatingSystem,
    arch: TargetArchitecture,
    endian: Endian,
    pointer_width: PointerWidth,
}

impl Target {
    /// triple
    pub fn triple(&self) -> &TargetTriple {
        &self.triple
    }

    /// env
    pub fn env(&self) -> &TargetEnv {
        &self.env
    }

    /// os
    pub fn os(&self) -> &TargetOperatingSystem {
        &self.os
    }

    /// arch
    pub fn arch(&self) -> &TargetArchitecture {
        &self.arch
    }

    /// endian
    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    /// pointer_width
    pub fn pointer_width(&self) -> &PointerWidth {
        &self.pointer_width
    }
}

/// Continuous Integration platform
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContinuousIntegrationPlatform(String);

impl Display for ContinuousIntegrationPlatform {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Compilation info
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Compilation {
    debug: bool,
    features: Vec<String>,
    opt_level: CompileOptLevel,
    rustc_version: RustcVersion,
    host: TargetTriple,
    profile: BuildProfile,
}

impl Compilation {
    /// build profile debug setting
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// compilation enabled features
    pub fn features(&self) -> &Vec<String> {
        &self.features
    }

    /// build profile optimization level
    pub fn opt_level(&self) -> &CompileOptLevel {
        &self.opt_level
    }

    /// rustc version used
    pub fn rustc_version(&self) -> &RustcVersion {
        &self.rustc_version
    }

    /// host triple for rustc
    pub fn host_triple(&self) -> &TargetTriple {
        &self.host
    }

    /// build profile used
    pub fn profile(&self) -> &BuildProfile {
        &self.profile
    }
}

/// The output of rustc -V
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RustcVersion(String);

impl Display for RustcVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Build profile used
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildProfile(String);

impl Display for BuildProfile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Value of OPT_LEVEL for the profile used during compilation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompileOptLevel(u8);

impl Display for CompileOptLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Contains HEAD's tag. The short commit id is used if HEAD is not tagged.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitVersion(String);

impl Display for GitVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Crate's package info, which is specified in cargo.toml.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Package {
    name: String,
    authors: Vec<String>,
    description: String,
    version: semver::Version,
}

impl Package {
    /// Package constructor
    pub fn new(
        name: String,
        authors: Vec<String>,
        description: String,
        version: semver::Version,
    ) -> Package {
        Package {
            name,
            authors,
            description,
            version,
        }
    }

    /// Package name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Package authors
    pub fn authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    /// Package description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Package version
    pub fn version(&self) -> &semver::Version {
        &self.version
    }
}

#[cfg(test)]
mod tests;
