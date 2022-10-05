#!/usr/bin/env bash

set -x
set -e

IP=$1

cargo build --release --manifest-path ../../dios-lang/Cargo.toml
rsync -rvP ../../dios-lang/target/release/dios-lang "ubuntu@$IP:"
rsync -rvP configs "ubuntu@$IP:~/"

ssh -t ubuntu@$IP <<EOF
uname -a
export RUST_LOG=info
./dios-lang synth out.json --config configs/all_ops_no_constants.json >stdout.log 2>stderr.log
EOF

RESULT_DIR=data/$(date +"%d-%m-%H%M")/
mkdir -p $RESULT_DIR
rsync -rvP "ubuntu@$IP:~/out.json" "ubuntu@$IP:~/stdout.log" "ubuntu@$IP:~/stderr.log" $RESULT_DIR
