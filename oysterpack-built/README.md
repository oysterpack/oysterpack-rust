From a DevOps perspective, it is critical to know exactly what is deployed.
`oysterpack_built` is used as a build-time dependency to gather application build related metadata.
The information is gathered from the cargo build. It produces a Rust source file named **built.rs** in the project's build script output directory.
The location can be obtained via:

```ignore
let built_rs = concat!(env!("OUT_DIR"), "/built.rs");
```

## How to integrate within your project

1. Add the following to **Cargo.toml**:

   ```toml
   [package]
   build = "build.rs"

   [dependencies]
   oysterpack_app_metadata = "0.1"
   semver = "0.9"
   chrono = "0.4"

   [build-dependencies]
   oysterpack_built = "0.3"
   ```
   - `oysterpack_built` is added as a build dependency
   - `build.rs` is the name of the cargo build script to use
   - [oysterpack_app_metadata][oysterpack_app_metadata] is used in conjuction with this crate to load the application build metadata
     into the domain model defined by [oysterpack_app_metadata][oysterpack_app_metadata]

[oysterpack_app_metadata]: https://crates.io/crates/oysterpack_app_metadata

2. Include the following in **build.rs**:

   ```ignore
   extern crate oysterpack_built;

   fn main() {
      oysterpack_built::run();
   }
   ```

3. The build script will by default write a file named **built.rs** into
   Cargo's build output directory, which will contain the following constants:

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
DEPENDENCIES_GRAPHVIZ_DOT|&str|graphviz .dot format for the effective dependency graph

The application metadata can be loaded via [oysterpack_app_metadata op_build_mod!()](https://docs.rs/oysterpack_app_metadata/latest/oysterpack_app_metadata/macro.op_build_mod.html)):

```ignore
#[macro_use]
extern crate oysterpack_app_metadata;
extern crate chrono;
extern crate semver;

// loads the application metadata into `pub mod build {...}'
op_build_mod!()

use oysterpack_app_metadata::Build;

fn main () {
    let app_build = build::get();
    // integrate the application build metadata ...
}
```