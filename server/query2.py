#!/usr/bin/env python3.11

import click
import json
import pandas as pd
from pathlib import Path
from dfply import filter_by, X, select, mutate
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


def all_experiments(base="completed", query_key=None, query_time=None):
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

    if query_key is not None:
        exps >>= filter_by(X.key == query_key)

    if query_time == "latest":
        max_time = exps["datetime"].max()
        exps >>= filter_by(X.datetime == max_time)
    elif query_time is not None:
        exps >>= filter_by(X.date.str.contains(query_time))

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
    if len(df.index) == 1:
        return df["timestamp"].values[0]
    elif len(df.index) > 1:
        return df["timestamp"] >> agg(["max", "min"]) >> agg(lambda x: x["max"] - x["min"])
    else:
        return -1


def extraction_time(exp_path):
    extraction_df = (data_df(exp_path) >> filter_by(X.name == "extraction"))
    return extraction_df["value"].astype("float").sum()


def compile_time(exp_path):
    return max(0, total_time(exp_path) - extraction_time(exp_path))


def cycles(exp_path):
    cycles_csv = exp_path / "results" / "cycles.csv"
    try:
        data = pd.read_csv(cycles_csv)
        if len(data["cycles"].values) > 0:
            return data["cycles"].values[0]
    except Exception:
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


def diospyros_cycles(egg_kernel_csv):
    exp_dir = Path(egg_kernel_csv.parents[0])
    benchmark = egg_kernel_csv.parents[1].stem
    if benchmark != "q-prod":
        params = egg_kernel_csv.parents[0].stem.rsplit("_", 1)[0]
    else:
        params = "0"

    stats = json.load((exp_dir / "stats.json").open("r"))

    try:
        return (pd.read_csv(egg_kernel_csv)
                >> mutate(
                    benchmark=benchmark,
                    params=params,
                    compile_time=stats["time"],
                    max_ram_used=stats["memory"] / float(10 ** 9),
                    saturated=stats["saturated"])
                >> replace({
                    "Diospyros": "dios",
                    "Naive": "naive",
                    "Naive hard size vec": "naive.clang",
                    "Naive hard size": "naive.fixed",
                    "Nature": "nature",
                    "Eigen": "eigen"
                })
                >> select([
                    "kernel",
                    "benchmark",
                    "params",
                    "cycles",
                    "compile_time",
                    "max_ram_used",
                    "saturated"
                ]))
    except pd.errors.EmptyDataError:
        return pd.DataFrame()


@query(key="performance", pinned_date="Apr15-1612")
def est_cycles(row):
    x = row.exp_dir
    return pd.DataFrame(data={
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
    })


@query(key="diospyros", pinned_date="Apr16-1712")
def diospyros(row):
    return (pd.concat(map(diospyros_cycles, row.exp_dir.glob("**/egg-kernel.csv")))
            >> sort_values(by=["benchmark", "params", "kernel"], key=sorter)
            >> reset_index(drop=True, names=["index"]))


@query(key="instruction", pinned_date="Apr16-1443")
def instruction(row):
    return pd.DataFrame(data={
        "date": [row.date],
        "benchmark": [row.benchmark],
        "param": [row.params],
        "exp": [row.exp_dir.name],
        "rules": [ruleset(row.exp_dir)],
        "cycles": [cycles(row.exp_dir)],
        "cost": [egraph_cost(row.exp_dir)],
        "dir": [row.exp_dir]
    })


@click.group()
def cli():
    pass


@cli.command()
@click.argument("key")
@click.option("-t", "--time")
@click.option("-m", "--metric", multiple=True)
def ls(key, time, metric):
    exps = all_experiments(query_key=key, query_time=time)

    res = []
    for _, row in exps.iterrows():
        data = {
            "date": [row.date],
            "key": [row.key],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp_dir": [row.exp_dir]
        }
        for m in metric:

            val = None
            if m == "soft_timeout":
                val = soft_timeout(row.exp_dir)
            else:
                raise Exception

            data[m] = val

        res.append(pd.DataFrame(data=data))

    pd.concat(res) >> reset_index(drop=True) >> display()


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
    exps = all_experiments(query_key=config["key"], query_time=query_date)

    res = []
    for _, row in exps.iterrows():
        res.append(config["func"](row))
    df = pd.concat(res) >> reset_index(drop=True) >> display()

    out = Path("figs") / "data" / f"{name}.csv"

    if diff and out.exists():
        print("Old version:")
        pd.read_csv(out) >> display()

    if commit:
        new_time = exps["date"].unique()
        print(f"Pin this time: {new_time}")
        df >> to_csv(out)


if __name__ == "__main__":
    cli()
