# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

## \[0.1.4\] 2018-11-03

### Changed
- upgraded to rust 2018 edition

## \[0.1.3\] 2018-11-03

### Changed
- the module path is logged instead of the source file path

## \[0.1.2\] 2018-10-25

### Changed
- refactored op_tests_mod macro to ensure log initialization is threadsafe

## \[0.1.1\] 2018-10-25

### Added
- tests::run_test is bound to the crate's root path:
```rust
#[cfg(test)]
pub use tests::run_test;
```

### Fixed
- initializing the logger may fail because of a potential race condition when tests are run in parallel
  - the fix is to ignore the error instead of panicking  

## \[0.1.0\] 2018-10-25
- initial release