#!/usr/bin/env bash

# upgrade
apt-get update
apt-get -y upgrade

# install rust
curl https://sh.rustup.rs -sSf | sh -s -- --no-modify-path -y
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> $HOME/.profile
