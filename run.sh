#!/usr/bin/env bash

mkdir -p output

container_manager="$1"
shift 1

$container_manager run -v "$(pwd)/output:/home/aec/comp-gen/output" "working-container" -- $@
