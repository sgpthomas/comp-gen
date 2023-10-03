#!/usr/bin/env python3

import datetime
import subprocess as sp
import sys
import time
from pathlib import Path
from typing import List

import click

EXPERIMENTS = [
    "overall",
    "pruning",
    "ruleset_ablation",
    "new_instructions",
    "alpha_beta",
]


def check_data(data: List[str], *, experiment_name: str):
    for d in data:
        if not (Path("server/figs/data") / d).exists():
            print(
                "The necessary data for this command does not exist. "
                + f"Run\n  ./aec.py gen-data {experiment_name}"
                + "\nto generate the data for this figure."
            )
            sys.exit(-1)


def call_generate(name: str, destination: str):
    sp.run(f"./R/generate.R {name}", shell=True, cwd="server/figs")
    sp.run("mkdir -p output", shell=True)
    sp.run(f"cp server/figs/{name}.pdf output/{destination}.pdf", shell=True)


def jobs(job_name, **kwargs):
    kwstr = " ".join([f"--{key} {value}" for key, value in kwargs.items()])
    sp.run(f"./jobs.py {job_name} {kwstr}", shell=True, cwd="server")


def sync(cmd: str, *args, name: str | None = None, ip: str | None = None, **kwargs):
    if name is not None:
        return sp.run(
            f"./sync.py {cmd} --name {name} {' '.join(args)}",
            shell=True,
            cwd="server",
            **kwargs,
        )
    elif ip is not None:
        return sp.run(
            f"./sync.py {cmd} --ip {ip} {' '.join(args)}",
            shell=True,
            cwd="server",
            **kwargs,
        )
    else:
        print("You need to provide at least one of `--name` or `--ip`.")
        sys.exit(-1)


def query(name):
    return sp.run(
        f"./query.py update {name} -t latest --commit", shell=True, cwd="server"
    )


def wait_then_process(query, *, name, ip, estimated_time=None):
    start_time = datetime.datetime.now()
    while sync("check", name=name, ip=ip, capture_output=True).returncode != 0:
        delta = datetime.datetime.now() - start_time
        print(f"Jobs running for {delta.seconds}s", end="", flush=True)
        if estimated_time is not None:
            print(f" [Estimated total ~{estimated_time}s]", flush=True)
        else:
            print("", flush=True)

        time.sleep(5)

    _process_data(query, ip, name)


def _make_figure(fig: str):
    """
    Pull this out as a separate function so that we can call
    it without exiting. The click command functions exit the
    process when the function finishes, making it impossible
    to implement the 'all' case the way that I do here.
    """

    match fig:
        case "figure4":
            check_data(
                ["diospyros.csv", "est_cycles.csv"],
                experiment_name="overall",
            )
            call_generate("cycle_performance", "figure4")
        case "figure5":
            check_data(
                ["diospyros.csv", "est_cycles.csv"],
                experiment_name="overall",
            )
            call_generate("compile_time", "figure5")
        case "figure6":
            check_data(
                ["pruning.csv"],
                experiment_name="pruning",
            )
            call_generate("pruning", "figure6")
        case "figure7":
            check_data(
                ["ruleset_ablation.csv", "diospyros.csv"],
                experiment_name="ruleset_ablation",
            )
            call_generate("ruleset_ablation", "figure7")
        case "figure8":
            check_data(
                ["rule_distribution.csv"], experiment_name="ruleset_distribution"
            )
            call_generate("ruleset_distribution", "figure8")
        case "figure9":
            check_data(["alpha_beta.csv"], experiment_name="ruleset_distribution")
            call_generate("alpha_beta", "figure9")
        case "table2":
            check_data(["instruction.csv"], experiment_name="instruction_ablation")
            call_generate("instruction_ablation", "table2")
        case "all":
            _make_figure("figure4")
            _make_figure("figure5")
            _make_figure("figure6")
            _make_figure("figure7")
            _make_figure("figure8")
            _make_figure("figure9")
            _make_figure("table2")
        case _:
            raise Exception("Unreachable")


def _process_data(experiment, ip, name):
    sync("download", "--clean", name=name, ip=ip)
    match experiment:
        case "overall":
            query("est_cycles")

        case "pruning":
            query("pruning")

        case "ruleset_ablation":
            query("ruleset_ablation")

        case "new_instructions":
            query("instruction")

        case "alpha_beta":
            query("alpha_beta")

        case _:
            raise Exception("Unreachable")


@click.group()
def cli():
    pass


@cli.command()
@click.argument(
    "fig",
    type=click.Choice(
        [
            "table2",
            "figure4",
            "figure5",
            "figure6",
            "figure7",
            "figure8",
            "figure9",
            "all",
        ]
    ),
)
def make(fig: str):
    _make_figure(fig)


@cli.command()
@click.argument("experiment", type=click.Choice(EXPERIMENTS))
@click.option("--all", is_flag=True)
@click.option("--no-wait", is_flag=True)
@click.option("--ip", type=str)
@click.option("--name", type=str)
def gen_data(experiment, all, no_wait, ip, name):
    match experiment:
        case "overall":
            if all:
                jobs("overall_performance")
            else:
                jobs("fast_overall_performance")

            jobs("estimate:performance", after="performance")
            sync("upload", "--clean", name=name, ip=ip)

            if not no_wait:
                wait_then_process(
                    "overall", name=name, ip=ip, estimated_time=600 if not all else None
                )

        case "pruning":
            if all:
                jobs("pruning")
            else:
                jobs("fast_pruning")

            jobs("estimate:pruning", after="pruning")
            sync("upload", "--clean", name=name, ip=ip)

            if not no_wait:
                wait_then_process(
                    "pruning", name=name, ip=ip, estimated_time=780 if not all else None
                )

        case "ruleset_ablation":
            if all:
                # jobs("ruleset_synthesis")
                jobs("ruleset_ablation", rulesets="rulesets/ablation")
            else:
                jobs("fast_ruleset_ablation", rulesets="rulesets/ablation")

            jobs("estimate:ruleset_ablation", after="ruleset_ablation")
            sync("upload", "--clean", name=name, ip=ip)

            if not no_wait:
                wait_then_process(
                    "ruleset_ablation",
                    name=name,
                    ip=ip,
                    estimated_time=600 if not all else None,
                )

        case "new_instructions":
            if all:
                raise NotImplementedError("Not yet tested")
            else:
                # jobs("")
                pass

            sync("upload", "--clean", name=name, ip=ip)

        case "alpha_beta":
            if all:
                jobs("alpha_beta_ablation")
            else:
                jobs("fast_alpha_beta_ablation")

            jobs("estimate:alpha-beta", after="alpha-beta")
            sync("upload", "--clean", name=name, ip=ip)

            if not no_wait:
                wait_then_process(
                    "alpha_beta",
                    name=name,
                    ip=ip,
                    estimated_time=600 if not all else None,
                )

        case _:
            raise Exception("Unreachable")


@cli.command()
@click.argument("experiment", type=click.Choice(EXPERIMENTS))
@click.option("--ip", type=str)
@click.option("--name", type=str)
def process_data(experiment, ip, name):
    _process_data(experiment, ip, name)


if __name__ == "__main__":
    cli()
