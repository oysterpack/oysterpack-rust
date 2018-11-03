# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## \[Unreleased\]

## \[0.1.3\] 2018-11-03

### Added
- GenericUid

### Changed
- serialization has been changed from a number to a ULID string format
  - ULIDs are 128 bit, which would fail to parse as JSON

## \[0.1.2\]

### Added
- new CLI for generating ULIDs

## \[0.1.1\]

No code changes - just fixing documentation.

## \[0.1.0\]

Initial release