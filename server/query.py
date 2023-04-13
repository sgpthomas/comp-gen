from pathlib import Path
import json
import pandas as pd
from dfply import filter_by, X, dfpipe, select, mutate, spread
from datetime import datetime


def cmp_params(a):
    if a.name == "params":
        # return a.map(lambda x: x.replace("_", "x").split("x")).map(
        #     lambda l: reduce(lambda x, y: x * y, map(int, l))
        # )
        def per_key(el):
            if "_" in el:
                left, right = el.split("_")
                a, b = left.split("x")
                c, d = right.split("x")
                a, b, c, d = int(a), int(b), int(c), int(d)
                return ((a * b) ** 2) + (c * d)
            else:
                el

        return a.map(per_key)
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


@dfpipe
def replace(df, *args, **kwargs):
    return df.replace(*args, **kwargs)


@dfpipe
def iloc(df, *args):
    if df.empty:
        return df
    else:
        return df.iloc[*args]


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
                "pruning" in config["metadata"],
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

        if "_" in config["name"]:
            name, params = config["name"].split("_", 1)
        else:
            name = config["name"]
            params = "0"
        cycles_csv = exp_path / "results" / "cycles.csv"

        if all(
            [
                # "Mar27-1209" in config["date"] or "Mar27-1552" in config["date"],
                # "Apr12-1759" in config["date"], # or "Apr12-2041" in config["date"] or "Apr12-2118" in config["date"],
                # "Apr12-2353" in config["date"] or "Apr13-0925" in config["date"],
                "Apr13-1000" in config["date"],
                "key" in config and config["key"] == "pruning",
            ]
        ):
            print(config["date"])
            print(exp_path)
            print(json.dumps(config, indent=2))

            cycles = None
            egraph_cost = None
            if cycles_csv.exists():
                df = pd.read_csv(cycles_csv) >> display()
                egraph_cost = (pd.read_csv(exp_path / "data.csv")
                               >> filter_by(X.iteration == "report")
                               >> filter_by(X.name == "cost")
                               >> iloc([-1]))["value"].values[0]
                cycles = df["cycles"].values[0]

            memory_csv = exp_path / "memory.csv"
            df = pd.read_csv(memory_csv, header=None, names=["timestamp", "ram_used"])
            memory_used = (df >> replace({"timeout": "-1"}) >> agg(["max"]))["ram_used"].values[0]
            compile_time = (df >> agg(["max"]))["timestamp"].values[0]

            killed = "killed" in df["ram_used"].values
            timeout = "timeout" in df["ram_used"].values

            # I want to get name, params, exp, pruning, cycles, cost, nodes
            df = (
                pd.DataFrame(data={
                    "benchmark": [name],
                    "params": [params],
                    "exp": [exp_path.name],
                    "pruning": [not ("noprune" in config["metadata"]["compile.json"])],
                    "cycles": [cycles],
                    "cost": [egraph_cost],
                    "timeout": [timeout],
                    "killed": [killed],
                    "compile_time": [compile_time],
                    "memory_used": [memory_used]
                })
            )

            # data = (
            #     pd.read_csv(exp_path / "data.csv")
            #     >> filter_by(
            #         (X.name == "timestamp") | (X.name == "nodes")
            #     )
            #     >> filter_by(X.iteration != "report")
            #     >> spread(X.name, X.value)
            # )
            # df = (
            #     pd.read_csv(exp_path / "iter_estimation.csv")
            #     >> inner_join(
            #         data.astype({"iteration": "int64"}),
            #         by=["phase", "iteration"])
            #     >> mutate(
            #         pruning="loop" in config["metadata"]["compile.json"],
            #         benchmark=name,
            #         params=params)
            #     >> select([
            #         "kernel",
            #         "benchmark",
            #         "params",
            #         "pruning",
            #         "phase",
            #         "iteration",
            #         "cycles",
            #         "cost",
            #         "nodes",
            #         "timestamp"
            #     ])
            # )
            res.append(df)

    out = Path("figs") / "data" / "pruning.csv"
    df = (
        pd.concat(res)
        >> sort_values(by=["benchmark", "params", "pruning"], key=cmp_params)
        >> reset_index(drop=True)
        >> display()
        >> to_csv(out, index_label="iter")
    )


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
        cycles_csv = exp_path / "results" / "cycles.csv"

        if all(
            [
                # "Mar29-1443" in config["date"],
                # "Apr10-1252" in config["date"] or "Apr09-1813" in config["date"],
                "Apr12-1031" in config["date"],
                "key" in config and config["key"] == "performance",
            ]
        ):
            print(exp_path)
            ruleset = Path(config["metadata"]["rules.json"]).stem
            memory = pd.read_csv(
                exp_path / "memory.csv", header=None, names=["timestamp", "ram_used"]
            )
            memory = memory >> agg(["min", "max"])
            extraction_time = (pd.read_csv(exp_path / "data.csv")
                               >> filter_by(X.name == "extraction"))["value"].astype("float").sum()
            compile_time = (memory["timestamp"] >> agg(lambda x: x["max"] - x["min"]))
            ram_used = memory["ram_used"]["max"]

            egraph_cost = (pd.read_csv(exp_path / "data.csv")
                           >> filter_by(X.iteration == "report")
                           >> filter_by(X.name == "cost")
                           >> iloc([-1]))
            if len(egraph_cost["value"].values) > 0:
                egraph_cost = egraph_cost["value"].values[0]
            else:
                egraph_cost = -1

            timeout = json.load((exp_path / "compile.json").open("r"))["timeout"]

            if cycles_csv.exists():
                data = pd.read_csv(cycles_csv)
                if len(data["cycles"].values) > 0:
                    cycles = data["cycles"].values[0]
                    kernel = data["kernel"].values[0]
                else:
                    cycles = -1
                    kernel = "compgen"
            else:
                cycles = -1
                kernel = "compgen"

            df = pd.DataFrame(data={
                "kernel": [kernel],
                "benchmark": [name],
                "params": [params],
                "ruleset": [ruleset],
                "exp": [exp_path.name],
                "timeout": [timeout],
                "greedy": [config["metadata"]["alt_cost"]],
                "cycles": [cycles],
                "cost": [egraph_cost],
                "compile_time": [compile_time - extraction_time],
                "max_ram_used": [ram_used]
            })

            res.append(df)
            # print(df[df["kernel"] == "compgen"]["cycles"])

    if len(res) == 0:
        print("No matches!")
        return

    df = (
        pd.concat(res)
        >> sort_values(by=["benchmark", "params", "timeout"], key=cmp_params)
        >> reset_index(drop=True, names=["index"])
        >> display()
        >> to_csv(Path("figs") / "data" / "est_cycles.csv", index=False)
    )


def stock_dios():
    res = []
    for egg_kernel_csv in Path("diospyros-results").glob("**/egg-kernel.csv"):
        exp_dir = Path(egg_kernel_csv.parents[0])
        benchmark = egg_kernel_csv.parents[1].stem
        if benchmark != "q-prod":
            params = egg_kernel_csv.parents[0].stem.rsplit("_", 1)[0]
        else:
            params = "0"

        stats = json.load((exp_dir / "stats.json").open("r"))

        df = (pd.read_csv(egg_kernel_csv)
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
        res.append(df)

    df = (pd.concat(res)
          >> sort_values(by=["benchmark", "params"], key=cmp_params)
          >> reset_index(drop=True, names=["index"])
          >> display()
          >> to_csv(Path("figs") / "data" / "stock_cycles.csv", index=False))


def noeqsat():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all([
                "key" in config and config["key"] == "noeqsat"
        ]):
            if "_" in config["name"]:
                name, params = config["name"].split("_", 1)
            else:
                name = config["name"]
                params = "0"
            res.append(pd.read_csv(exp_path / "results" / "cycles.csv")
                       >> mutate(
                           benchmark=name,
                           params=params)
                       >> select(["benchmark", "params", "kernel", "cycles", "correct"]))

    (pd.concat(res)
     >> sort_values(by=["benchmark", "params"], key=cmp_params)
     >> reset_index(drop=True, names=["index"])
     >> display
     >> to_csv(Path("figs") / "data" / "noeqsat.csv", index=False))


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
            df = (
                pd.read_csv(exp_path / "data.csv")
                >> filter_by(X.name == "cost")
                >> filter_by(X.iteration != "report")
                >> mutate(benchmark=benchmark, params=params)
                >> select(["benchmark", "params", "iteration", "value"])
            )
            cost += [df]

    # write data to csv
    df = pd.DataFrame(runtime, columns=["benchmark", "runtime"])
    out = Path("figs") / "data" / "backoff_fail.csv"
    df.to_csv(out, index_label="index")
    print(f"Wrote {out}")

    # write cost data to csv
    df = pd.concat(cost) >> reset_index(drop=True)
    out = Path("figs") / "data" / "backoff_cost.csv"
    df.to_csv(out)
    print(f"Wrote {out}")


def fix():
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "key" in config and config["key"] == "fix":
            print(exp_path, config["metadata"]["alt_cost"])


def ruleset_ablation():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all([
                "key" in config and config["key"] == "ruleset_ablation",
                "Apr10-1245" in config["date"]
        ]):
            if "_" in config["name"]:
                name, params = config["name"].split("_", 1)
            else:
                name = config["name"]
                params = "0"

            # get timeout used for ruleset
            print(exp_path, config["date"])
            timeout = timeout = Path(config["metadata"]["rules.json"]).stem
            try:
                rules = json.load((exp_path / "rules.json").open("r"))
                if "abs_timeout" in rules["params"]:
                    timeout = rules["params"]["abs_timeout"]
            except Exception as e:
                print(f"{exp_path} broken! {e}")

            cycles = None
            if (exp_path / "results" / "cycles.csv").exists():
                cycles = pd.read_csv(exp_path / "results" / "cycles.csv")
                cycles = cycles["cycles"].values[0]

            try:
                df = (
                    pd.read_csv(exp_path / "data.csv")
                    >> filter_by(X.name == "cost", X.iteration == "report")
                    >> iloc([-1])
                    >> mutate(
                        benchmark=name,
                        params=params,
                        exp=exp_path.name,
                        ruleset=timeout,
                        cycles=cycles,
                    )
                    >> replace({"expanding_vecmac": -2, "original_dios_rules": -1})
                    >> spread(X.name, X.value)
                    >> display()
                    >> select(X.benchmark, X.params, X.exp, X.ruleset, X.cost, X.cycles)
                )
                res.append(df)
            except Exception:
                print(f"{exp_path} broken")

    out = Path("figs") / "data" / "ruleset_ablation.csv"
    df = (
        pd.concat(res)
        >> sort_values(by=["benchmark", "params", "ruleset"], key=cmp_params)
        >> reset_index(drop=True)
        >> to_csv(out, index_label="index")
    )


def instruction():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "_" in config["name"]:
            name, params = config["name"].split("_", 1)
        else:
            name = config["name"]
            params = "0"

        if all(["key" in config and config["key"] == "instruction"]):
            cycles_csv = exp_path / "results" / "cycles.csv"

            egraph_cost = None
            cycles = None
            if cycles_csv.exists():
                df = pd.read_csv(cycles_csv)
                cycles = df["cycles"].values[0]

            egraph_cost = (
                pd.read_csv(exp_path / "data.csv")
                >> filter_by(X.iteration == "report")
                >> filter_by(X.name == "cost")
                >> iloc([-1])
            )["value"].values[0]

            res.append(
                pd.DataFrame(data={
                    "benchmark": [name],
                    "params": [params],
                    "exp": [exp_path.name],
                    "rules": Path(config["metadata"]["rules.json"]).stem,
                    "cycles": [cycles],
                    "cost": [egraph_cost]
                })
            )
    out = Path("figs") / "data" / "instruction.csv"
    (pd.concat(res)
        >> sort_values(by=["benchmark", "params", "rules"], key=cmp_params)
        >> reset_index(drop=True)
        >> display()
        >> to_csv(out, index_label="index"))


def play():
    # _ = latest_date()

    # res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all(["metadata" in config and "backoff" in config["metadata"]["compile.json"]]):
            print(exp_path)
            print(json.dumps(config, indent=2))
    # df = pd.concat(res)
    # print(df.to_string())


def main():
    # exp_iter("2d-conv_3x3_3x3")
    pruning()
    # compile_est_cycles()
    # stock_dios()
    # scheduler()
    # play()
    # fix()
    # noeqsat()
    # ruleset_ablation()
    instruction()


if __name__ == "__main__":
    main()
