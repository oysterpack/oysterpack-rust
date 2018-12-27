# Change Log

All user visible changes to this project will be documented in this file. The format is based on [Keep a Changelog](http://keepachangelog.com/).

This project adheres to [Semantic Versioning](http://semver.org/), as described for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## \[Unreleased\]

## \[0.2.3\] 2018-12-27
- re-organized project structure

### Added
- benchmark tests
- re-export ulid and domain attributes from oysterpack_uid_macros

### Changed
- ULID serde serialization to u128 instead of string for performance boost

### Removed
- TypedULID 

## \[0.2.2\] 2018-12-07
- upgraded to rust 2018 edition

## \[0.2.1\] 2018-11-07

### Added
- impl AsRef<str> for Domain

## \[0.2.0\] 2018-11-04

The API has been re-designed and is not backward compatible with v0.1

### Added
- ULID struct
- DomainULID struct
- HasDomain trait

### Changed
- serialization has been changed from a number to a ULID string format
  - ULIDs are 128 bit, which would fail to parse as JSON
- renamed uid::ulid() to ulid_str() to signify that it returns the ULID as a raw string
- renamed uid module to ulid
- renamed Uid to TypedULID

### Removed
- uid ULID low level functions are no longer re-exported  

## \[0.1.2\]

### Added
- new CLI for generating ULIDs

## \[0.1.1\]

No code changes - just fixing documentation.

## \[0.1.0\]

Initial release