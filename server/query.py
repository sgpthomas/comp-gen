from pathlib import Path
import json
import pandas as pd
from dfply import filter_by, X, dfpipe, select, mutate
from functools import reduce
from datetime import datetime


def cmp_params(a):
    if a.name == "params":
        return a.map(lambda x: x.replace("_", "x").split("x")).map(
            lambda l: reduce(lambda x, y: x * y, map(int, l))
        )
    else:
        return a


def latest_date():
    dates = set()
    for config_path in Path("completed").glob("**/config.json"):
        # exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        dates.add(datetime.strptime(config["date"], "%b%d-%H%M"))

    print("\n".join(map(lambda x: x.strftime("%b%d-%H%M"), sorted(dates))))
    latest = list(sorted(dates))[-1]
    return latest.strftime("%b%d-%H%M")


@dfpipe
def pivot_table(df, **kwargs):
    df = pd.pivot_table(df, **kwargs)
    return df.reset_index(drop=True, names=["index"])


@dfpipe
def reset_index(df, **kwargs):
    return df.reset_index(**kwargs)


@dfpipe
def sort_values(df, **kwargs):
    return df.sort_values(**kwargs)


@dfpipe
def to_csv(df, path, **kwargs):
    df.to_csv(path, **kwargs)
    print(f"Wrote {path}")
    return df


@dfpipe
def display(df):
    print(df.to_string())
    return df


@dfpipe
def agg(df, list):
    return df.agg(list)


def exp_iter(name, date=None):
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all(
            [
                "Mar27" in config["date"],
                name in config["name"],
                "pruning" in config["metadata"]
            ]
        ):
            df = (
                pd.read_csv(exp_path / "data.csv")
                >> filter_by((X.name == "nodes") | (X.name == "cost"))
                >> filter_by(X.iteration != "report")
                >> pivot_table(
                    index=["phase", "iteration"],
                    columns=["name"],
                    values="value",
                    sort=False,
                )
                >> mutate(pruning=config["metadata"]["alt_cost"])
                >> select(["pruning", "cost", "nodes"])
            )
            res += [df]

    out = Path("figs") / "data" / "2d-conv-3x3_3x3_iter.csv"
    final = pd.concat(res)
    final.to_csv(out, index_label="index")
    print(final)
    print(f"Wrote {out}")


def pruning():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all([
                # "Mar27-1209" in config["date"] or "Mar27-1552" in config["date"],
                "Mar28-1016" in config["date"],
                "key" in config and config["key"] == "pruning"
        ]):
            print(config["date"])
            print(exp_path)
            df = (pd.read_csv(exp_path / "data.csv")
                  >> filter_by((X.name == "cost") | (X.name == "timestamp") | (X.name == "nodes"))
                  >> filter_by(X.iteration != "report")
                  >> mutate(
                      date=config["date"],
                      benchmark=config["name"],
                      pruning="loop" in config["metadata"]["compile.json"])
                  >> select(["date", "benchmark", "pruning", "phase", "iteration", "name", "value"])
                  )
            res.append(df)

    df = pd.concat(res) >> reset_index(drop=True)
    print(df.to_string())
    out = Path("figs") / "data" / "pruning.csv"
    df.to_csv(out, index_label="iter")
    print(f"Wrote {out}")


def compile_est_cycles():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if "_" in config["name"]:
            name, params = config["name"].split("_", 1)
        else:
            name = config["name"]
            params = "0"
        cycles_csv = exp_path / "results" / f"{name}.csv"

        if all([
                "Mar28" in config["date"],
                cycles_csv.exists(),
        ]):
            ruleset = Path(config["metadata"]["rules.json"]).stem
            memory = pd.read_csv(exp_path / "memory.csv", header=None, names=["timestamp", "ram_used"])
            memory = (memory
                      >> agg(["min", "max"]))
            compile_time = (memory >> agg(lambda x: x['max'] - x['min']))["timestamp"]
            ram_used = memory["ram_used"]["max"]

            df = (
                pd.read_csv(cycles_csv)
                >> mutate(
                    benchmark=name,
                    params=params,
                    ruleset=ruleset,
                    greedy=config["metadata"]["alt_cost"],
                    compile_time=compile_time,
                    max_ram_used=ram_used
                )
                >> select([
                    "kernel",
                    "benchmark",
                    "params",
                    "ruleset",
                    "greedy",
                    "cycles",
                    "compile_time",
                    "max_ram_used"
                ])
            )
            res.append(df)
            # print(df[df["kernel"] == "compgen"]["cycles"])

    df = (pd.concat(res)
          >> sort_values(by=["benchmark", "params"], key=cmp_params)
          >> reset_index(drop=True, names=["index"])
          >> display()
          >> to_csv(
              Path("figs") / "data" / "est_cycles.csv",
              index=False
    ))


def compile_times():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all(
            [
                "Mar20" in config["date"],
                "expanding" in config["metadata"]["rules.json"],
            ]
        ):
            memory = pd.read_csv(
                exp_path / "memory.csv", header=None, names=["timestamp", "ram_used"]
            )
            df = memory.agg(["min", "max"])
            max_ts = df.loc[["max"]]["timestamp"].values[0]
            min_ts = df.loc[["min"]]["timestamp"].values[0]
            killed = "killed" in list(memory["ram_used"].values)
            res.append(
                [
                    config["name"],
                    "pruning" if config["metadata"]["alt_cost"] else "stock",
                    max_ts - min_ts,
                    killed,
                ]
            )

    out = Path("figs") / "data" / "compile_times.csv"
    df = pd.DataFrame(res, columns=["benchmark", "type", "runtime", "killed"])
    df.to_csv(out, index_label="index")
    print(df)
    print(f"Wrote {out}")


def scheduler():
    runtime = []
    cost = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all(["Mar23" in config["date"]]):
            print(exp_path)
            memory = pd.read_csv(
                exp_path / "memory.csv", header=None, names=["timestamp", "ram_used"]
            )
            df = memory.agg(["min", "max"])
            max_ts = df.loc[["max"]]["timestamp"].values[0]
            min_ts = df.loc[["min"]]["timestamp"].values[0]
            runtime.append([config["name"], max_ts - min_ts])

            benchmark, params = config["name"].split("_", 1)
            df = (pd.read_csv(exp_path / "data.csv")
                  >> filter_by(X.name == "cost")
                  >> filter_by(X.iteration != "report")
                  >> mutate(benchmark=benchmark, params=params)
                  >> select(["benchmark", "params", "iteration", "value"]))
            cost += [df]

    # write data to csv
    df = pd.DataFrame(runtime, columns=["benchmark", "runtime"])
    out = Path("figs") / "data" / "backoff_fail.csv"
    df.to_csv(out, index_label="index")
    print(f"Wrote {out}")

    # write cost data to csv
    df = (pd.concat(cost) >> reset_index(drop=True))
    out = Path("figs") / "data" / "backoff_cost.csv"
    df.to_csv(out)
    print(f"Wrote {out}")


def play():
    _ = latest_date()

    # res = []
    # for config_path in Path("completed").glob("**/config.json"):
    #     exp_path = Path(config_path.parents[0])
    #     config = json.load(config_path.open("r"))

    #     if all(["Mar23" in config["date"]]):
    #         print(json.dumps(config, indent=2))
    # df = pd.concat(res)
    # print(df.to_string())


def main():
    # exp_iter("2d-conv_3x3_3x3")
    # pruning()
    compile_est_cycles()
    # compile_times()
    # scheduler()
    # play()


# if __name__ == "__main__":
main()
