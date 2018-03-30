# Code Review Checklist

[API Guidelines](https://rust-lang-nursery.github.io/api-guidelines/checklist.html)

# Public API

## Documentation
- [ ] The public API must have Doc comments
```rust
/// Generate library docs for the following public item.
```
- [ ] Public modules must have Doc comments
```rust
//! Generate library docs for the enclosing public item.
```
- [ ] All changes should be documented in **CHANGELOG.md**
    - see [Diesel change log](https://github.com/diesel-rs/diesel/blob/master/CHANGELOG.md) as an example
```
### Added

### Changed

### Removed

### Deprecated

### Fixed
```

## Stucts and Enums
- [ ] The Debug trait must be implemented
     - in most cases it can be derived :
```rust
#[derive(Debug)]
pub struct MinMax(i64, i64);
```
- [ ] Should the Display trait be implemented ?

## Testing
- [ ] Unit test code coverage must be 100%
- [ ] Public API must have integration tests
- [ ] Performance critical APIs must have benchmark tests
