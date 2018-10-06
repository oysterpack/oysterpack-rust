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
                let dependencies: Vec<$crate::metadata::PackageId> = ::serde_json::from_str(DEPENDENCIES_JSON).unwrap();
                builder.package(
                    PKG_NAME.to_string(),
                    PKG_AUTHORS
                        .split(':')
                        .map(|author| author.to_string())
                        .collect(),
                    PKG_DESCRIPTION.to_string(),
                    ::semver::Version::parse(PKG_VERSION).unwrap(),
                    PKG_HOMEPAGE.to_string(),
                    dependencies
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
