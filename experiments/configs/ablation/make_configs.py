import json


def make_config(alpha, beta, timeout=180):
    return {
        "total_node_limit": 2000000000,
        "total_iter_limit": 4000,
        "timeout": timeout,
        "dry_run": False,
        "dump_rules": True,
        "debug": False,
        "reuse_egraphs": True,
        "cd_filter": None,
        "require_all_vars": False,
        "scheduler": "simple",
        "phase": {
            "phases": [
                {
                    "phases": [
                        {
                            "name": "pre-compile",
                            "cd": [None, None],
                            "ca": [beta, None],
                            "node_limit": 500000,
                            "timeout": 30,
                            "iter_limit": 2,
                            "fresh_egraph": True,
                            "disabled": False,
                        },
                        {
                            "name": "compile",
                            "cd": [alpha, None],
                            "ca": [None, None],
                            "timeout": 30,
                            "iter_limit": 2,
                            "disabled": False,
                        },
                        {
                            "name": "litvec",
                            "cd": [0.39, 0.41],
                            "ca": [0.20, 0.22],
                            "timeout": 30,
                            "iter_limit": 2,
                            "disabled": False,
                        },
                    ],
                    "loops": 20,
                },
                {
                    "name": "opt",
                    "cd": [None, None],
                    "ca": [None, beta],
                    "fresh_egraph": True,
                    "iter_limit": 10,
                    "scheduler": "simple",
                    "disabled": False,
                },
            ]
        },
    }


# if __name__ == "__main__":
for beta in [0, 1, 2, 3, 4, 5, 6, 10, 15, 20, 30, 2020, 2021, 2022]:
    for alpha in [
        -5000,
        -4039,
        -15,
        -10,
        -5,
        -1,
        -0.5,
        0.5,
        1,
        5,
        10,
        15,
        4039,
        5000,
    ]:
        with open(f"config_a{alpha}_b{beta}.json", "w+") as f:
            json.dump(make_config(float(alpha), float(beta)), f, indent=2)
