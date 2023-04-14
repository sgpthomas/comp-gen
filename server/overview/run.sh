#!/usr/bin/env bash

RUST_LOG=debug,egg=info $compgen_bin compile 2d-conv --dios-bin $dios_bin --dios-example-bin $dios_example_bin --dios-params params.json --vector-width 4 --rules rules.json --config compile.json --output-dir results --costfn alternative