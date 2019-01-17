## Installing a C compiler on debian / ubuntu
```
sudo apt-get update
sudo atp-get install build-essential
```
- the Rust compiler needs a linker, which is provided by the C compiler
- some common Rust packages depend on C code and will need a C compiler too

## installing musl-tools
```
sudo apt-get install musl-tools
```
- [MUSL support for fully static binaries](https://rust-lang-nursery.github.io/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)

## Running test code coverage
https://crates.io/crates/cargo-tarpaulin

## Profiling apps using [Valgrand](http://valgrind.org/)
There are Valgrind tools that can automatically detect many memory management and threading bugs, and profile your programs in detail.

## Cargo plugins
1. [cargo-tree](https://github.com/sfackler/cargo-tree)
    ```
    cargo tree
    ```
2. [rustfmt](https://github.com/rust-lang-nursery/rustfmt)
    ```
    cargo fmt
    ```
3. [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) - Find out what takes most of the space in your executable

4. [tarpaulin](https://github.com/xd009642/tarpaulin) - code coverage reporting tool for the Cargo build system

5. [cargo-watch](https://crates.io/crates/cargo-watch) - Cargo Watch watches over your project's source for changes, and runs Cargo commands when they occur.


## Tools
1. [glogg](http://glogg.bonnefon.org/)
glogg is a multi-platform GUI application to browse and search through long or complex log files.
It is designed with programmers and system administrators in mind. glogg can be seen as a graphical, interactive combination of grep and less.

2. [Oracle VirtualBox](https://www.virtualbox.org)

    ```
    deb https://download.virtualbox.org/virtualbox/debian <mydist> contrib

    # Ubuntu 16.04
    deb https://download.virtualbox.org/virtualbox/debian xenial contrib
    ```

3. [vagrant](https://www.vagrantup.com/)

## Module Release Process and Checklist
- [ ] ensure `#![deny(missing_docs, missing_debug_implementations, warnings)]` is enabled for compile
- [ ] format the code via `cargo fmt`
- [ ] review documentation: `cargo doc --open`
- [ ] review Cargo.toml
- [ ] review CHANGELOG.md
- [ ] review README.md
- [ ] tag the module using the following naming convention : `{module_name}_v{module_version}`
    - `git tag -a oysterpack_built_v0.2.0 -m "oysterpack_built 0.2.0 release"`
    - `git push --tags` 