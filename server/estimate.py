#!/usr/bin/env python3

import click
from pathlib import Path
import json
import shutil
import subprocess
import pandas as pd
from datetime import datetime
import process as p
from dfply import mutate
from query import reset_index, to_csv, select


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


def estimate_kernel(exp_dir, force=False, results="results"):
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

    if not (exp_path / results / "kernel.c").exists():
        print("Kernel file not found.")
        return

    # copy file into results directory
    shutil.copy(harness_file, exp_path / results / "harness.c")
    shutil.copy(util_file, exp_path / results / "utils.h")

    cmd = [
        "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-clang++",
        "-std=c++11", "-O3", "-mlongcalls", "-LNO:SIMD", "-w", "-mtext-section-literals",
        "-DXCHAL_HAVE_FUSIONG_SP_VFPU=1",
        "-DOUTFILE='\"cycles.csv\"'"
    ]
    cmd += param_strings(benchmark_name, params)
    cmd += [
        "-I", "/usr/include/eigen3",
        "-I", "~/Research/xtensa/fusiong3_library/include",
        "-I", "~/Research/xtensa/fusiong3_library/include_private",
        "kernel.c", "harness.c",
        "-o", "kernel.o"
    ]

    # run xt compiler
    if force or not (exp_path / results / "kernel.o").exists():
        print("Compiling", end="...", flush=True)
        subprocess.run("rm -f kernel.o", shell=True, cwd=exp_path / results)
        subprocess.run(" ".join(cmd), shell=True, cwd=exp_path / results)
        print("Done")

    # simulate the resulting object file
    print("Simulating", end="...", flush=True)
    subprocess.run(" ".join([
        "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-run",
        "kernel.o"
    ]), shell=True, cwd=exp_path / results, capture_output=True)
    print("Done")

    df = pd.read_csv(exp_path / results / "cycles.csv")
    print(df)
    return df


@click.group()
def cli():
    pass


@cli.command()
@click.argument("exp_dir")
def single(exp_dir):
    print(f"Estimating cycles for {exp_dir}")
    estimate_kernel(exp_dir, results="results")


@cli.command()
@click.argument("date")
@click.option("--force", is_flag=True)
def many(date, force):

    experiments = {}

    # gather all experiments with "performance" key
    for config_path in Path("completed").glob("**/config.json"):
        exp_dir = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all([
                "key" in config and config["key"] == "performance",
                # force => cycles.csv doesn't exist
                force or (not (exp_dir / "results" / "cycles.csv").exists())
        ]):
            exp_date = config["date"]
            if exp_date in experiments:
                experiments[exp_date].append(exp_dir)
            else:
                experiments[exp_date] = [exp_dir]

    # handle the case when date is the string "latest"
    if date == "latest":
        options = sorted(
            experiments.keys(),
            key=lambda x: datetime.strptime(x, "%b%d-%H%M")
        )
        date = options[-1]

    if date in experiments:
        for exp_dir in experiments[date]:
            print(f"Running {exp_dir}")
            estimate_kernel(exp_dir, force=force)
    else:
        raise Exception(f"Experiment {date} not found! Options: {experiments.keys()}")


@cli.command()
@click.argument("exp_dir")
def log(exp_dir):
    """
    Run estimation for every iteration for a single log file.
    """

    exp_dir = Path(exp_dir)
    assert exp_dir.exists()
    assert (exp_dir / "stderr.log").exists()

    prog_filter = p.Chunker(
        start=p.matches(r"Starting Phase (\w+)", lambda m: m.group(1)),
        combine=p.unique_dict_append,
        data=p.Combine(
            p.Chunker(
                start=p.matches(r"Iteration (\d+)", lambda m: int(m.group(1))),
                combine=p.dict_combine,
                data=p.LineFilter(
                    r"Best program: (\(.*\))",
                    lambda m: {"prog": m.group(1)}
                )
            )
        )
    )

    # setup new results directory to do estimating in
    iter_results = exp_dir / "iter_results"
    iter_results.mkdir(exist_ok=True)

    shutil.copy(exp_dir / "results" / "spec.rkt", iter_results)
    shutil.copy(exp_dir / "results" / "outputs.rkt", iter_results)
    shutil.copy(exp_dir / "results" / "prelude.rkt", iter_results)

    stderr_log = (exp_dir / "stderr.log").open("r").readlines()
    res = []
    n = 0
    for phase_name, iters in prog_filter.run(stderr_log).items():
        for iter_n, prog in iters[0].items():
            print(f"Estimating {phase_name}: {iter_n}", end="...\n")
            if len(prog) == 0:
                print(f"no program? {phase_name}, {iter_n}")
                continue
            with (iter_results / "res.rkt").open("w") as f:
                f.write(prog[0]["prog"])
                f.write("\n")
                n += 1
            shutil.copy(iter_results / "res.rkt", iter_results / f"res-{n}.rkt")
            subprocess.run([
                "../../diospyros/dios", "-w", "4",
                "--egg", "--suppress-git", "-o", str(iter_results / "kernel.c"),
                str(iter_results)
            ])
            res.append(estimate_kernel(exp_dir, force=True, results="iter_results")
                       >> mutate(
                           phase=phase_name,
                           iter=iter_n)
                       >> select(["kernel", "cycles", "correct", "phase", "iter"])
                       )
            print("Done")
    df = (pd.concat(res)
          >> reset_index(drop=True)
          >> to_csv(Path("figs") / "data" / "every_iteration.csv"))
    print(df)


if __name__ == "__main__":
    cli()
