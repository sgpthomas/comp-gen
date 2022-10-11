#!/usr/bin/env bash

set -x

ARGS="$@"

tmux new -d -s "exp"
tmux send-keys -t "exp" "export RUST_LOG=info,egg=off" ENTER
tmux send-keys -t "exp" "./dios-lang $ARGS >stdout.log 2>stderr.log" ENTER
