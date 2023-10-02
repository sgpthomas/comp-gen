#!/usr/bin/env python3

import subprocess as sp
import sys
from pathlib import Path
from typing import List

import click


def check_files(files: List[str], *, experiment_name: str):
    for f in files:
        if not Path(f).exists():
            print(
                "The necessary data for this command does not exist. "
                + f"Run\n  `./aec.py gen-data {experiment_name}`"
                + "\nto generate the data for this figure."
            )
            sys.exit(-1)


@click.group()
def cli():
    pass


@cli.command()
@click.argument("fig", type=click.Choice(["4", "5", "6", "7", "8", "9"]))
def make_figure(fig: int):
    match fig:
        case "4":
            check_files(
                ["server/figs/data/diospyros.csv", "server/figs/data/est_cycles.csv"],
                experiment_name="overall",
            )
            sp.run("./R/generate.R cycle_performance", shell=True, cwd="server/figs")
            sp.run("cp server/figs/cycle_performance.pdf output", shell=True)
        case "5":
            print("hi")
        case "6":
            print("hi")
        case "7":
            print("hi")
        case "8":
            print("hi")
        case "9":
            print("hi")
        case _:
            raise Exception("Unreachable")


if __name__ == "__main__":
    cli()
