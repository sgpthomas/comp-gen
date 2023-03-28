#!/usr/bin/env python3

import click
from pathlib import Path
import json
import shutil
import subprocess
import pandas as pd
from datetime import datetime


def param_strings(benchmark, params):
    if benchmark == "2d-conv":
        i_matrix, f_matrix = params.split("_")
        i_rows, i_cols = i_matrix.split("x")
        f_rows, f_cols = f_matrix.split("x")
        return [
            f"-DI_ROWS='{i_rows}'",
            f"-DI_COLS={i_cols}",
            f"-DF_ROWS={f_rows}",
            f"-DF_COLS={f_cols}"
        ]
    elif benchmark == "q-prod":
        return []
    elif benchmark == "mat-mul":
        a_matrix, b_matrix = params.split("_")
        a_rows, a_cols = a_matrix.split("x")
        b_rows, b_cols = b_matrix.split("x")
        return [
            f"-DA_ROWS='{a_rows}'",
            f"-DA_COLS='{a_cols}'",
            f"-DB_ROWS='{b_rows}'",
            f"-DB_COLS='{b_cols}'",
        ]
    else:
        raise Exception(f"Don't know how to generate param string for {benchmark}")


def estimate_kernel(exp_dir):
    exp_path = Path(exp_dir)
    config = json.load((exp_path / "config.json").open("r"))

    if "_" in config["name"]:
        benchmark_name, params = config["name"].split("_", 1)
    else:
        benchmark_name = config["name"]
        params = None

    # find the harness file that is associated with this benchmark
    # looks in `harnesses/benchmark_name.c`
    harness_file = Path("harnesses") / f"{benchmark_name}.c"
    util_file = Path("harnesses") / "utils.h"

    if not harness_file.exists():
        print(f"Harness file for {benchmark_name} doesn't exist.")
        return

    if not (exp_path / "results" / "kernel.c").exists():
        print("Kernel file not found.")
        return

    # copy file into results directory
    shutil.copy(harness_file, exp_path / "results" / "harness.c")
    shutil.copy(util_file, exp_path / "results" / "utils.h")

    cmd = [
        "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-clang++",
        "-std=c++11", "-O3", "-mlongcalls", "-LNO:SIMD", "-w", "-mtext-section-literals",
        "-DXCHAL_HAVE_FUSIONG_SP_VFPU=1",
        f"-DOUTFILE='\"{benchmark_name}.csv\"'"
    ]
    cmd += param_strings(benchmark_name, params)
    cmd += [
        "-I", "/usr/include/eigen3",
        "-I", "~/Research/xtensa/fusiong3_library/include",
        "-I", "~/Research/xtensa/fusiong3_library/include_private",
        "kernel.c", "harness.c",
        "-o", f"{benchmark_name}.o"
    ]

    # run xt compiler
    if not (exp_path / "results" / f"{benchmark_name}.o").exists():
        print("Compiling", end="...", flush=True)
        subprocess.run(" ".join(cmd), shell=True, cwd=exp_path / "results")
        print("Done")

    # simulate the resulting object file
    print("Simulating", end="...", flush=True)
    subprocess.run(" ".join([
        "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-run",
        f"{benchmark_name}.o"
    ]), shell=True, cwd=exp_path / "results", capture_output=True)
    print("Done")

    df = pd.read_csv(exp_path / "results" / f"{benchmark_name}.csv")
    print(df)


@click.group()
def cli():
    pass


@cli.command()
@click.argument("exp_dir")
def single(exp_dir):
    print(f"Estimating cycles for {exp_dir}")
    estimate_kernel(exp_dir)


@cli.command()
@click.argument("date")
def all(date):

    experiments = {}

    # gather all experiments with "performance" key
    for config_path in Path("completed").glob("**/config.json"):
        exp_dir = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "key" in config and config["key"] == "performance":
            exp_date = config["date"]
            if len(list((exp_dir / "results").glob("*.csv"))) == 0 and exp_date in experiments:
                experiments[exp_date].append(exp_dir)
            else:
                experiments[exp_date] = [exp_dir]
    if date == "latest":
        date = sorted(
            experiments.keys(),
            key=lambda x: datetime.strptime(x, "%b%d-%H%M")
        )[-1]

    if date in experiments:
        for exp_dir in experiments[date]:
            print(f"Running {exp_dir}")
            estimate_kernel(exp_dir)
    else:
        raise Exception(f"Experiment {date} not found! Options: {experiments.keys()}")


if __name__ == "__main__":
    cli()
