#!/usr/bin/env python3

import json
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import click
import pandas as pd


def param_strings(benchmark, params):
    if benchmark == "2d-conv":
        i_matrix, f_matrix = params.split("_")
        i_rows, i_cols = i_matrix.split("x")
        f_rows, f_cols = f_matrix.split("x")
        return [
            f"-DI_ROWS='{i_rows}'",
            f"-DI_COLS={i_cols}",
            f"-DF_ROWS={f_rows}",
            f"-DF_COLS={f_cols}",
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
    elif benchmark == "qr-decomp":
        return [f"-DSIZE='{params}'"]
    else:
        raise Exception(f"Don't know how to generate param string for {benchmark}")


def build_exec_path(base, binary_name):
    if base is None:
        return binary_name
    elif isinstance(base, list):
        return str((Path("/".join(base)) / binary_name).expanduser())
    else:
        return str((Path(base) / binary_name).expanduser())


def estimate_kernel(
    exp_dir,
    force=False,
    results="results",
    override="",
    benchmark_name=None,
    params=None,
    debug=False,
    xtensa_root=None,
    xtensa_version="RI-2021.8-linux",
    dios_root=None,
):
    try:
        exp_path = Path(exp_dir)

        if (exp_path / "config.json").exists():
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

        # if a kernel file doesn't exist, try pulling one out of the log file
        if force or not (exp_path / results / "kernel.c").exists():
            # if res.rkt doesn't exists, try pulling one from the log
            if not (exp_path / results / "res.rkt").exists():
                stderr_log = exp_dir / "stderr.log"
                progs = list(
                    filter(
                        lambda l: "Best program: " in l,
                        stderr_log.open("r").readlines(),
                    )
                )
                if len(progs) == 0:
                    print("No kernel found.")
                    return

                prog = progs[-1].split(": ")[1]
                with (exp_path / results / "res.rkt").open("w") as res:
                    print(len(prog))
                    res.write(prog)

            # we have a res.rkt now, compile it
            subprocess.run(
                [
                    build_exec_path(dios_root, "dios"),
                    "-w",
                    "4",
                    "--egg",
                    "--suppress-git",
                    "-o",
                    str(exp_path / results / "kernel.c"),
                    str(exp_path / results),
                ],
                timeout=60 * 60 * 3,
            )

            if not (exp_path / results / "kernel.c").exists():
                print("Failed to produce kernel.c!")
                return

        # copy file into results directory
        shutil.copy(harness_file, exp_path / results / "harness.c")
        shutil.copy(util_file, exp_path / results / "utils.h")

        cmd = [
            # "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-clang++",
            build_exec_path(
                [xtensa_root, xtensa_version, "XtensaTools", "bin"], "xt-clang++"
            ),
            "-std=c++11",
            "-mlongcalls",
            "-O3",
            "-LNO:simd",
            "-fvectorize",
            "-mtext-section-literals",
            "-DXCHAL_HAVE_FUSIONG_SP_VFPU=1",
            "-DOUTFILE='\"cycles.csv\"'",
        ]
        cmd += param_strings(benchmark_name, params)
        cmd += [
            "-I",
            "/usr/include/eigen3",
            "-I",
            f"{xtensa_root}/fusiong3_library/include",
            # "~/Research/xtensa/fusiong3_library/include",
            "-I",
            f"{xtensa_root}/fusiong3_library/include_private",
            # "~/Research/xtensa/fusiong3_library/include_private",
            "kernel.c",
        ]

        # run xt compiler
        if force or not (exp_path / results / "kernel.o").exists():
            print("Compiling", end="...", flush=True)
            subprocess.run("rm -f kernel.o", shell=True, cwd=exp_path / results)
            # run once to generate .s files, and then again to generate object file

            if debug:
                subprocess.run(
                    " ".join(cmd + ["-S"]),
                    shell=True,
                    cwd=exp_path / results,
                    timeout=60 * 5,
                )
            subprocess.run(
                " ".join(cmd + ["harness.c", "-o", "kernel.o"]),
                shell=True,
                cwd=exp_path / results,
                timeout=60 * 60 * 3,
            )

            print("Done")

        # simulate the resulting object file
        print("Simulating", end="...", flush=True)
        xt_run_cmd = [
            build_exec_path(
                [xtensa_root, xtensa_version, "XtensaTools", "bin"], "xt-run"
            )
            # "~/Research/xtensa/RI-2021.8-linux/XtensaTools/bin/xt-run"
        ]
        if debug:
            xt_run_cmd += ["--client_commands='trace --level=0 trace.out'"]
        xt_run_cmd += ["kernel.o"]
        subprocess.run(
            " ".join(xt_run_cmd),
            shell=True,
            cwd=exp_path / results,
            capture_output=False,
        )
        print("Done")

        df = pd.read_csv(exp_path / results / "cycles.csv")

        if override != "":
            df = df.replace(to_replace="compgen", value=override)
            df.to_csv(exp_path / results / "cycles.csv")

        print(df)
        return df
    except subprocess.TimeoutExpired:
        print("Timeout!!")
        cycles_csv = exp_path / results / "cycles.csv"
        cycles_csv.touch()
        return pd.DataFrame()
    except FileNotFoundError as e:
        print("File not found")
        print(e)
        cycles_csv = exp_path / results / "cycles.csv"
        cycles_csv.touch()
        return pd.DataFrame()


@click.group()
def cli():
    pass


@cli.command()
@click.argument("exp_dir")
@click.option("--force", is_flag=True)
@click.option("--results", default="results")
@click.option("--name", default=None)
@click.option("--params", default=None)
@click.option("--debug", is_flag=True)
@click.option("--xtensa", default="~/xtensa")
@click.option("--dios", default="~/diospyros")
def single(exp_dir, force, results, name, params, debug, xtensa, dios):
    print(f"Estimating cycles for {exp_dir}")
    estimate_kernel(
        exp_dir,
        results=results,
        force=force,
        benchmark_name=name,
        params=params,
        debug=debug,
        xtensa_root=xtensa,
        dios_root=dios,
    )


@cli.command()
@click.argument("date")
@click.option("--force", is_flag=True)
@click.option("--key", default="performance")
@click.option("--override-name", default="")
@click.option("--debug", is_flag=True)
def many(date, force, key, override_name, debug):
    experiments = {}

    # gather all experiments with "performance" key
    for config_path in Path("completed").glob("**/config.json"):
        exp_dir = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all(
            [
                "key" in config and config["key"] == key,
                # force => cycles.csv doesn't exist
                force or (not (exp_dir / "results" / "cycles.csv").exists()),
            ]
        ):
            exp_date = config["date"]
            if exp_date in experiments:
                experiments[exp_date].append(exp_dir)
            else:
                experiments[exp_date] = [exp_dir]

    # handle the case when date is the string "latest"
    if date == "latest":
        options = sorted(
            experiments.keys(), key=lambda x: datetime.strptime(x, "%b%d-%H%M")
        )
        date = options[-1]
    elif date == "list":
        options = sorted(
            experiments.keys(), key=lambda x: datetime.strptime(x, "%b%d-%H%M")
        )
        for k in options:
            print(f"{k}")
            for exp in experiments[k]:
                print(f" - {exp}")
            print("")
        return

    if date in experiments:
        for exp_dir in experiments[date]:
            print(f"Running {exp_dir}")
            estimate_kernel(exp_dir, force=force, override=override_name, debug=debug)
    else:
        raise Exception(f"Experiment {date} not found! Options: {experiments.keys()}")


if __name__ == "__main__":
    cli()
