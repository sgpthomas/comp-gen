#!/usr/bin/env python3

import json
import sys
import glob
import os

def main():
    base = "base.json"
    with open(base, "r") as f:
        base_config = json.load(f)

    use_smt = [True, False]
    smt_fallback = [True, False]
    constants = [
        [],
        [
            {"kind": "int", "value": 0},
	    {"kind": "int", "value": 1}
        ]
    ]
    seed_rules = [
        [],
        [
            {
	        "lhs": "(Vec (+ ?a) (+ ?b))",
	        "rhs": "(VecAdd (Vec ?a) (Vec ?b))"
            },
	    {
	        "lhs": "(Vec (* ?a) (* ?b))",
	        "rhs": "(VecMul (Vec ?a) (Vec ?b))"
            },
            {
	        "lhs": "(Vec (- ?a) (- ?b))",
	        "rhs": "(VecMinus (Vec ?a) (Vec ?b))"
            }
        ]
    ]

    for smt_opt in use_smt:
        for smt_fallback_opt in smt_fallback:
            for const in constants:
                for sr in seed_rules:
                    config = base_config.copy()
                    config["constants"] = const
                    config["seed_rules"] = sr
                    config["use_smt"] = smt_opt
                    config["smt_fallback"] = smt_fallback_opt
                    name = "config_c{}_sr{}_us{}_sf{}.json".format(
                        len(const),
                        len(sr),
                        smt_opt,
                        smt_fallback_opt,
                    )
                    with open(name, "w") as f:
                        json.dump(config, f, indent=2)

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "make":
        main()
    elif len(sys.argv) > 1 and sys.argv[1] == "clean":
        for path in glob.glob("config_c*_sr*_us*_sf*.json"):
            os.remove(path)
    else:
        print(f"Usage: {sys.argv[0]} [make|clean]")
