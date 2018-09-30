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

//! This module provides the `op_build_mod!()` macro that will generate a public module named `build`
//! that contains build-time info. The build-time info is extracted during compilation via
//! [oysterpack_built](https://crates.io/crates/oysterpack_built).
//!
//! The generated `build` module will consist of:
//! - constants for each piece of build metadata
//!
//! Constant | Type | Description
//! -------- | ---- | -----------
//! BUILT_TIME_UTC|&str|The built-time in RFC822, UTC
//! CFG_ENDIAN|&str|The endianness, given by cfg!(target_endian).
//! CFG_ENV|&str|The toolchain-environment, given by cfg!(target_env).
//! CFG_FAMILY|&str|The OS-family, given by cfg!(target_family).
//! CFG_OS|&str|The operating system, given by cfg!(target_os).
//! CFG_POINTER_WIDTH|u8|The pointer width, given by cfg!(target_pointer_width).
//! CFG_TARGET_ARCH|&str|The target architecture, given by cfg!(target_arch).
//! CI_PLATFORM|Option<&str>|The Continuous Integration platform detected during compilation.
//! DEBUG|bool|Value of DEBUG for the profile used during compilation.
//! FEATURES|\[&str; N\]|The features that were enabled during compilation.
//! FEATURES_STR|&str|The features as a comma-separated string.
//! GIT_VERSION|Option<&str>|If the crate was compiled from within a git-repository, GIT_VERSION contains HEAD's tag. The short commit id is used if HEAD is not tagged.
//! HOST|&str|The host triple of the rust compiler.
//! NUM_JOBS|u32|The parallelism that was specified during compilation.
//! OPT_LEVEL|&str|Value of OPT_LEVEL for the profile used during compilation.
//! PKG_AUTHORS|&str|A colon-separated list of authors.
//! PKG_DESCRIPTION|&str|The description.
//! PKG_HOMEPAGE|&str|The homepage.
//! PKG_NAME|&str|The name of the package.
//! PKG_VERSION|&str|The full version.
//! PKG_VERSION_MAJOR|&str|The major version.
//! PKG_VERSION_MINOR|&str|The minor version.
//! PKG_VERSION_PATCH|&str|The patch version.
//! PKG_VERSION_PRE|&str|The pre-release version.
//! PROFILE|&str|release for release builds, debug for other builds.
//! RUSTC|&str|The compiler that cargo resolved to use.
//! RUSTC_VERSION|&str|The output of rustc -V
//! RUSTDOC|&str|The documentation generator that cargo resolved to use.
//! RUSTDOC_VERSION|&str|The output of rustdoc -V
//!
//! - `fn get() -> Build`
//!     - [Build](struct.Build.html) provides a consolidated view of the build-time metadata.
//!       This makes it easier to work with the build-time metadata in a typesafe manner.
//!
//! **NOTE:** The `op_build_mod!()` depends on the following dependencies in order to compile:
//! - [semver](https://crates.io/crates/semver)
//! - [chrono](https://crates.io/crates/chrono)
//!

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_built_mod/0.2.2")]

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

/// Generate a public module named `build` which includes build-time info generated via
/// [oysterpack_built](https://crates.io/crates/oysterpack_built)
#[macro_export]
macro_rules! op_build_mod {
    () => {
        /// provides build-time information
        pub mod build {
            // The file has been placed there by the build script.
            include!(concat!(env!("OUT_DIR"), "/built.rs"));

            /// Collects the build-time info to construct a new Build instance
            pub fn get() -> $crate::Build {
                let mut builder = $crate::BuildBuilder::new();
                builder.timestamp(
                    ::chrono::DateTime::parse_from_rfc2822(BUILT_TIME_UTC)
                        .map(|ts| ts.with_timezone(&::chrono::Utc))
                        .unwrap(),
                );
                builder.target(
                    $crate::TargetTriple::new(TARGET),
                    $crate::TargetEnv::new(CFG_ENV),
                    $crate::TargetOperatingSystem::new(CFG_FAMILY.to_string(), CFG_OS.to_string()),
                    $crate::TargetArchitecture::new(CFG_TARGET_ARCH),
                    $crate::Endian::new(CFG_ENDIAN),
                    $crate::PointerWidth::new(CFG_POINTER_WIDTH.parse().unwrap()),
                );
                if let Some(ci) = CI_PLATFORM {
                    builder.ci_platform($crate::ContinuousIntegrationPlatform::new(ci));
                }
                if let Some(git_version) = GIT_VERSION {
                    builder.git_version($crate::GitVersion::new(git_version));
                }
                builder.compilation(
                    DEBUG,
                    FEATURES.iter().map(|feature| feature.to_string()).collect(),
                    $crate::CompileOptLevel::new(OPT_LEVEL.parse().unwrap()),
                    $crate::RustcVersion::new(RUSTC_VERSION),
                    $crate::TargetTriple::new(HOST),
                    $crate::BuildProfile::new(PROFILE),
                );
                builder.package(
                    PKG_NAME.to_string(),
                    PKG_AUTHORS
                        .split(':')
                        .map(|author| author.to_string())
                        .collect(),
                    PKG_DESCRIPTION.to_string(),
                    ::semver::Version::parse(PKG_VERSION).unwrap(),
                    PKG_HOMEPAGE.to_string(),
                );
                builder.build()
            }
        }
    };
}

/// macro that generates a new type for a String
macro_rules! op_tuple_struct_string {
    (
        $(#[$outer:meta])*
        $name:ident
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name (String);

        impl $name {
            /// TargetTriple constructor
            pub fn new(value: &str) -> $name {
                $name(value.to_string())
            }

            /// get the underlying value
            pub fn get(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

/// macro that generates a new type where the underlying value implements Copy
macro_rules! op_tuple_struct_copy {
    (
        $(#[$outer:meta])*
        $name:ident($T:ty)
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name ($T);

        impl $name {
            /// TargetTriple constructor
            pub fn new(value: $T) -> $name {
                $name(value)
            }

            /// get the underlying value
            pub fn get(&self) -> $T {
                self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// Build provides a consolidated view of the crate's build-time metadata.
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
    pub fn timestamp(&mut self, timestamp: DateTime<Utc>) {
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
    pub fn package(
        &mut self,
        name: String,
        authors: Vec<String>,
        description: String,
        version: semver::Version,
        homepage: String,
    ) {
        self.package = Some(Package {
            name,
            authors,
            description,
            version,
            homepage,
        })
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

op_tuple_struct_string! {
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
    TargetTriple
}

op_tuple_struct_string! {
    /// Target environment - corresponds to the abi part of the target triple
    TargetEnv
}

op_tuple_struct_string! {
    /// Target architecture
    TargetArchitecture
}

op_tuple_struct_string! {
    /// endianness
    Endian
}

op_tuple_struct_copy! {
    /// pointer width
    PointerWidth(u8)
}

/// The target operating system
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetOperatingSystem {
    family: String,
    os: String,
}

impl TargetOperatingSystem {
    /// TargetOperatingSystem constructor
    pub fn new(family: String, os: String) -> TargetOperatingSystem {
        TargetOperatingSystem { family, os }
    }

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

op_tuple_struct_string! {
    /// Continuous Integration platform
    ContinuousIntegrationPlatform
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

op_tuple_struct_string! {
    /// The output of rustc -V
    RustcVersion
}

op_tuple_struct_string! {
    /// Build profile used for compilation
    BuildProfile
}

op_tuple_struct_copy!{
    /// Value of OPT_LEVEL for the profile used during compilation.
    CompileOptLevel(u8)
}

op_tuple_struct_string! {
    /// Contains HEAD's tag. The short commit id is used if HEAD is not tagged.
    GitVersion
}

/// Crate's package info, which is specified in cargo.toml.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Package {
    name: String,
    authors: Vec<String>,
    description: String,
    version: semver::Version,
    homepage: String,
}

impl Package {
    /// Package constructor
    pub fn new(
        name: String,
        authors: Vec<String>,
        description: String,
        version: semver::Version,
        homepage: String,
    ) -> Package {
        Package {
            name,
            authors,
            description,
            version,
            homepage,
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

    /// Package homepage
    pub fn homepage(&self) -> &str {
        &self.homepage
    }
}

#[cfg(test)]
mod tests;
