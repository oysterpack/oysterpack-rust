# OysterPack Built

This builds upon [built](https://crates.io/crates/built) ... pun intended :)

## Usage

Add this to your `Cargo.toml`:
```toml
[package]
build = "build.rs"

[dependencies]
oysterpack_built = "0.1.0"

[build-dependencies]
oysterpack_built = "0.1.0"
```

Add or modify a build script. In build.rs:

### Libraries
```rust
extern crate built;
fn main() {
    oysterpack_built::write_lib_built_file().expect("Failed to acquire build-time information");
}
```

### Applications
```rust
extern crate built;
fn main() {
    oysterpack_built::write_app_built_file().expect("Failed to acquire build-time information");
}
```
- includes application dependency info
  - **NOTE:** dependency info can only be collected for standalone projects, i.e., this will not work for projects that are part of a Cargo workspace.
    - Cargo.lock is used to get the application's dependencies. Since Cargo.lock is shared by all projects in a workspace, this approach won't work for workspaces.


The build-script will by default write a file named built.rs into Cargo's output directory. It can be picked up in main.rs (or anywhere else) like this:
```rust
// Use of a mod or pub mod is not actually necessary.
pub mod build {
   // The file has been placed there by the build script.
   include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
```

