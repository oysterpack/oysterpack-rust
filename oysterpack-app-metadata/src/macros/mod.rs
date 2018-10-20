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

//! macros

/// Generates a public module which includes build-time info generated via
/// [oysterpack_built](https://crates.io/crates/oysterpack_built).
///
/// This macro is intended to be used by application binary crates that use
/// [oysterpack_built](https://crates.io/crates/oysterpack_built) as a build dependency to collect
/// application metadata at build-time.
///
/// The macro can be invoked in 3 different ways:
/// - `op_build_mod!()`
///
///     ```ignore
///         pub mod build {
///             include!(concat!(env!("OUT_DIR"), "/built.rs"));
///
///             /// Collects the build-time info to construct a new Build instance
///             pub fn get() -> $crate::Build { ... }
///         }
///     ```
///
/// - `op_build_mod!($name:ident)`
///
///     ```ignore
///         pub mod $name {
///             include!(concat!(env!("OUT_DIR"), "/built.rs"));
///
///             /// Collects the build-time info to construct a new Build instance
///             pub fn get() -> $crate::Build { ... }
///         }
///     ```
///
/// - `op_build_mod!($name:ident, $file:expr)`
///
///     ```ignore
///         pub mod $name {
///             include!($file));
///
///             /// Collects the build-time info to construct a new Build instance
///             pub fn get() -> $crate::Build { ... }
///         }
///     ```
///
/// Below is a sample module package body that would be generated:
///
/// ```ignore
/// pub mod build {
///   /// Collects the build-time info to construct a new Build instance
///   pub fn get() -> $crate::Build { ... }
///
///   /// The Continuous Integration platform detected during compilation.
///   pub const CI_PLATFORM: Option<&str> = None;
///   #[doc="The full version."]
///   pub const PKG_VERSION: &str = "0.1.0";
///   #[doc="The major version."]
///   pub const PKG_VERSION_MAJOR: &str = "0";
///   #[doc="The minor version."]
///   pub const PKG_VERSION_MINOR: &str = "1";
///   #[doc="The patch version."]
///   pub const PKG_VERSION_PATCH: &str = "0";
///   #[doc="The pre-release version."]
///   pub const PKG_VERSION_PRE: &str = "";
///   #[doc="A colon-separated list of authors."]
///   pub const PKG_AUTHORS: &str = "Alfio Zappala <oysterpack.inc@gmail.com>";
///   #[doc="The name of the package."]
///   pub const PKG_NAME: &str = "oysterpack_app_template";
///   #[doc="The description."]
///   pub const PKG_DESCRIPTION: &str = "OysterPack Application Template";
///   #[doc="The homepage."]
///   pub const PKG_HOMEPAGE: &str = "https://github.com/oysterpack/oysterpack";
///   #[doc="The target triple that was being compiled for."]
///   pub const TARGET: &str = "x86_64-unknown-linux-gnu";
///   #[doc="The host triple of the rust compiler."]
///   pub const HOST: &str = "x86_64-unknown-linux-gnu";
///   #[doc="`release` for release builds, `debug` for other builds."]
///   pub const PROFILE: &str = "debug";
///   #[doc="The compiler that cargo resolved to use."]
///   pub const RUSTC: &str = "rustc";
///   #[doc="The documentation generator that cargo resolved to use."]
///   pub const RUSTDOC: &str = "rustdoc";
///   #[doc="Value of OPT_LEVEL for the profile used during compilation."]
///   pub const OPT_LEVEL: &str = "0";
///   #[doc="The parallelism that was specified during compilation."]
///   pub const NUM_JOBS: u32 = 8;
///   #[doc="Value of DEBUG for the profile used during compilation."]
///   pub const DEBUG: bool = true;
///   /// The features that were enabled during compilation.
///   pub const FEATURES: [&str; 0] = [];
///   /// The features as a comma-separated string.
///   pub const FEATURES_STR: &str = "";
///   /// The output of `rustc -V`
///   pub const RUSTC_VERSION: &str = "rustc 1.29.1 (b801ae664 2018-09-20)";
///   /// The output of `rustdoc -V`
///   pub const RUSTDOC_VERSION: &str = "rustdoc 1.29.1 (b801ae664 2018-09-20)";
///   /// If the crate was compiled from within a git-repository, `GIT_VERSION` contains HEAD's tag. The short commit id is used if HEAD is not tagged.
///   pub const GIT_VERSION: Option<&str> = Some("oysterpack_built_v0.2.3-8-g640aba3");
///   /// The built-time in RFC822, UTC
///   pub const BUILT_TIME_UTC: &str = "Tue, 09 Oct 2018 21:49:26 GMT";
///   /// The target architecture, given by `cfg!(target_arch)`.
///   pub const CFG_TARGET_ARCH: &str = "x86_64";
///   /// The endianness, given by `cfg!(target_endian)`.
///   pub const CFG_ENDIAN: &str = "little";
///   /// The toolchain-environment, given by `cfg!(target_env)`.
///   pub const CFG_ENV: &str = "gnu";
///   /// The OS-family, given by `cfg!(target_family)`.
///   pub const CFG_FAMILY: &str = "unix";
///   /// The operating system, given by `cfg!(target_os)`.
///   pub const CFG_OS: &str = "linux";
///   /// The pointer width, given by `cfg!(target_pointer_width)`.
///   pub const CFG_POINTER_WIDTH: &str = "64";
///   /// graphviz .dot format for the dependency graph
///   pub const DEPENDENCIES_GRAPHVIZ_DOT: &str = r#"digraph {
///       0 [label="oysterpack_app_template=0.1.0"]
///       1 [label="oysterpack_app_metadata=0.1.0"]
///       2 [label="chrono=0.4.6"]
///       3 [label="semver=0.9.0"]
///       4 [label="serde_derive=1.0.79"]
///       5 [label="log=0.4.5"]
///       6 [label="serde=1.0.79"]
///       7 [label="fern=0.5.6"]
///       8 [label="cfg-if=0.1.5"]
///       9 [label="proc-macro2=0.4.19"]
///       10 [label="syn=0.15.6"]
///       11 [label="quote=0.6.8"]
///       12 [label="unicode-xid=0.1.0"]
///       13 [label="time=0.1.40"]
///       14 [label="libc=0.2.43"]
///       15 [label="semver-parser=0.7.0"]
///       16 [label="num-integer=0.1.39"]
///       17 [label="num-traits=0.2.6"]
///       0 -> 1
///       0 -> 2
///       0 -> 3
///       0 -> 4
///       0 -> 5
///       0 -> 6
///       0 -> 7
///       7 -> 5
///       5 -> 8
///       4 -> 9
///       4 -> 10
///       4 -> 11
///       11 -> 9
///       10 -> 9
///       10 -> 11
///       10 -> 12
///       9 -> 12
///       13 -> 14
///       3 -> 6
///       3 -> 15
///       2 -> 6
///       2 -> 16
///       2 -> 17
///       2 -> 13
///       16 -> 17
///       1 -> 6
///       1 -> 3
///       1 -> 2
///       1 -> 4
///   }
///   "#;
/// }
/// ```
#[macro_export]
macro_rules! op_build_mod {
    ($name:ident, $file:expr) => {
        /// provides build-time information
        pub mod $name {
            // The file has been placed there by the build script.
            include!($file);

            /// Collects the build-time info to construct a new Build instance
            pub fn get() -> $crate::Build {
                fn package_dependencies() -> Vec<$crate::metadata::PackageId> {
                    let mut dependencies: Vec<$crate::metadata::PackageId> =
                        DEPENDENCIES_GRAPHVIZ_DOT
                            .lines()
                            .filter(|line| !line.contains("->") && line.contains("["))
                            .skip(1)
                            .map(|line| {
                                let line = &line[line.find('"').unwrap() + 1..];
                                let line = &line[..line.find('"').unwrap()];
                                let tokens: Vec<&str> = line.split("=").collect();
                                $crate::metadata::PackageId::new(
                                    tokens.get(0).unwrap().to_string(),
                                    $crate::semver::Version::parse(tokens.get(1).unwrap()).unwrap(),
                                )
                            }).collect();
                    dependencies.sort();
                    dependencies
                }

                let mut builder = $crate::metadata::BuildBuilder::new();
                builder.timestamp(
                    $crate::chrono::DateTime::parse_from_rfc2822(BUILT_TIME_UTC)
                        .map(|ts| ts.with_timezone(&$crate::chrono::Utc))
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
                    $crate::semver::Version::parse(PKG_VERSION).unwrap(),
                    PKG_HOMEPAGE.to_string(),
                    package_dependencies(),
                );
                builder.build()
            }
        }
    };
    ($name:ident) => {
        op_build_mod!($name, concat!(env!("OUT_DIR"), "/built.rs"));
    };
    () => {
        op_build_mod!(build, concat!(env!("OUT_DIR"), "/built.rs"));
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

#[cfg(test)]
mod tests;
