#!/usr/bin/env python3

import json
from datetime import datetime
from server import unique_name
from pathlib import Path
import os
import shutil
import itertools


def mat_mul(jobs_dir, a_rows, a_cols, b_rows, b_cols, ruleset, compile, alt_cost, key=None):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}"
    job_dir = unique_name(
        jobs_dir / f"{date_str}-{name}",
        0
    )
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "alt_cost": alt_cost
        }
    }
    params = {
        "A-rows": a_rows,
        "A-cols": a_cols,
        "B-rows": b_rows,
        "B-cols": b_cols,
        "reg-size": 4
    }
    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")
    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin", "compile", "mat-mul",
        "--dios-bin", "$dios_bin",
        "--dios-example-bin", "$dios_example_bin",
        "--dios-params", "params.json",
        "--vector-width", "4",
        "--rules", "rules.json",
        "--config", "compile.json",
        "--output-dir", "results"
    ]

    if alt_cost:
        command += ["--alt-cost"]

    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)
    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    os.chmod(str(job_dir / "run.sh"), 0o777)


def make_2d_conv(jobs_dir, irows, icols, frows, fcols, ruleset, compile, alt_cost, key=None):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"2d-conv_{irows}x{icols}_{frows}x{fcols}"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "alt_cost": alt_cost
        }
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {
        "input-rows": irows,
        "input-cols": icols,
        "filter-rows": frows,
        "filter-cols": fcols,
        "reg-size": 4
    }
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin", "compile", "2d-conv",
        "--dios-bin", "$dios_bin",
        "--dios-example-bin", "$dios_example_bin",
        "--dios-params", "params.json",
        "--vector-width", "4",
        "--rules", "rules.json",
        "--config", "compile.json",
        "--output-dir", "results"
    ]

    if alt_cost:
        command += ["--alt-cost"]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def q_prod(jobs_dir, ruleset, compile, alt_cost, key=None):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = "q-prod"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "command": "./run.sh",
        "benchmark": "q-prod",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "alt_cost": alt_cost
        }
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {
        "reg-size": 4
    }
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin", "compile", "q-prod",
        "--dios-bin", "$dios_bin",
        "--dios-example-bin", "$dios_example_bin",
        "--dios-params", "params.json",
        "--vector-width", "4",
        "--rules", "rules.json",
        "--config", "compile.json",
        "--output-dir", "results"
    ]

    if alt_cost:
        command += ["--alt-cost"]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def qr_decomp(jobs_dir, N, ruleset, compile, alt_cost, key=None):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"qr-decomp_{N}"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "alt_cost": alt_cost
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {
        "N": N,
        "reg-size": 4
    }
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin", "compile", "qr-decomp",
        "--dios-bin", "$dios_bin",
        "--dios-example-bin", "$dios_example_bin",
        "--dios-params", "params.json",
        "--vector-width", "4",
        "--rules", "rules.json",
        "--config", "compile.json",
        "--output-dir", "results"
    ]

    if alt_cost:
        command += ["--alt-cost"]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def make_synthesis(jobs_dir, timeout):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(jobs_dir / f"{date_str}-synthesis-{timeout}", 0)
    job_dir.mkdir(exist_ok=False)
    synth_config = json.load((Path("synthesis") / "base.json").open("r"))
    synth_config["ruler_config"]["abs_timeout"] = timeout
    job_config = {
        "date": date_str,
        "name": "synthesis",
        "memory_limit": 220,
        "command": "./run.sh",
        "metadata": {
            "timeout": timeout
        }
    }
    json.dump(job_config, (job_dir / "config.json").open("w"), indent=2)
    json.dump(synth_config, (job_dir / "synth.json").open("w"), indent=2)

    command = [
        "RUST_LOG=debug,egg=info,z3=none",
        "$compgen_bin", "synth", "ruleset.json",
        "--config", "synth.json"
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines(
            "\n".join([
                "#!/usr/bin/env bash",
                "",
                " ".join(command)
            ])
        )
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def dict_from_dir(path, ext="json"):
    path = path.expanduser().resolve()
    assert path.exists()

    res = {}

    for p in path.glob(f"*.{ext}"):
        res[p.stem] = p.expanduser().resolve()

    return res


rulesets = dict_from_dir(Path("../experiments/rulesets"))
configs = dict_from_dir(Path("../experiments/configs"))
# rulesets = {
#     "expanding": "../experiments/rulesets/expanding.json",
#     "expanding_vecmac": "../experiments/rulesets/expanding_vecmac.json",
#     "ruler": "../experiments/rulesets/ruleset_timeout432000.json",
#     "t2": "~/Research/diospyros/t2.json"
# }

# # resolve the ruleset paths
# for key, val in rulesets.items():
#     rulesets[key] = Path(val).expanduser().resolve()

# configs = {
#     "wack": "../experiments/configs/wack.json",
#     "phased": "../experiments/configs/phased.json",
#     "phased_no_opt": "../experiments/configs/phased_no_opt.json",
#     "phased_no_opt_alt_cost": "../experiments/configs/phased_no_opt_alt_cost.json",
#     "loop_alt_cost": "../experiments/configs/loop_alt_cost.json",
#     "loop_alt_cost_t180": "../experiments/configs/loop_alt_cost_t180.json",
#     "loop_alt_cost_t1800": "../experiments/configs/loop_alt_cost_t1800.json",
#     "loop_no_opt_alt_cost_t1800": "../experiments/configs/loop_no_opt_alt_cost_t1800.json",
#     "all-simple": "../experiments/configs/all-simple.json",
#     "all-backoff": "../experiments/configs/all-backoff.json",
#     "loop-more-expansion": "../experiments/configs/loop_more_expansion.json",
#     "loop-dios-cost": "../experiments/configs/loop_dios_cost.json",
#     "loop_more_compilation": "../experiments/configs/loop_more_compilation.json",
#     "none": "../experiments/configs/none.json",
# }
# resolve the ruleset paths
# for key, val in configs.items():
#     configs[key] = Path(val).expanduser().resolve()


def overall_performance():
    """
    This measures the overall performance of the compiler in terms of
    1) How good are the results of the compiler?
    2) How fast is the compiler?
    3) How well does the compiler scale?

    This is a run with all of the benchmarks that Dios uses.
    """

    print("Creating overall performance jobs")

    mat_mul_sizes = [
        # [2, 2, 2, 2],
        # [2, 3, 3, 3],
        # [3, 3, 3, 3],
        # [4, 4, 4, 4],
        # [8, 8, 8, 8],
        # [10, 10, 10, 10],
        # [16, 16, 16, 16],
        [20, 20, 20, 20]
    ]
    conv_2d_sizes = [
        # [3, 3, 2, 2],
        # [3, 3, 3, 3],
        # [3, 5, 3, 3],
        # [4, 4, 3, 3],
        # [8, 8, 3, 3],
        # [10, 10, 2, 2],
        # [10, 10, 3, 3],
        # [10, 10, 4, 4],
        # [16, 16, 2, 2],
        # [16, 16, 3, 3],
        # [16, 16, 4, 4],
        [20, 20, 3, 3],
    ]
    # qr_decomp_sizes = [
    #     3,
    #     4
    # ]
    ruleset = rulesets["expanding_vecmac"]
    config = configs["loop_alt_cost_t1800"]

    # create all the jobs
    for size in mat_mul_sizes:
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            config,
            True,
            key="performance"
        )

    for size in conv_2d_sizes:
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            config,
            True,
            key="performance"
        )

    # q_prod(Path("jobs"), ruleset, config, True, key="performance")

    # for size in qr_decomp_sizes:
    #     qr_decomp(
    #         Path("jobs"),
    #         size,
    #         ruleset,
    #         config,
    #         True,
    #         key="performance"
    #     )


def pruning_experiment():
    """
    This experiment is meant to show that pruning dramatically decreases
    the how long it takes to find a low-cost program.
    """

    print("Creating pruning experiments")

    params = [
        [3, 3, 2, 2],
    ]

    for p in params:
        # pruning config
        make_2d_conv(
            Path("jobs"),
            *p,
            rulesets["ruleset_timeout432000"],
            configs["loop_alt_cost_t1800"],
            True,
            key="pruning"
        )
        # no pruning config
        # make_2d_conv(
        #     Path("jobs"),
        #     *p,
        #     rulesets["ruleset_timeout432000"],
        #     configs["phased_no_opt_alt_cost"],
        #     True,
        #     key="pruning"
        # )


def understand_cost_function():
    """
    1) Try looping config with more expansion iterations.
    2) Try looping+pruning with original cost function.
    """

    params = [
        [10, 10, 2, 2],
    ]

    for p in params:
        # make_2d_conv(
        #     Path("jobs"),
        #     *p,
        #     rulesets["ruleset_timeout432000"],
        #     configs["loop_dios_cost"],
        #     False,
        #     key="fix"
        # )
        # make_2d_conv(
        #     Path("jobs"),
        #     *p,
        #     rulesets["ruleset_timeout432000"],
        #     configs["loop_more_expansion"],
        #     True,
        #     key="fix"
        # )
        make_2d_conv(
            Path("jobs"),
            *p,
            rulesets["ruleset_timeout432000"],
            configs["loop_alt_cost_t1800"],
            True,
            key="fix"
        )


def no_eqsat():
    """
    Get estimation numbers for all benchmarks with no equality saturation.
    In other words, what is the effect of doing just the Diospyros symbolic execution.
    """

    mat_mul_sizes = [
        [2, 2, 2, 2],
        [2, 3, 3, 3],
        [3, 3, 3, 3],
        [4, 4, 4, 4],
        [8, 8, 8, 8],
        [10, 10, 10, 10],
        [16, 16, 16, 16],
    ]
    conv_2d_sizes = [
        [3, 3, 2, 2],
        [3, 3, 3, 3],
        [3, 5, 3, 3],
        [4, 4, 3, 3],
        [8, 8, 3, 3],
        [10, 10, 2, 2],
        [10, 10, 3, 3],
        [10, 10, 4, 4],
        [16, 16, 2, 2],
        [16, 16, 3, 3],
        [16, 16, 4, 4],
    ]

    ruleset = rulesets["expanding_vecmac"]
    config = configs["none"]

    # create all the jobs
    for size in mat_mul_sizes:
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            config,
            True,
            key="noeqsat"
        )

    for size in conv_2d_sizes:
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            config,
            True,
            key="noeqsat"
        )

    q_prod(Path("jobs"), ruleset, config, True, key="noeqsat")


def ruleset_ablation():
    """
    Measure the performance difference between using different rulesets.
    """

    conv_2d_sizes = [
        # [3, 3, 2, 2],
        # [3, 3, 3, 3],
        # [3, 5, 3, 3],
        [4, 4, 3, 3],
        [8, 8, 3, 3],
        [10, 10, 2, 2],
        # [10, 10, 3, 3],
        [10, 10, 4, 4],
        # [16, 16, 2, 2],
        # [16, 16, 3, 3],
        # [16, 16, 4, 4],
    ]

    config = configs["loop_alt_cost_t180"]
    rs = [
        "expanding",
        "expanding_vecmac",
        "ruleset_timeout4000",
        "ruleset_timeout8000",
        "ruleset_timeout86400",
        "ruleset_timeout432000",
        "original_dios_rules",
    ]

    exps = itertools.product(conv_2d_sizes, rs)
    for (size, r) in exps:
        make_2d_conv(
            Path("jobs"),
            *size,
            rulesets[r],
            config,
            True,
            key="ruleset_ablation"
        )


def ruleset_synthesis():
    """
    Synthesize rulesets with different settings.
    """

    timeouts = [
        60,
        600,
        6000,
        60000
    ]

    for t in timeouts:
        make_synthesis(Path("jobs"), t)


def main():
    # overall_performance()
    # pruning_experiment()
    # understand_cost_function()
    # no_eqsat()
    # ruleset_ablation()
    ruleset_synthesis()


if __name__ == "__main__":
    main()
