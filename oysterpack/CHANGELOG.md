# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

## \[0.2.3\] 2018-10-28

### Changed
- upgraded to rust 2018 edition

## \[0.2.2\] 2018-10-28

### Added
- integrated oysterpack_log

### Changed
- explicity export macros

## \[0.2.1\] 2018-10-21

### Added
- [oysterpack_macros](https://crates.io/crates/oysterpack_macros)
  - macros are re-exported

## Removed
- [log](https://crates.io/crates/log) as a dependency
  - moved to dev-dependencies for now. Once the Rust 2018 edition becomes available,
    then the log crate will be curated to re-export the log macros.

## \[0.2.0\] 2018-10-20

### Added
- integrated the following crates :
  - [oysterpack_app_metadata](https://crates.io/crates/oysterpack_app_metadata)
  - [oysterpack_app_metadata_macros](https://crates.io/crates/oysterpack_app_metadata_macros)
  - [oysterpack_uid](https://crates.io/crates/oysterpack_uid)
  - [serde](https://crates.io/crates/serde)
  - [serde_derive](https://crates.io/crates/serde_derive)
  - [semver](https://crates.io/crates/semver)
  - [chrono](https://crates.io/crates/chrono)

## \[0.1.1\] 2018-03-17

