# [API Guidelines](https://rust-lang-nursery.github.io/api-guidelines/checklist.html)

## Interoperability
- [ ] [Types eagerly implement common traits (C-COMMON-TRAITS)](https://rust-lang-nursery.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)
- [ ] [Conversions use the standard traits From, AsRef, AsMut (C-CONV-TRAITS)](https://rust-lang-nursery.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv-traits)
- [ ] [Data structures implement Serde's Serialize, Deserialize (C-SERDE)](https://rust-lang-nursery.github.io/api-guidelines/interoperability.html#data-structures-implement-serdes-serialize-deserialize-c-serde)
- [ ] [Types are Send and Sync where possible (C-SEND-SYNC)](https://rust-lang-nursery.github.io/api-guidelines/interoperability.html#types-are-send-and-sync-where-possible-c-send-sync)
- [ ] [Error types are meaningful and well-behaved (C-GOOD-ERR)](https://rust-lang-nursery.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)

## Documentation
- [ ] [Crate level docs are thorough and include examples (C-CRATE-DOC)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#crate-level-docs-are-thorough-and-include-examples-c-crate-doc)
- [ ] [All items have a rustdoc example (C-EXAMPLE)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#all-items-have-a-rustdoc-example-c-example)
- [ ] [Examples use ?, not try!, not unwrap (C-QUESTION-MARK)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#examples-use--not-try-not-unwrap-c-question-mark)
- [ ] [Function docs include error, panic, and safety considerations (C-FAILURE)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)
- [ ] [Crate sets html_root_url attribute (C-HTML-ROOT)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#crate-sets-html_root_url-attribute-c-html-root)
- [ ] [Release notes document all significant changes (C-RELNOTES)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#release-notes-document-all-significant-changes-c-relnotes)
- [ ] [Rustdoc does not show unhelpful implementation details (C-HIDDEN)](https://rust-lang-nursery.github.io/api-guidelines/documentation.html#rustdoc-does-not-show-unhelpful-implementation-details-c-hidden)

## Predictability
- [ ] [Smart pointers do not add inherent methods (C-SMART-PTR)](https://rust-lang-nursery.github.io/api-guidelines/predictability.html#smart-pointers-do-not-add-inherent-methods-c-smart-ptr)

## Flexability
- [ ] [Functions minimize assumptions about parameters by using generics (C-GENERIC)](https://rust-lang-nursery.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generics-c-generic)
- [ ] [Traits are object-safe if they may be useful as a trait object (C-OBJECT)](https://rust-lang-nursery.github.io/api-guidelines/flexibility.html#traits-are-object-safe-if-they-may-be-useful-as-a-trait-object-c-object)

## Type Safety
- [ ] [Types for a set of flags are bitflags, not enums (C-BITFLAG)](https://rust-lang-nursery.github.io/api-guidelines/type-safety.html#types-for-a-set-of-flags-are-bitflags-not-enums-c-bitflag)

## Dependability
- [ ] [Functions validate their arguments (C-VALIDATE)](https://rust-lang-nursery.github.io/api-guidelines/dependability.html#functions-validate-their-arguments-c-validate)
- [ ] [Destructors never fail (C-DTOR-FAIL)](https://rust-lang-nursery.github.io/api-guidelines/dependability.html#destructors-never-fail-c-dtor-fail)

## Debugability
- [ ] [All public types implement Debug (C-DEBUG)](https://rust-lang-nursery.github.io/api-guidelines/debuggability.html#all-public-types-implement-debug-c-debug)

## Future Proofing
- [ ] [Sealed traits protect against downstream implementations (C-SEALED)](https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
- [ ] [Structs have private fields (C-STRUCT-PRIVATE)](https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private)


## Testing
- [ ] Unit test code coverage must be 100%
- [ ] Public API must have integration tests
- [ ] Performance critical APIs must have benchmark tests
