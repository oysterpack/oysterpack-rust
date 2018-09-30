This module provides the `op_build_mod!()` macro that will generate a
public module named `build` that contains build-time metadata.
This module is meant to be used with [oysterpack_built](https://crates.io/crates/oysterpack_built),
which extracts The build-time info during compilation.

The generated `build` module will consist of:

- constants for each piece of build metadata

Constant | Type | Description
-------- | ---- | -----------
BUILT_TIME_UTC|&str|The built-time in RFC822, UTC
CFG_ENDIAN|&str|The endianness, given by cfg!(target_endian).
CFG_ENV|&str|The toolchain-environment, given by cfg!(target_env).
CFG_FAMILY|&str|The OS-family, given by cfg!(target_family).
CFG_OS|&str|The operating system, given by cfg!(target_os).
CFG_POINTER_WIDTH|u8|The pointer width, given by cfg!(target_pointer_width).
CFG_TARGET_ARCH|&str|The target architecture, given by cfg!(target_arch).
CI_PLATFORM|Option<&str>|The Continuous Integration platform detected during compilation.
DEBUG|bool|Value of DEBUG for the profile used during compilation.
FEATURES|\[&str; N\]|The features that were enabled during compilation.
FEATURES_STR|&str|The features as a comma-separated string.
GIT_VERSION|Option<&str>|If the crate was compiled from within a git-repository, GIT_VERSION contains HEAD's tag. The short commit id is used if HEAD is not tagged.
HOST|&str|The host triple of the rust compiler.
NUM_JOBS|u32|The parallelism that was specified during compilation.
OPT_LEVEL|&str|Value of OPT_LEVEL for the profile used during compilation.
PKG_AUTHORS|&str|A colon-separated list of authors.
PKG_DESCRIPTION|&str|The description.
PKG_HOMEPAGE|&str|The homepage.
PKG_NAME|&str|The name of the package.
PKG_VERSION|&str|The full version.
PKG_VERSION_MAJOR|&str|The major version.
PKG_VERSION_MINOR|&str|The minor version.
PKG_VERSION_PATCH|&str|The patch version.
PKG_VERSION_PRE|&str|The pre-release version.
PROFILE|&str|release for release builds, debug for other builds.
RUSTC|&str|The compiler that cargo resolved to use.
RUSTC_VERSION|&str|The output of rustc -V
RUSTDOC|&str|The documentation generator that cargo resolved to use.
RUSTDOC_VERSION|&str|The output of rustdoc -V
- `fn get() -> Build`
    - [Build](struct.Build.html) provides a consolidated view of the build-time metadata.
      This makes it easier to work with the build-time metadata in a
      typesafe manner.

**NOTE:** The `op_build_mod!()` depends on the following dependencies in order to compile:

- [semver](https://crates.io/crates/semver)
- [chrono](https://crates.io/crates/chrono)




