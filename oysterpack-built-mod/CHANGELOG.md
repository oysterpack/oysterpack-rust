# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## \[Unreleased\]

## \[0.2.0\] 2018-09-23
- type safe domain model is now provided for build-time info, via the
  `Build` struct
  - this requires the following crate dependencies:
    - [semver](https://crates.io/crates/semver)
    - [chrono](https://crates.io/crates/chrono)
    - [serde](https://crates.io/crates/serde)
    - [serde_derive](https://crates.io/crates/serde_derive)

## \[0.1.0\] 2018-09-23
Initial release