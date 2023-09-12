#!/usr/bin/env bash

export PATH="/root/.cargo/bin:/root/racket/bin:$PATH"

cd /root/comp-gen
echo "Pulling changes..."
git pull

cd /root/comp-gen/server
python3 server.py
