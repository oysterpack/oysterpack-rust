This module provides the `op_build_mod!()` macro that will generate a
public module named `build` that contains build-time metadata.
This module is meant to be used with [oysterpack_built](https://crates.io/crates/oysterpack_built),
which extracts The build-time info during compilation.

The generated `build` module will consist of:

- constants for each piece of build metadata
- `fn get() -> Build`
    - [Build](struct.Build.html) provides a consolidated view of the build-time metadata.
      This makes it easier to work with the build-time metadata in a
      typesafe manner.

**NOTE:** The `op_build_mod!()` depends on the following dependencies in order to compile:

- [semver](https://crates.io/crates/semver)
- [chrono](https://crates.io/crates/chrono)





