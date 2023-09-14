#!/usr/bin/env bash

RUST_LOG=debug,egg=info,z3=off $compgen_bin synth ruleset.json --config synth.json