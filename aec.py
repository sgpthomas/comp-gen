#!/usr/bin/env python3

import subprocess as sp
import sys
from pathlib import Path
from typing import List

import click


def check_data(data: List[str], *, experiment_name: str):
    for d in data:
        if not (Path("server/figs/data") / d).exists():
            print(
                "The necessary data for this command does not exist. "
                + f"Run\n  ./aec.py gen-data {experiment_name}"
                + "\nto generate the data for this figure."
            )
            sys.exit(-1)


def call_generate(name: str):
    sp.run(f"./R/generate.R {name}", shell=True, cwd="server/figs")
    sp.run(f"cp server/figs/{name}.pdf output", shell=True)


@click.group()
def cli():
    pass


@cli.command()
@click.argument("fig", type=click.Choice(["4", "5", "6", "7", "8", "9"]))
def make_figure(fig: int):
    match fig:
        case "4":
            check_data(
                ["diospyros.csv", "est_cycles.csv"],
                experiment_name="overall",
            )
            call_generate("cycle_performance")
        case "5":
            check_data(
                ["diospyros.csv", "est_cycles.csv"],
                experiment_name="overall",
            )
            call_generate("compile_time")
        case "6":
            check_data(
                ["pruning.csv"],
                experiment_name="pruning",
            )
            call_generate("pruning")
        case "7":
            check_data(
                ["ruleset_ablation.csv", "diospyros.csv"],
                experiment_name="ruleset_ablation",
            )
            call_generate("ruleset_ablation")
        case "8":
            check_data(
                ["rule_distribution.csv"], experiment_name="ruleset_distribution"
            )
            call_generate("ruleset_distribution")
        case "9":
            check_data(["alpha_beta.csv"], experiment_name="ruleset_distribution")
            call_generate("alpha_beta")
        case _:
            raise Exception("Unreachable")


if __name__ == "__main__":
    cli()
