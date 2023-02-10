import json
from datetime import datetime
from server import unique_name
from pathlib import Path


def make_job(job_dir, config):
    print(f"Created {config['benchmark']} job at {job_dir}")
    job_dir.mkdir(exist_ok=False)
    json.dump(config, (job_dir / "config.json").open("w"))


def mat_mul(jobs_dir, ruleset_name, compile_config, a_rows, a_cols, b_rows, b_cols):
    config = None
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(jobs_dir / f"{date_str}-mat-mul", 0)
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
        "memory_limit": 10
    }
    make_job(job_dir, config)


mat_mul(Path("jobs"), "expanding", "phased", 2, 3, 3, 3)
