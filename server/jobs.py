import json
from datetime import datetime
from server import unique_name
from pathlib import Path
import subprocess


def make_job(job_dir, config):
    print(f"Created {config['benchmark']} job at {job_dir}")
    job_dir.mkdir(exist_ok=False)
    json.dump(
        config,
        (job_dir / "config.json").open("w"),
        indent=2
    )


def mat_mul(jobs_dir, ruleset_name, compile_config, a_rows, a_cols, b_rows, b_cols):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}",
        0
    )
    config = {
        "benchmark": "mat-mul",
        "ruleset": ruleset_name,
        "compile_config": compile_config,
        "params": {
            "A-rows": a_rows,
            "A-cols": a_cols,
            "B-rows": b_rows,
            "B-cols": b_cols,
            "reg-size": 4
        },
        "memory_limit": 220
    }
    make_job(job_dir, config)


def make_2d_conv(jobs_dir, ruleset_name, compile_config, irows, icols, frows, fcols):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(
        jobs_dir / f"{date_str}-2d-conv_{irows}x{icols}_{frows}x{fcols}",
        0
    )
    config = {
        "benchmark": "2d-conv",
        "ruleset": ruleset_name,
        "compile_config": compile_config,
        "params": {
            "input-rows": irows,
            "input-cols": icols,
            "filter-rows": frows,
            "filter-cols": fcols,
            "reg-size": 4
        },
        "memory_limit": 220
    }
    make_job(job_dir, config)


for rule in ["expanding", "ruler"]:
    # mat_mul(Path("jobs"), rule, "phased", 2, 2, 2, 2)
    # mat_mul(Path("jobs"), rule, "phased", 2, 3, 3, 3)
    # mat_mul(Path("jobs"), rule, "phased", 3, 3, 3, 3)
    mat_mul(Path("jobs"), rule, "phased", 4, 4, 4, 4)
    mat_mul(Path("jobs"), rule, "phased", 8, 8, 8, 8)

    make_2d_conv(Path("jobs"), rule, "phased", 3, 3, 2, 2)
    make_2d_conv(Path("jobs"), rule, "phased", 3, 3, 3, 3)
    make_2d_conv(Path("jobs"), rule, "phased", 3, 5, 3, 3)
    make_2d_conv(Path("jobs"), rule, "phased", 4, 4, 3, 3)
    make_2d_conv(Path("jobs"), rule, "phased", 8, 8, 3, 3)

    make_2d_conv(Path("jobs"), rule, "phased", 10, 10, 2, 2)
    make_2d_conv(Path("jobs"), rule, "phased", 10, 10, 3, 3)
    make_2d_conv(Path("jobs"), rule, "phased", 10, 10, 4, 4)

    make_2d_conv(Path("jobs"), rule, "phased", 16, 16, 2, 2)
    make_2d_conv(Path("jobs"), rule, "phased", 16, 16, 3, 3)
    make_2d_conv(Path("jobs"), rule, "phased", 16, 16, 4, 4)
