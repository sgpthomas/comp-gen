#!/usr/bin/env python3

import json
import sys
import glob
import os

def main():
    base = "base.json"
    with open(base, "r") as f:
        base_config = json.load(f)

    abs_timeout_options = map(lambda s: s * 60, [1, 10, 100, 1000, 10000])

    for option in abs_timeout_options:
        config = base_config.copy()
        config["abs_timeout"] = option
        with open("abs_timeout_{}.json".format(option), "w") as f:
            json.dump(config, f, indent=2)

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "make":
        main()
    elif len(sys.argv) > 1 and sys.argv[1] == "clean":
        for path in glob.glob("abs_timeout_*.json"):
            os.remove(path)
    else:
        print(f"Usage: {sys.argv[0]} [make|clean]")
