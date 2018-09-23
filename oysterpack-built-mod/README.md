This module provides the `op_build_mod!()` macro.
`op_build_mod!()` will generate a public module that contains build-time
info that was generated via [oysterpack_built](https://crates.io/crates/oysterpack_built).

Its main purposes are:
1. standardize the module package naming
2. centralize the code and remove copy and paste boilerplate code
    - this will make it easier to upgrade


