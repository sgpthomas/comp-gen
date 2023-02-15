import json
from datetime import datetime
from server import unique_name
from pathlib import Path


def make_job(job_dir, config):
    print(f"Created {config['benchmark']} job at {job_dir}")
    job_dir.mkdir(exist_ok=False)
    json.dump(
        config,
        (job_dir / "config.json").open("w"),
        indent=2
    )


def base_job(ruleset_name, compile_config, cost_function):
    return {
        "ruleset": ruleset_name,
        "compile_config": compile_config,
        "cost_function": cost_function,
        "memory_limit": 220
    }


def mat_mul(base, jobs_dir, a_rows, a_cols, b_rows, b_cols):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}",
        0
    )
    config = {
        "benchmark": "mat-mul",
        "params": {
            "A-rows": a_rows,
            "A-cols": a_cols,
            "B-rows": b_rows,
            "B-cols": b_cols,
            "reg-size": 4
        },
    }
    make_job(job_dir, {**base, **config})


def make_2d_conv(base, jobs_dir, irows, icols, frows, fcols):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-2d-conv_{irows}x{icols}_{frows}x{fcols}",
        0
    )
    config = {
        "benchmark": "2d-conv",
        "params": {
            "input-rows": irows,
            "input-cols": icols,
            "filter-rows": frows,
            "filter-cols": fcols,
            "reg-size": 4
        },
    }
    make_job(job_dir, {**base, **config})


def q_prod(base, jobs_dir):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-q-prod",
        0
    )
    config = {
        "benchmark": "q-prod",
        "params": {
            "reg-size": 4
        },
    }
    make_job(job_dir, {**base, **config})


def qr_decomp(base, jobs_dir, N):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-qr-decomp_{N}",
        0
    )
    config = {
        "benchmark": "qr-decomp",
        "params": {
            "N": N,
            "reg-size": 4
        },
    }
    make_job(job_dir, {**base, **config})


rulesets = [
    # "expanding",
    "ruler",
    # "t2"
]

configs = [
    "wack",
    # "phased",
]

cost_function = [
    "original",
    # "alternative"
]

for rule in rulesets:
    for config in configs:
        for cf in cost_function:
            jobs = Path("jobs")
            base = base_job(rule, config, cf)

            # mat_mul(base, jobs 2, 2, 2, 2)
            # mat_mul(base, jobs, 2, 3, 3, 3)
            # mat_mul(base, jobs, 3, 3, 3, 3)
            # mat_mul(base, jobs, 4, 4, 4, 4)
            # mat_mul(base, jobs, 8, 8, 8, 8)
            # mat_mul(base, jobs, 10, 10, 10, 10)
            # mat_mul(base, jobs, 16, 16, 16, 16)

            # make_2d_conv(base, jobs, 3, 3, 2, 2)
            # make_2d_conv(base, jobs, 3, 3, 3, 3)
            # make_2d_conv(base, jobs, 3, 5, 3, 3)
            # make_2d_conv(base, jobs, 4, 4, 3, 3)
            # make_2d_conv(base, jobs, 8, 8, 3, 3)

            # make_2d_conv(base, jobs, 10, 10, 2, 2)
            # make_2d_conv(base, jobs, 10, 10, 3, 3)
            # make_2d_conv(base, jobs, 10, 10, 4, 4)

            # make_2d_conv(base, jobs, 16, 16, 2, 2)
            # make_2d_conv(base, jobs, 16, 16, 3, 3)
            # make_2d_conv(base, jobs, 16, 16, 4, 4)

            q_prod(base, jobs)

            # qr_decomp(base, jobs, 3)
            # qr_decomp(base, jobs, 4)
