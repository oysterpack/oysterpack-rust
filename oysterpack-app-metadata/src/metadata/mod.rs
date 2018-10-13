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

//! Application build metadata domain model

use chrono::{DateTime, Utc};
use semver;
use std::fmt;

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
        dependencies: Vec<PackageId>,
    ) {
        self.package = Some(Package {
            id: PackageId { name, version },
            authors,
            description,
            homepage,
            dependencies,
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
    id: PackageId,
    authors: Vec<String>,
    description: String,
    homepage: String,
    dependencies: Vec<PackageId>,
}

impl Package {
    /// Package constructor
    pub fn new(
        id: PackageId,
        authors: Vec<String>,
        description: String,
        homepage: String,
        dependencies: Vec<PackageId>,
    ) -> Package {
        Package {
            id,
            authors,
            description,
            homepage,
            dependencies,
        }
    }

    /// Package id
    pub fn id(&self) -> &PackageId {
        &self.id
    }

    /// Package name
    pub fn name(&self) -> &str {
        &self.id.name
    }

    /// Package version
    pub fn version(&self) -> &semver::Version {
        &self.id.version
    }

    /// Package authors
    pub fn authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    /// Package description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Package homepage
    pub fn homepage(&self) -> &str {
        &self.homepage
    }

    /// Effective package dependencies
    pub fn dependencies(&self) -> &[PackageId] {
        self.dependencies.as_slice()
    }
}

/// Identifier for a specific version of a package.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct PackageId {
    name: String,
    version: semver::Version,
}

impl PackageId {
    /// PackageId constructor
    pub fn new(name: String, version: semver::Version) -> PackageId {
        PackageId { name, version }
    }

    /// Package name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Package version
    pub fn version(&self) -> &semver::Version {
        &self.version
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests;
