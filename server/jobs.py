import json
from datetime import datetime
from server import unique_name
from pathlib import Path
import os
import shutil
import itertools


def mat_mul(jobs_dir, a_rows, a_cols, b_rows, b_cols, ruleset, compile, alt_cost):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}"
    job_dir = unique_name(
        jobs_dir / f"{date_str}-{name}",
        0
    )
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "memory_limit": 220,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "alt_cost": alt_cost
        }
    }
    params = {
        "A-rows": a_rows,
        "A-cols": a_cols,
        "B-rows": b_rows,
        "B-cols": b_cols,
        "reg-size": 4
    }
    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")
    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin", "compile", "mat-mul",
        "--dios-bin", "$dios_bin",
        "--dios-example-bin", "$dios_example_bin",
        "--dios-params", "params.json",
        "--vector-width", "4",
        "--rules", "rules.json",
        "--config", "compile.json",
        "--output-dir", "results"
    ]

    if alt_cost:
        command += ["--alt-cost"]

    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)
    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    os.chmod(str(job_dir / "run.sh"), 0o777)


# def make_2d_conv(base, jobs_dir, irows, icols, frows, fcols):
#     date_str = datetime.now().strftime("%b%d-%H%M")
#     job_dir = unique_name(
#         jobs_dir / f"{date_str}-2d-conv_{irows}x{icols}_{frows}x{fcols}",
#         0
#     )
#     config = {
#         "benchmark": "2d-conv",
#         "params": {
#             "input-rows": irows,
#             "input-cols": icols,
#             "filter-rows": frows,
#             "filter-cols": fcols,
#             "reg-size": 4
#         },
#     }
    # make_job(job_dir, {**base, **config})


# def q_prod(base, jobs_dir):
#     date_str = datetime.now().strftime("%b%d-%H%M")
#     job_dir = unique_name(
#         jobs_dir / f"{date_str}-q-prod",
#         0
#     )
#     config = {
#         "benchmark": "q-prod",
#         "params": {
#             "reg-size": 4
#         },
#     }
    # make_job(job_dir, {**base, **config})


# def qr_decomp(base, jobs_dir, N):
#     date_str = datetime.now().strftime("%b%d-%H%M")
#     job_dir = unique_name(
#         jobs_dir / f"{date_str}-qr-decomp_{N}",
#         0
#     )
#     config = {
#         "benchmark": "qr-decomp",
#         "params": {
#             "N": N,
#             "reg-size": 4
#         },
#     }
    # make_job(job_dir, {**base, **config})


rulesets = {
    "expanding": "../experiments/expanding_rules.json",
    "ruler": "../experiments/rule-generation/data/11-14-test/ruleset_timeout432000.json",
    # "t2": "~/Research/diospyros/t2.json"
}

# resolve the ruleset paths
for key, val in rulesets.items():
    rulesets[key] = Path(val).expanduser().resolve()

configs = {
    "wack": "../experiments/wack.json",
    "phased": "../experiments/compile.json",
    # "loop": "../experiments/compile_alt_cost.json"
}

# resolve the ruleset paths
for key, val in configs.items():
    configs[key] = Path(val).expanduser().resolve()

alt_cost = [
    True,
    False
]

mat_mul_sizes = [
    [2, 2, 2, 2],
    [2, 3, 3, 3],
    [3, 3, 3, 3],
    [4, 4, 4, 4],
    [8, 8, 8, 8],
    [10, 10, 10, 10],
    [16, 16, 16, 16]
]

exps = itertools.product(mat_mul_sizes, rulesets, configs, alt_cost)
for (size, r, c, b) in exps:
    mat_mul(Path("jobs"), *size, rulesets[r], configs[c], b)

# for size in mat_mul_sizes:
#     mat_mul(Path("jobs"), *size)

# for rule in rulesets:
#     for config in configs:
#         for cf in cost_function:
#             jobs = Path("jobs")
#             base = base_job(rule, config, cf)

#             # make_2d_conv(base, jobs, 3, 3, 2, 2)
#             # make_2d_conv(base, jobs, 3, 3, 3, 3)
#             # make_2d_conv(base, jobs, 3, 5, 3, 3)
#             # make_2d_conv(base, jobs, 4, 4, 3, 3)
#             # make_2d_conv(base, jobs, 8, 8, 3, 3)

#             # make_2d_conv(base, jobs, 10, 10, 2, 2)
#             # make_2d_conv(base, jobs, 10, 10, 3, 3)
#             # make_2d_conv(base, jobs, 10, 10, 4, 4)

#             # make_2d_conv(base, jobs, 16, 16, 2, 2)
#             # make_2d_conv(base, jobs, 16, 16, 3, 3)
#             # make_2d_conv(base, jobs, 16, 16, 4, 4)

#             q_prod(base, jobs)

#             # qr_decomp(base, jobs, 3)
#             # qr_decomp(base, jobs, 4)
