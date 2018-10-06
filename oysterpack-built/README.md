`oysterpack_built` is used as a build-time dependency to gather information about the cargo build
environment. It serializes the build-time information into Rust-code, which can then be compiled
into the final crate.

## What is the Motivation?
From a DevOps perspective, it is critical to know exactly what is deployed.

`oysterpack_built` provides the same functionality as [built](https://crates.io/crates/built).
Its main purpose is to standardize the integration for OysterPack apps.

## How to integrate within your project

1. Add the following to **Cargo.toml**:

           [package]
           build = "build.rs"

           [build-dependencies]
           oysterpack_built = "0.2"

    - `oysterpack_built` is added as a build dependency
    - `build.rs` is the name of the cargo build script to use
        - NOTE: By default Cargo looks up for "build.rs" file in a package root (even if you do
          not specify a value for build - see [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)).
2. Include the following in **build.rs**:

           extern crate oysterpack_built;

           fn main() {
              oysterpack_built::write_built_file();
           }

3. The build script will by default write a file named **built.rs** into Cargo's output directory.
   It can be picked up and compiled via the `op_build_mod!()` macro provided by [oysterpack_built_mod](https://crates.io/crates/oysterpack_built_mod).
   The `op_build_mod!()` will create a public module named *build*, which will contain the build-time
   information. See [oysterpack_built_mod](https://crates.io/crates/oysterpack_built_mod) for details.

Take a look at the [changelog](CHANGELOG.md) for a detailed list of all changes.

### Notes
- When running tests, enable the "build-time" feature because the unit tests are testing the build-time feature

            cargo test --features "build-time"