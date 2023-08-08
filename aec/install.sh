#!/usr/bin/env bash

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install-rustup.sh
chmod +x /tmp/install-rustup.sh
sh /tmp/install-rustup.sh -y

cargo="/root/.cargo/bin/cargo"

git clone https://github.com/sgpthomas/comp-gen.git /root/comp-gen
$cargo build --manifest-path /root/comp-gen/dios-lang/Cargo.toml --release



