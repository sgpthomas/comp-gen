from pathlib import Path
import json
import pandas as pd
from dfply import filter_by, X, dfpipe, select, mutate


# def cmp_params(a):
#     return list(map(int, a.replace("_", "x").split("x")))
# res = sorted(res, key=lambda a: (cmp_params(a[1]), a[2]))
# for r in res:
#     print(r)
# out = Path("..") / "experiments" / "cycle-estimation" / "2d-conv-3x3_3x3_iter.csv"
# with out.open("w+") as f:
#     f.write("index,costfn,cost,nodes\n")
#     for row in res:
#         f.write(",".join(row))
#         f.write("\n")

@dfpipe
def pivot_table(df, **kwargs):
    df = pd.pivot_table(df, **kwargs)
    return df.reset_index(drop=True, names=["index"])


def exp_iter(name, date=None):
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all([
                "Mar20" in config["date"],
                "expanding" in config["metadata"]["rules.json"],
                name in config["name"]
        ]):
            df = (
                pd.read_csv(exp_path / "data.csv")
                >> filter_by((X.name == "nodes") | (X.name == "cost"))
                >> filter_by(X.iteration != "report")
                >> pivot_table(
                    index=["phase", "iteration"],
                    columns=["name"],
                    values="value",
                    sort=False
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


def compile_times():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all([
                "Mar20" in config["date"],
                "expanding" in config["metadata"]["rules.json"],
        ]):
            memory = pd.read_csv(
                exp_path / "memory.csv",
                header=None,
                names=["timestamp", "ram_used"]
            )
            df = memory.agg(['min', 'max'])
            max_ts = df.loc[['max']]['timestamp'].values[0]
            min_ts = df.loc[['min']]['timestamp'].values[0]
            killed = "killed" in list(memory["ram_used"].values)
            res.append([
                config["name"],
                "pruning" if config["metadata"]["alt_cost"] else "stock",
                max_ts - min_ts,
                killed
            ])

    out = Path("figs") / "data" / "compile_times.csv"
    df = pd.DataFrame(res, columns=["benchmark", "type", "runtime", "killed"])
    df.to_csv(out, index_label="index")
    print(df)
    print(f"Wrote {out}")


def scheduler():
    res = []
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if "metadata" not in config:
            continue

        if all([
                "Mar23" in config["date"]
        ]):
            memory = pd.read_csv(
                exp_path / "memory.csv",
                header=None,
                names=["timestamp", "ram_used"]
            )
            df = memory.agg(['min', 'max'])
            max_ts = df.loc[['max']]['timestamp'].values[0]
            min_ts = df.loc[['min']]['timestamp'].values[0]
            res.append([config["name"], max_ts - min_ts])

    # write data to csv
    df = pd.DataFrame(res, columns=["benchmark", "runtime"])
    out = Path("figs") / "data" / "backoff_fail.csv"
    df.to_csv(out, index_label="index")
    print(f"Wrote {out}")


def play():
    for config_path in Path("completed").glob("**/config.json"):
        exp_path = Path(config_path.parents[0])
        config = json.load(config_path.open("r"))

        if all([
                "Mar20" in config["date"]
        ]):
            print(exp_path)


def main():
    # exp_iter("2d-conv_3x3_3x3")
    # compile_times()
    # scheduler()
    play()


# if __name__ == "__main__":
main()
