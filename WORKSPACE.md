1. Install [Rust](https://www.rust-lang.org/en-US/install.html)

    ```
    curl https://sh.rustup.rs -sSf | sh
    ```

2. Install [Tarpaulin](https://github.com/xd009642/tarpaulin) - code coverage tool

    ```
    sudo apt-get update
    sudo apt-get install libssl-dev pkg-config cmake zlib1g-dev

    cargo install cargo-tarpaulin

    # to convert cobertura.xml into an HTML report
    sudo pip install pycobertura
    ```