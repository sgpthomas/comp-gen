from pathlib import Path
import json

parent_dir = Path("completed")
for exp_path in parent_dir.glob("**/stderr.log"):
    try:
        exp_path = Path(exp_path.parents[0])
        config = json.load((exp_path / "config.json").open("r"))
        if all([
                # "Mar09" in config["date"],
                # config["name"] == "2d-conv_8x8_3x3",
                config["metadata"]["alt_cost"],
                "expanding" in config["metadata"]["rules.json"],
                "compile_alt_cost" in config["metadata"]["compile.json"]
        ]):
            print(exp_path, config["name"])
            print(json.dumps(config, indent=2))
    except Exception:
        pass
