#!/usr/bin/env bash

export PATH="/root/.cargo/bin:/root/racket/bin:$PATH"

# if we have an xtensa directory, install the core
if [ -e /root/xtensa ]; then
    cd /root/xtensa/___downloads/RI-2021.8/builds/

    # delete existing installation if it exists
    rm -rf RI-2021.8-linux

    # extract the core that we want, and enter that directory
    tar xvf XRC_FUSIONG3_TYP_SPVFPU_linux.tgz

    # run the install script
    cd RI-2021.8-linux/XRC_FUSIONG3_TYP_SPVFPU/
    ./install --xtensa-tools /root/xtensa/RI-2021.8-linux/XtensaTools

    # point the license file at the license server
    export LM_LICENSE_FILE="27010@10.0.2.2"
fi

cd /root/comp-gen
echo "Pulling changes..."
git pull -r
cargo build --release --manifest-path=dios-lang/Cargo.toml

cd /root/comp-gen/server
python3 server.py
