Provides the ability to gather information about the crate's cargo build.

All OysterPack modules must provide build time info. This module standardizes the approach, which
leverages [built](https://crates.io/crates/built).

Take a look at the [changelog][changelog] for a detailed list of all changes.


## How to integrate within your project

1. Add the following to **Cargo.toml**:
   ```toml
   [package]
   build = "build.rs"

   [build-dependencies]
   oysterpack_built = "0.2"
   ```

2. Include the following in **build.rs**:

   **For Library Modules**
   ```no_run
   extern crate oysterpack_built;

   fn main() {
       oysterpack_built::write_library_built_file();
   }
   ```

   **For Application (Binary) Modules**
   ```no_run
   extern crate oysterpack_built;

   fn main() {
       oysterpack_built::write_app_built_file();
   }
   ```
   - includes application dependency info
     - **NOTE:** dependency info can only be collected for standalone projects, i.e., this will not work for projects that are part of a Cargo workspace.
       - Cargo.lock is used to get the application's dependencies. Since Cargo.lock is shared by all projects in a workspace, this approach won't work for workspaces.

3. The build script will by default write a file named **built.rs** into Cargo's output directory. It can be picked up like this:
   ```no_run
   // Use of a mod or pub mod is not actually necessary.
   pub mod build {
      // The file has been placed there by the build script.
      include!(concat!(env!("OUT_DIR"), "/built.rs"));
   }
   ```
   - `OUT_DIR` [environment variable is set by Cargo for build scripts](https://doc.rust-lang.org/cargo/reference/environment-variables.html)