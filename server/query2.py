#!/usr/bin/env python3.11

import click
import json
import pandas as pd
from pathlib import Path
from dfply import filter_by, X, select
from utils import sort_values, sorter, reset_index, display, agg, replace, iloc, to_csv
from datetime import datetime


QUERIES = {}


def query(key=None, pinned_date=None):
    def inner(func):
        QUERIES[func.__name__] = {
            "key": key,
            "pinned_date": pinned_date,
            "func": func
        }

    return inner


def all_experiments(base="completed", key=None, time=None):
    res = []
    for config_path in Path(base).glob("**/config.json"):
        config = json.load(config_path.open("r"))
        key = None
        if "key" in config:
            key = config["key"]
        date = config["date"]
        exp_dir = Path(config_path.parents[0])
        if "_" in config["name"]:
            name, params = config["name"].split("_", 1)
        else:
            name = config["name"]
            params = "0"
        res.append(pd.DataFrame(data={
            "date": [date],
            "key": [key],
            "benchmark": [name],
            "params": [params],
            "exp_dir": [exp_dir],
            "datetime": [datetime.strptime(date, "%b%d-%H%M")]
        }))
    exps = pd.concat(res) >> sort_values(
        by=["date", "key", "benchmark", "params"],
        key=sorter
    ) >> reset_index(drop=True)

    if key is not None:
        exps >>= filter_by(X.key == key)

    if time == "latest":
        max_time = exps["datetime"].max()
        exps >>= filter_by(X.datetime == max_time)
    elif time is not None:
        exps >>= filter_by(X.date.str.contains(time))

    return exps


def memory_df(exp_path):
    return pd.read_csv(
        exp_path / "memory.csv", header=None, names=["timestamp", "ram_used"]
    )


def data_df(exp_path):
    return pd.read_csv(exp_path / "data.csv")


def ruleset(exp_path):
    ruleset = json.load((exp_path / "config.json").open("r"))["metadata"]["rules.json"]
    return Path(ruleset).stem


def soft_timeout(exp_path):
    return json.load((exp_path / "compile.json").open("r"))["timeout"]


def hard_timeout(exp_path):
    return json.load((exp_path / "config.json").open("r"))["timeout"]


def max_ram(exp_path):
    memory = memory_df(exp_path) >> replace({"timeout": "-1"}) >> agg(["max"])
    return memory["ram_used"]["max"]


def killed(exp_path):
    return "killed" in list(memory_df(exp_path)["ram_used"].values)


def timed_out(exp_path):
    return "timeout" in list(memory_df(exp_path)["ram_used"].values)


def total_time(exp_path):
    df = memory_df(exp_path) >> filter_by(X.ram_used != "timeout", X.ram_used != "killed")
    return df["timestamp"] >> agg(["max", "min"]) >> agg(lambda x: x["max"] - x["min"])


def extraction_time(exp_path):
    extraction_df = (data_df(exp_path) >> filter_by(X.name == "extraction"))
    return extraction_df["value"].astype("float").sum()


def compile_time(exp_path):
    return total_time(exp_path) - extraction_time(exp_path)


def cycles(exp_path):
    cycles_csv = exp_path / "results" / "cycles.csv"
    if cycles_csv.exists():
        data = pd.read_csv(cycles_csv)
        if len(data["cycles"].values) > 0:
            return data["cycles"].values[0]
    return -1


def egraph_cost(exp_path):
    cost = (
        data_df(exp_path)
        >> filter_by(X.iteration == "report")
        >> filter_by(X.name == "cost")
        >> iloc([-1])
    )
    if len(cost["value"].values) > 0:
        return cost["value"].values[0]
    else:
        return -1


@query(key="performance", pinned_date="Apr12-1031")
def est_cycles(row):
    x = row.exp_dir
    return {
        "kernel": ["compgen"],
        "benchmark": [row.benchmark],
        "params": [row.params],
        "exp": [row.exp_dir.name],
        "ruleset": [ruleset(x)],
        "timeout": [soft_timeout(x)],
        "hard_timeout": [hard_timeout(x)],
        "cycles": [cycles(x)],
        "cost": [egraph_cost(x)],
        "compile_time": [compile_time(x)],
        "extraction_time": [extraction_time(x)],
        "total_time": [total_time(x)],
        "max_ram_used": [max_ram(x)]
    }


@click.group()
def cli():
    pass


@cli.command()
@click.argument("key")
@click.option("-t", "--time")
def ls(key, time):
    all_experiments(key=key, time=time) >> select(~X.datetime) >> display()


@cli.command()
@click.argument("name")
@click.option("-t", "--time")
@click.option("--commit", is_flag=True)
@click.option("--diff", is_flag=True)
def update(name, time, commit, diff):
    if name not in QUERIES:
        raise Exception(f"{name} is not a known query. Choose:\n{list(QUERIES.keys())}")

    config = QUERIES[name]
    if time is not None:
        query_date = time
    else:
        query_date = config["pinned_date"]
    exps = all_experiments(key=config["key"], time=query_date)

    res = []
    for _, row in exps.iterrows():
        res.append(pd.DataFrame(data=config["func"](row)))
    df = pd.concat(res) >> reset_index(drop=True) >> display()

    out = Path("figs") / "data" / f"{name}.csv"

    if diff:
        print("Old version:")
        pd.read_csv(out) >> display()

    if commit:
        df >> to_csv(out, indent=False)


if __name__ == "__main__":
    cli()
