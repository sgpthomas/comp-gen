from pathlib import Path
import json
import pandas as pd

parent_dir = Path("completed")
res = []
for exp_path in parent_dir.glob("**/stderr.log"):
    try:
        exp_path = Path(exp_path.parents[0])
        config = json.load((exp_path / "config.json").open("r"))
        if all(
            [
                "Mar22" in config["date"],
                # "2d-conv_3x3_3x3" in config["name"],
                # "expanding" in config["metadata"]["rules.json"],
                # config["metadata"]["alt_cost"]
            ]
        ):
            print(exp_path, json.dumps(config, indent=2))
            df = pd.read_csv(exp_path / "data.csv")
            print(df)
            # print(df)
            # cost = df[
            #     (df["phase"] == "opt")
            #     & (df["iteration"] == "report")
            #     & (df["name"] == "cost")
            # ]["value"].values[0]
            # if config["metadata"]["alt_cost"]:
            #     res.append([
            #         "2d-conv",
            #         config["name"].split("_", 1)[1],
            #         "greedy",
            #         cost,
            #         config["metadata"]["compile.json"].split("/")[-1]
            #     ])
            # else:
            #     res.append([
            #         "2d-conv",
            #         config["name"].split("_", 1)[1],
            #         "diospyros",
            #         cost,
            #         config["metadata"]["compile.json"].split("/")[-1]
            #     ])
            # nodes = df[
            #     (df["name"] == "nodes") &
            #     (df["iteration"] != "report")
            # ]
            # cost = df[
            #     (df["name"] == "cost") &
            #     (df["iteration"] != "report")
            # ]
            # name = "greedy" if config["metadata"]["alt_cost"] else "normal"
            # res += list(map(
            #     lambda x: [str(x[0]), name, str(x[1][0]), str(x[1][1])],
            #     list(
            #         enumerate(
            #             zip(
            #                 cost["value"].values,
            #                 nodes["value"].values
            #             )
            #         )
            #     )
            # ))
            # if config["metadata"]["alt_cost"]:
            #     res.append([
            #         "2d-conv",
            #         config["name"].split("_", 1)[1],
            #         cost
            #     ])
    except Exception:
        pass


def cmp_params(a):
    return list(map(int, a.replace("_", "x").split("x")))


# res = sorted(res, key=lambda a: (cmp_params(a[1]), a[2]))
for r in res:
    print(r)
# out = Path("..") / "experiments" / "cycle-estimation" / "2d-conv-3x3_3x3_iter.csv"
# with out.open("w+") as f:
#     f.write("index,costfn,cost,nodes\n")
#     for row in res:
#         f.write(",".join(row))
#         f.write("\n")
