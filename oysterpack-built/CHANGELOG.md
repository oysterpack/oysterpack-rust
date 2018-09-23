# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

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