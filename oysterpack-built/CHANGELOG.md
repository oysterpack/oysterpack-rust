# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

## \[0.3.1\] 2018-10-13
There were no code changes made. This purpose of this release was to augment the crate metadata.

## \[0.3.0\] 2018-10-10

### Added
- [Collect crate dependencies](https://github.com/oysterpack/oysterpack/issues/1)

## \[0.3.0\] 2018-10-10

### Added
- DEPENDENCIES_GRAPHVIZ_DOT
  - crate dependencies in Graphviz DOT format

## \[0.2.3\] 2018-10-01

### Changed
- fixed README.md formatting issues

## \[0.2.2\] 2018-09-26

### Changed
- refactored code to remove need for public exports

## \[0.2.1\] 2018-09-23

### Added
- copyright to tests/version-numbers.rs

### Removed
- dev-dependencies on semver and chrono

## \[0.2.0\] 2018-09-23

### Changed
- `build::write_library_built_file()` and `build::write_app_built_file()`
    - returns () instead of io::Result<()>
    - panics if build-time information fails to be acquired

## \[0.1.1\] 2018-09-22

### Changed
- fixed README.md
- updated docs

## \[0.1.0\] 2018-09-22
Initial release