# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

## \[0.2.0\] 2018-10-20
- re-exported the `semver` and `chrono` crates because the `op_build_mod!()`
  macro depends on them. This makes the macro self-contained within this crate.

## \[0.1.2\] 2018-10-13
- Fixing build issue issue on crates.io

## \[0.1.0\] 2018-10-13
- initial release
- [build metadata domain model enhanced to support crate dependencies](https://github.com/oysterpack/oysterpack/issues/2)