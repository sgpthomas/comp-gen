#!/usr/bin/env python3.11

import json
from datetime import datetime
from pathlib import Path

import click
import pandas as pd
from dfply import X, filter_by, mutate, select

from utils import (agg, display, iloc, replace, reset_index, sort_values,
                   sorter, to_csv)

QUERIES = {}


def query(key=None, pinned_date=None, sort_keys=None):
    def inner(func):
        QUERIES[func.__name__] = {
            "key": key,
            "pinned_date": pinned_date,
            "func": func,
            "sort_keys": sort_keys if sort_keys is not None else [],
        }

    return inner


def parse_date(date):
    try:
        return datetime.strptime(date, "%b%d-%H%M")
    except ValueError:
        try:
            return datetime.strptime(date, "%b%d")
        except ValueError:
            return None


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
        res.append(
            pd.DataFrame(
                data={
                    "date": [date],
                    "key": [key],
                    "benchmark": [name],
                    "params": [params],
                    "exp_dir": [exp_dir],
                    "datetime": [parse_date(date)],
                }
            )
        )
    exps = (
        pd.concat(res)
        >> sort_values(by=["date", "key", "benchmark", "params"], key=sorter)
        >> reset_index(drop=True)
    )

    if query_key is not None:
        exps >>= filter_by(X.key == query_key)

    if query_time == "latest":
        max_time = exps["datetime"].max()
        exps >>= filter_by(X.datetime == max_time)
    elif isinstance(query_time, list):
        matches = []
        for qt in query_time:
            matches.append(exps >> filter_by(X.date.str.contains(qt)))
        exps = pd.concat(matches)
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


def compile_json(exp_path):
    c = json.load((exp_path / "config.json").open("r"))["metadata"]["compile.json"]
    return Path(c).stem


def soft_timeout(exp_path):
    try:
        return json.load((exp_path / "compile.json").open("r"))["timeout"]
    except Exception:
        return None


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
    df = memory_df(exp_path) >> filter_by(
        X.ram_used != "timeout", X.ram_used != "killed"
    )
    if len(df.index) == 1:
        return df["timestamp"].values[0]
    elif len(df.index) > 1:
        return (
            df["timestamp"] >> agg(["max", "min"]) >> agg(lambda x: x["max"] - x["min"])
        )
    else:
        return -1


def extraction_time(exp_path):
    extraction_df = data_df(exp_path) >> filter_by(X.name == "extraction")
    return extraction_df["value"].astype("float").sum()


def eqsat_time(exp_path):
    eqsat_df = data_df(exp_path) >> filter_by(X.name == "time")
    return eqsat_df["value"].astype("float").sum()


def compile_time(exp_path):
    return max(0, total_time(exp_path) - extraction_time(exp_path))


def cycles(exp_path):
    cycles_csv = exp_path / "results" / "cycles.csv"
    try:
        data = pd.read_csv(cycles_csv)
        if len(data["cycles"].values) > 0:
            return data["cycles"].values[0]
    except Exception as e:
        print(e)
        return None


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


def ruleset_timeout(exp_path):
    rules = json.load((exp_path / "rules.json").open("r"))
    if "abs_timeout" in rules["params"]:
        return rules["params"]["abs_timeout"]
    else:
        return None


def phase_count(exp_path, phase):
    return (
        data_df(exp_path)
        >> filter_by(X.phase.str.contains(phase))
        >> filter_by(X.iteration == "report")
        >> filter_by(X.name == "iterations")
    )["value"].count()


def phase_iterations(exp_path, phase):
    return (
        (
            data_df(exp_path)
            >> filter_by(X.phase.str.contains(phase))
            >> filter_by(X.iteration == "report")
            >> filter_by(X.name == "iterations")
        )["value"]
        .astype("int")
        .sum()
    )


def pruning_ruleset(exp_path):
    config = json.load((exp_path / "config.json").open("r"))
    if "pruning" in config["metadata"]:
        return config["metadata"]["pruning"]
    else:
        # for older cases where I hadn't set the metadata key properly yet
        compile_json = json.load((exp_path / "compile.json").open("r"))
        fresh_egraph = compile_json["phase"]["phases"][0]["phases"][0]["fresh_egraph"]
        # fresh_egraph -> yes pruning
        return fresh_egraph


def extract_alpha_beta(exp_path):
    compile_config = json.load((exp_path / "config.json").open("r"))["metadata"][
        "compile.json"
    ]
    stem = Path(compile_config).stem
    _, alpha, beta = stem.split("_")
    return float(alpha[1:]), float(beta[1:])


def diospyros_cycles(egg_kernel_csv):
    exp_dir = Path(egg_kernel_csv.parents[0])
    benchmark = egg_kernel_csv.parents[1].stem
    if benchmark != "q-prod":
        params = egg_kernel_csv.parents[0].stem.rsplit("_", 1)[0]
    else:
        params = "0"

    stats = json.load((exp_dir / "stats.json").open("r"))
    lines = (exp_dir / "compile-log.txt").open("r").readlines()
    eqsat_times = list(
        filter(
            lambda x: x != 0,
            map(lambda x: int(x.split(": ")[1]) if "Eqsat" in x else 0, lines),
        )
    )
    eqsat_time = 0 if len(eqsat_times) == 0 else eqsat_times[0] / 1000.0

    try:
        return (
            pd.read_csv(egg_kernel_csv)
            >> mutate(
                benchmark=benchmark,
                params=params,
                total_time=stats["time"],
                eqsat_time=eqsat_time,
                max_ram_used=stats["memory"] / float(10**9),
                saturated=stats["saturated"],
            )
            >> replace(
                {
                    "Diospyros": "dios",
                    "Naive": "naive",
                    "Naive hard size vec": "naive.clang",
                    "Naive hard size": "naive.fixed",
                    "Nature": "nature",
                    "Eigen": "eigen",
                }
            )
            >> select(
                [
                    "kernel",
                    "benchmark",
                    "params",
                    "cycles",
                    "total_time",
                    "eqsat_time",
                    "max_ram_used",
                    "saturated",
                ]
            )
        )
    except pd.errors.EmptyDataError:
        return pd.DataFrame()


@query(key="performance", pinned_date="Sep05")
def est_cycles(row):
    x = row.exp_dir
    return pd.DataFrame(
        data={
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
            "eqsat_time": [eqsat_time(x)],
            "total_time": [total_time(x)],
            "max_ram_used": [max_ram(x)],
            "pre_iterations": [phase_iterations(x, "pre")],
            "compile_iterations": [phase_iterations(x, "compile")],
            "opt_iter": [phase_iterations(x, "opt")],
        }
    )


@query(key="diospyros", pinned_date="Sep06-1712")
def diospyros(row):
    return (
        pd.concat(map(diospyros_cycles, row.exp_dir.glob("**/egg-kernel.csv")))
        >> sort_values(by=["benchmark", "params", "kernel"], key=sorter)
        >> reset_index(drop=True, names=["index"])
    )


@query(key="instruction", pinned_date="Apr18-1245")
def instruction(row):
    return pd.DataFrame(
        data={
            "date": [row.date],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "rules": [ruleset(row.exp_dir)],
            "cycles": [cycles(row.exp_dir)],
            "cost": [egraph_cost(row.exp_dir)],
            "dir": [row.exp_dir],
        }
    )


@query(key="ruleset_ablation", pinned_date="Apr18-1238")
def ruleset_ablation(row):
    return pd.DataFrame(
        data={
            "date": [row.date],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "ruleset": [ruleset_timeout(row.exp_dir)],
            "cost": [egraph_cost(row.exp_dir)],
            "cycles": [cycles(row.exp_dir)],
            "timed_out": [timed_out(row.exp_dir)],
            "dir": [row.exp_dir],
        }
    )


@query(
    key="pruning",
    pinned_date="Apr18-1240",
    sort_keys=["benchmark", "params", "pruning"],
)
def pruning(row):
    return pd.DataFrame(
        data={
            "date": [row.date],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "pruning": [pruning_ruleset(row.exp_dir)],
            "cycles": [cycles(row.exp_dir)],
            "cost": [egraph_cost(row.exp_dir)],
            "soft_timeout": [soft_timeout(row.exp_dir)],
            "killed": [killed(row.exp_dir)],
            "compile_time": [compile_time(row.exp_dir)],
            "memory_used": [max_ram(row.exp_dir)],
        }
    )


@query(key="pruning", sort_keys=["benchmark", "params", "pruning"])
def pruning_temp(row):
    if all([soft_timeout(row.exp_dir) == 360, pruning_ruleset(row.exp_dir)]):
        return pd.DataFrame(
            data={
                "date": [row.date],
                "benchmark": [row.benchmark],
                "params": [row.params],
                "exp": [row.exp_dir.name],
                "pruning": [pruning_ruleset(row.exp_dir)],
                "cycles": [cycles(row.exp_dir)],
                "cost": [egraph_cost(row.exp_dir)],
                "soft_timeout": [soft_timeout(row.exp_dir)],
                "killed": [killed(row.exp_dir)],
                "compile_time": [compile_time(row.exp_dir)],
                "memory_used": [max_ram(row.exp_dir)],
            }
        )
    else:
        return pd.DataFrame()


@query(
    key="optimization",
    pinned_date="Apr18-1100",
    sort_keys=["benchmark", "params", "optimization"],
)
def optimization(row):
    x = row.exp_dir
    return pd.DataFrame(
        data={
            "kernel": ["compgen"],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "optimization": ["w_opt" in compile_json(x)],
            "timeout": [soft_timeout(x)],
            "hard_timeout": [hard_timeout(x)],
            "cycles": [cycles(x)],
            "cost": [egraph_cost(x)],
            "dir": [row.exp_dir],
        }
    )


@query(key="performance", pinned_date="Apr11-1244")
def long(row):
    x = row.exp_dir
    if soft_timeout(x) == 1800:
        return pd.DataFrame(
            data={
                "date": [row.date],
                "kernel": ["compgen"],
                "benchmark": [row.benchmark],
                "params": [row.params],
                "exp": [row.exp_dir.name],
                "timeout": [soft_timeout(x)],
                "compile_time": [compile_time(x)],
                "cycles": [cycles(x)],
                "cost": [egraph_cost(x)],
                "dir": [row.exp_dir],
            }
        )
    else:
        return pd.DataFrame()


@query(key="large")  # , pinned_date=["Aug31-1417", "Aug31-1651"]
def large(row):
    x = row.exp_dir
    return pd.DataFrame(
        data={
            "date": [row.date],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "timeout": [soft_timeout(x)],
            "compile_time": [compile_time(x)],
            "cycles": [cycles(x)],
            "cost": [egraph_cost(x)],
            "killed": [killed(row.exp_dir)],
            "compile_time": [compile_time(row.exp_dir)],
            "memory_used": [max_ram(row.exp_dir)],
        }
    )


@query(key="alpha-beta", pinned_date="Aug31-1025")
def alpha_beta(row):
    x = row.exp_dir
    return pd.DataFrame(
        data={
            "date": [row.date],
            "benchmark": [row.benchmark],
            "params": [row.params],
            "exp": [row.exp_dir.name],
            "timeout": [soft_timeout(x)],
            "compile_time": [compile_time(x)],
            "cycles": [cycles(x)],
            "alpha": [extract_alpha_beta(x)[0]],
            "beta": [extract_alpha_beta(x)[1]],
            "cost": [egraph_cost(x)],
            "killed": [killed(row.exp_dir)],
            "compile_time": [compile_time(row.exp_dir)],
            "memory_used": [max_ram(row.exp_dir)],
        }
    )


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
            "exp_dir": [row.exp_dir],
        }
        for m in metric:
            if m == "soft_timeout":
                data[m] = [soft_timeout(row.exp_dir)]
            if m == "cost":
                data[m] = [egraph_cost(row.exp_dir)]

        res.append(pd.DataFrame(data=data))

    print(pd.concat(res) >> reset_index(drop=True))


@cli.command()
@click.argument("name", required=False)
@click.option("-t", "--time")
@click.option("--commit", is_flag=True)
@click.option("--diff", is_flag=True)
def update(name, time, commit, diff):
    query_list = "\n".join([f"- {x}" for x in QUERIES.keys()])
    if name is None:
        print(f"Available queries:\n{query_list}")
        return
    elif name not in QUERIES:
        print(f"{name} is not a known query. Available queries:\n{query_list}")
        return

    config = QUERIES[name]
    if time is not None:
        query_date = time
    else:
        query_date = config["pinned_date"]
    exps = all_experiments(query_key=config["key"], query_time=query_date)

    res = []
    for _, row in exps.iterrows():
        try:
            res.append(config["func"](row))
        except Exception as e:
            print(f"Something went wrong with {row}")
            print(e)
    df = (
        pd.concat(res)
        >> sort_values(by=config["sort_keys"], key=sorter)
        >> reset_index(drop=True)
        >> display()
    )

    data_dir = Path("figs") / "data"
    data_dir.mkdir(exist_ok=True)
    out = data_dir / f"{name}.csv"

    if diff and out.exists():
        print("Old version:")
        _ = pd.read_csv(out) >> display()

    if commit:
        new_time = exps["date"].unique()
        print(f"Pin this time: {new_time}")
        _ = df >> to_csv(out, index_label="index")

    return df


if __name__ == "__main__":
    cli()
