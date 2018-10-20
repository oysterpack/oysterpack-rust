/// The Continuous Integration platform detected during compilation.
pub const CI_PLATFORM: Option<&str> = None;
#[doc = "The full version."]
pub const PKG_VERSION: &str = "0.1.0";
#[doc = "The major version."]
pub const PKG_VERSION_MAJOR: &str = "0";
#[doc = "The minor version."]
pub const PKG_VERSION_MINOR: &str = "1";
#[doc = "The patch version."]
pub const PKG_VERSION_PATCH: &str = "0";
#[doc = "The pre-release version."]
pub const PKG_VERSION_PRE: &str = "";
#[doc = "A colon-separated list of authors."]
pub const PKG_AUTHORS: &str = "Alfio Zappala <oysterpack.inc@gmail.com>";
#[doc = "The name of the package."]
pub const PKG_NAME: &str = "oysterpack_app_template";
#[doc = "The description."]
pub const PKG_DESCRIPTION: &str = "OysterPack Application Template";
#[doc = "The homepage."]
pub const PKG_HOMEPAGE: &str = "https://github.com/oysterpack/oysterpack";
#[doc = "The target triple that was being compiled for."]
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
#[doc = "The host triple of the rust compiler."]
pub const HOST: &str = "x86_64-unknown-linux-gnu";
#[doc = "`release` for release builds, `debug` for other builds."]
pub const PROFILE: &str = "debug";
#[doc = "The compiler that cargo resolved to use."]
pub const RUSTC: &str = "rustc";
#[doc = "The documentation generator that cargo resolved to use."]
pub const RUSTDOC: &str = "rustdoc";
#[doc = "Value of OPT_LEVEL for the profile used during compilation."]
pub const OPT_LEVEL: &str = "0";
#[doc = "The parallelism that was specified during compilation."]
pub const NUM_JOBS: u32 = 8;
#[doc = "Value of DEBUG for the profile used during compilation."]
pub const DEBUG: bool = true;
/// The features that were enabled during compilation.
pub const FEATURES: [&str; 0] = [];
/// The features as a comma-separated string.
pub const FEATURES_STR: &str = "";
/// The output of `rustc -V`
pub const RUSTC_VERSION: &str = "rustc 1.29.1 (b801ae664 2018-09-20)";
/// The output of `rustdoc -V`
pub const RUSTDOC_VERSION: &str = "rustdoc 1.29.1 (b801ae664 2018-09-20)";
/// If the crate was compiled from within a git-repository, `GIT_VERSION` contains HEAD's tag. The short commit id is used if HEAD is not tagged.
pub const GIT_VERSION: Option<&str> = Some("oysterpack_built_v0.2.3-16-gce7b5a1");
/// The built-time in RFC822, UTC
pub const BUILT_TIME_UTC: &str = "Thu, 11 Oct 2018 20:51:55 GMT";
/// The target architecture, given by `cfg!(target_arch)`.
pub const CFG_TARGET_ARCH: &str = "x86_64";
/// The endianness, given by `cfg!(target_endian)`.
pub const CFG_ENDIAN: &str = "little";
/// The toolchain-environment, given by `cfg!(target_env)`.
pub const CFG_ENV: &str = "gnu";
/// The OS-family, given by `cfg!(target_family)`.
pub const CFG_FAMILY: &str = "unix";
/// The operating system, given by `cfg!(target_os)`.
pub const CFG_OS: &str = "linux";
/// The pointer width, given by `cfg!(target_pointer_width)`.
pub const CFG_POINTER_WIDTH: &str = "64";

/// graphviz .dot format for the dependency graph
pub const DEPENDENCIES_GRAPHVIZ_DOT: &str = r#"digraph {
    0 [label="oysterpack_app_template=0.1.0"]
    1 [label="fern=0.5.6"]
    2 [label="log=0.4.5"]
    3 [label="semver=0.9.0"]
    4 [label="oysterpack_app_metadata=0.1.0"]
    5 [label="chrono=0.4.6"]
    6 [label="num-traits=0.2.6"]
    7 [label="serde=1.0.79"]
    8 [label="time=0.1.40"]
    9 [label="num-integer=0.1.39"]
    10 [label="libc=0.2.43"]
    11 [label="serde_derive=1.0.79"]
    12 [label="quote=0.6.8"]
    13 [label="proc-macro2=0.4.19"]
    14 [label="syn=0.15.6"]
    15 [label="unicode-xid=0.1.0"]
    16 [label="semver-parser=0.7.0"]
    17 [label="cfg-if=0.1.5"]
    0 -> 1
    0 -> 2
    0 -> 3
    0 -> 4
    0 -> 5
    5 -> 6
    5 -> 7
    5 -> 8
    5 -> 9
    9 -> 6
    8 -> 10
    4 -> 11
    4 -> 3
    4 -> 5
    4 -> 7
    11 -> 12
    11 -> 13
    11 -> 14
    14 -> 12
    14 -> 13
    14 -> 15
    13 -> 15
    12 -> 13
    3 -> 7
    3 -> 16
    2 -> 17
    1 -> 2
}
"#;