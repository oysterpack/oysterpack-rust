## Installing a C compiler on debian / ubuntu
```
sudo apt-get update
sudo atp-get install build-essential
```
- the Rust compiler needs a linker, which is provided by the C compiler
- some common Rust packages depend on C code and will need a C compiler too

## Running test code coverage

```
# runs tests with code coverage and produces cobertura.xml output file
cargo tarpaulin -v -o Xml

# convert cobertura.xml into an HTML report
pycobertura show --format html --output coverage.html cobertura.xml
```

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