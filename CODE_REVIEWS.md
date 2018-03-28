# Code Review Checklist

# Public API

## Comments
- [ ] The public API must have Doc comments
```rust
/// Generate library docs for the following public item.
```
- [ ] Public modules must have Doc comments
```rust
//! Generate library docs for the enclosing public item.
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
