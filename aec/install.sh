#!/usr/bin/env bash

# update path
export PATH="/root/.cargo/bin:/root/racket/bin:$PATH"

# clone repos that we need
git clone https://github.com/sgpthomas/comp-gen.git /root/comp-gen
cd /root/comp-gen
cargo build --manifest-path dios-lang/Cargo.toml --release
cd /

git clone https://github.com/sgpthomas/diospyros.git /root/diospyros
cd /root/diospyros
raco pkg install --batch --deps search-auto rosette threading
make
cd /

# install some needed python dependencies for the experiment server
pip3 install psutil click pandas
