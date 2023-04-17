#!/usr/bin/env python3

import json
from datetime import datetime
from server import unique_name
from pathlib import Path
import os
import shutil
import itertools
from typing import Callable


def mat_mul(
    jobs_dir,
    a_rows,
    a_cols,
    b_rows,
    b_cols,
    ruleset,
    compile,
    costfn,
    key=None,
    timeout=60 * 30,
):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "timeout": timeout,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "costfn": costfn,
        },
    }
    params = {
        "A-rows": a_rows,
        "A-cols": a_cols,
        "B-rows": b_rows,
        "B-cols": b_cols,
        "reg-size": 4,
    }
    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")
    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin",
        "compile",
        "mat-mul",
        "--dios-bin",
        "$dios_bin",
        "--dios-example-bin",
        "$dios_example_bin",
        "--dios-params",
        "params.json",
        "--vector-width",
        "4",
        "--rules",
        "rules.json",
        "--config",
        "compile.json",
        "--output-dir",
        "results",
        "--costfn",
        costfn,
    ]

    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)
    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", " ".join(command)]))
    os.chmod(str(job_dir / "run.sh"), 0o777)


def make_2d_conv(
    jobs_dir,
    irows,
    icols,
    frows,
    fcols,
    ruleset,
    compile,
    costfn,
    key=None,
    timeout=60 * 30,
    memory_limit=220,
):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"2d-conv_{irows}x{icols}_{frows}x{fcols}"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": memory_limit,
        "timeout": timeout,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "costfn": costfn,
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {
        "input-rows": irows,
        "input-cols": icols,
        "filter-rows": frows,
        "filter-cols": fcols,
        "reg-size": 4,
    }
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin",
        "compile",
        "2d-conv",
        "--dios-bin",
        "$dios_bin",
        "--dios-example-bin",
        "$dios_example_bin",
        "--dios-params",
        "params.json",
        "--vector-width",
        "4",
        "--rules",
        "rules.json",
        "--config",
        "compile.json",
        "--output-dir",
        "results",
        "--costfn",
        costfn,
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", " ".join(command)]))
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def q_prod(jobs_dir, ruleset, compile, costfn, key=None, timeout=60 * 30):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = "q-prod"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "timeout": timeout,
        "command": "./run.sh",
        "benchmark": "q-prod",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "costfn": costfn,
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {"reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin",
        "compile",
        "q-prod",
        "--dios-bin",
        "$dios_bin",
        "--dios-example-bin",
        "$dios_example_bin",
        "--dios-params",
        "params.json",
        "--vector-width",
        "4",
        "--rules",
        "rules.json",
        "--config",
        "compile.json",
        "--output-dir",
        "results",
        "--costfn",
        costfn,
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", " ".join(command)]))
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def qr_decomp(jobs_dir, N, ruleset, compile, costfn, key=None, timeout=60 * 30):
    date_str = datetime.now().strftime("%b%d-%H%M")
    name = f"qr-decomp_{N}"
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)
    config = {
        "date": date_str,
        "name": name,
        "key": key,
        "memory_limit": 220,
        "timeout": timeout,
        "command": "./run.sh",
        "metadata": {
            "rules.json": str(ruleset),
            "compile.json": str(compile),
            "costfn": costfn,
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {"N": N, "reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    shutil.copy(compile, job_dir / "compile.json")

    command = [
        "RUST_LOG=debug,egg=info",
        "$compgen_bin",
        "compile",
        "qr-decomp",
        "--dios-bin",
        "$dios_bin",
        "--dios-example-bin",
        "$dios_example_bin",
        "--dios-params",
        "params.json",
        "--vector-width",
        "4",
        "--rules",
        "rules.json",
        "--config",
        "compile.json",
        "--output-dir",
        "results",
        "--costfn",
        costfn,
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", " ".join(command)]))
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def make_synthesis(
    jobs_dir,
    synth_timeout,
    eqsat_iter=3,
    eqsat_timeout=60,
    binops=None,
    triops=None,
    timeout=6000,
):
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(jobs_dir / f"{date_str}-synthesis-{synth_timeout}", 0)
    job_dir.mkdir(exist_ok=False)
    synth_config = json.load((Path("synthesis") / "base.json").open("r"))
    synth_config["ruler_config"]["abs_timeout"] = synth_timeout
    synth_config["ruler_config"]["eqsat_iter_limit"] = eqsat_iter
    synth_config["ruler_config"]["eqsat_time_limit"] = eqsat_timeout
    if binops is not None:
        synth_config["binops"] = binops
    if triops is not None:
        synth_config["triops"] = triops
    job_config = {
        "date": date_str,
        "name": "synthesis",
        "memory_limit": 220,
        "timeout": timeout,
        "command": "./run.sh",
        "metadata": {
            "timeout": synth_timeout,
            "eqsat_iter_limit": eqsat_iter,
            "eqsat_timeout": eqsat_timeout,
        },
    }
    json.dump(job_config, (job_dir / "config.json").open("w"), indent=2)
    json.dump(synth_config, (job_dir / "synth.json").open("w"), indent=2)

    command = [
        "RUST_LOG=debug,egg=info,z3=off",
        "$compgen_bin",
        "synth",
        "ruleset.json",
        "--config",
        "synth.json",
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", " ".join(command)]))
    # make the run script executable
    os.chmod(str(job_dir / "run.sh"), 0o777)


def dict_from_dir(path, pat="*.json", key=None):
    path = path.expanduser().resolve()
    assert path.exists()

    res = {}

    for p in path.glob(pat):
        k = p.stem
        if key is not None and isinstance(key, Callable):
            k = key(p)
        res[k] = p.expanduser().resolve()

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
        [2, 2, 2, 2],
        [2, 3, 3, 3],
        [3, 3, 3, 3],
        [4, 4, 4, 4],
        [8, 8, 8, 8],
        [10, 10, 10, 10],
        [16, 16, 16, 16],
        [18, 18, 18, 18],
        [20, 20, 20, 20],
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
        [18, 18, 2, 2],
        [18, 18, 3, 3],
        [18, 18, 4, 4],
    ]
    q_prod_params = [0]
    qr_decomp_sizes = [3, 4]
    ruleset = rulesets["ruleset_timeout86400"]
    cs = [
        configs["loop_alt_cost_t180"],
        # configs["loop_alt_cost_t1800"]
    ]

    # create all the jobs
    for size, c in itertools.product(mat_mul_sizes, cs):
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=json.load(c.open("r"))["timeout"] * 5,
        )

    for size, c in itertools.product(conv_2d_sizes, cs):
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=json.load(c.open("r"))["timeout"] * 5,
        )

    for _, c in itertools.product(q_prod_params, cs):
        q_prod(
            Path("jobs"),
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=json.load(c.open("r"))["timeout"] * 5,
        )

    for size, c in itertools.product(qr_decomp_sizes, cs):
        qr_decomp(
            Path("jobs"),
            size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=json.load(c.open("r"))["timeout"] * 5,
        )


def pruning_experiments():
    """
    This experiment is meant to show that pruning dramatically decreases
    the how long it takes to find a low-cost program.
    """

    print("Creating pruning experiments")

    params = [
        [3, 3, 3, 3],
        [4, 4, 3, 3],
        [8, 8, 3, 3],
        [10, 10, 3, 3],
        [16, 16, 3, 3],
    ]
    timeout = 360

    for p in params:
        # pruning config
        make_2d_conv(
            Path("jobs"),
            *p,
            rulesets["ruleset_timeout86400"],
            configs["loop_alt_cost_t360"],
            "alternative",
            key="pruning",
            memory_limit=100,
            timeout=timeout * 10,
        )
        # no pruning config
        make_2d_conv(
            Path("jobs"),
            *p,
            rulesets["ruleset_timeout86400"],
            configs["loop_alt_cost_noprune_t360"],
            "alternative",
            key="pruning",
            memory_limit=100,
            timeout=timeout * 10,
        )


def understand_cost_function():
    """
    1) Try looping config with more expansion iterations.
    2) Try looping+pruning with original cost function.
    """

    sizes = [
        [2, 2, 2, 2],
        [4, 4, 4, 4],
        [8, 8, 8, 8],
        [16, 16, 16, 16],
    ]

    for s in sizes:
        mat_mul(
            Path("jobs"),
            *s,
            rulesets["expanding_vecmac"],
            configs["loop_alt_cost_t1800"],
            "alternative",
            key="fix",
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
        mat_mul(Path("jobs"), *size, ruleset, config, "alternative", key="noeqsat")

    for size in conv_2d_sizes:
        make_2d_conv(Path("jobs"), *size, ruleset, config, "alternative", key="noeqsat")

    q_prod(Path("jobs"), ruleset, config, "alternative", key="noeqsat")


def ruleset_ablation():
    """
    Measure the performance difference between using different rulesets.
    """

    conv_2d_sizes = [
        [3, 3, 2, 2],
        [3, 3, 3, 3],
        [3, 5, 3, 3],
        [4, 4, 3, 3],
        [8, 8, 3, 3],
        # [10, 10, 2, 2],
        [10, 10, 3, 3],
        # [10, 10, 4, 4],
        # [16, 16, 2, 2],
        [16, 16, 3, 3],
        # [16, 16, 4, 4],
    ]

    config = configs["loop_alt_cost_t1800"]

    def read_time(p):
        config = json.load((p.parents[0] / "config.json").open("r"))
        return config["metadata"]["timeout"]

    rules = dict_from_dir(
        Path("completed") / "synthesis", pat="**/ruleset.json", key=read_time
    )
    rs = (
        list(map(lambda x: rules[x], [60, 600, 6000, 60000]))
        + [rulesets["ruleset_timeout43200"]]
        + [rulesets["ruleset_timeout86400"]]
        # + [rulesets["expanding_vecmac"]]
        # + [rulesets["original_dios_rules"]]
    )
    exps = itertools.product(conv_2d_sizes, rs)
    for (size, r) in exps:
        make_2d_conv(
            Path("jobs"), *size, r, config, "alternative", key="ruleset_ablation"
        )


def ruleset_synthesis():
    """
    Synthesize rulesets with different settings.
    """

    timeouts = [
        # 60,
        # 600,
        # 6000,
        # 60000,
        # 600000,
        60
        * 60
        * 24
        * 3
    ]

    eqsat_settings = [
        # (2, 60),
        (3, 60),
        # (4, 120),
    ]
    exps = itertools.product(timeouts, eqsat_settings)
    for (t, eqsat) in exps:
        make_synthesis(Path("jobs"), t, eqsat_iter=eqsat[0], eqsat_timeout=eqsat[1])


def scheduler():
    """
    Make a job that uses all the rules with the backoff scheduler.
    """

    size = [[8, 8, 3, 3]]

    for s in size:
        make_2d_conv(
            Path("jobs"),
            *s,
            rulesets["ruleset_timeout86400"],
            configs["all-backoff"],
            "alternative",
            key="scheduler",
        )
        make_2d_conv(
            Path("jobs"),
            *s,
            rulesets["ruleset_timeout86400"],
            configs["all-simple"],
            "alternative",
            key="scheduler",
        )


def add_instruction_ruleset():
    """
    We make these claims that it's trivial to change the spec to explore the
    design space of different instructions. Test this by generating rulesets
    with no fused ops, with just MULS, and with both MAC, MULS.
    """

    binops = ["/", "+", "*", "-"]
    # baseline + muls + mulsgn
    make_synthesis(
        Path("jobs"), 60000, binops=binops + ["~*"], triops=["mac"], timeout=6000000
    )
    # baseline + muls
    # make_synthesis(Path("jobs"), 60000, binops=binops, triops=["mac", "muls"], timeout=6000000)
    # baseline + mulsgn
    # make_synthesis(Path("jobs"), 60000, binops=binops + ["~*"], triops=["mac"], timeout=6000000)


def test_instruction_ruleset():
    """
    Run q-prod, qr-decomp with no fused ops, with muls, and with both fused ops
    """
    global rulesets

    rulesets_dir = Path("instruction-rulesets")
    base_path = rulesets["ruleset_timeout86400"]
    mulsgn_path = Path("completed") / "synthesis" / "17" / "ruleset.json"
    muls_path = Path("completed") / "synthesis" / "10" / "ruleset.json"
    base_ruleset = json.load(base_path.open("r"))
    mulsgn_ruleset = json.load(mulsgn_path.open("r"))
    muls_ruleset = json.load(muls_path.open("r"))

    base_eqs = base_ruleset["eqs"] + [
        {
            "lhs": "(sqrt 1)",
            "rhs": "1",
            "bidirectional": True
        },
        {
            "lhs": "(sqrt 0)",
            "rhs": "0",
            "bidirectional": True
        },
        {
            "lhs": "(sgn 0)",
            "rhs": "0",
            "bidirectional": True
        },
        {
            "lhs": "(sgn 1)",
            "rhs": "1",
            "bidirectional": True
        },
        {
            "lhs": "(VecMinus (Vec ?a) (Vec ?b))",
            "rhs": "(Vec (- ?a ?b))",
            "bidirectional": True
        },
        {
            "lhs": "(VecSgn (Vec ?a))",
            "rhs": "(Vec (sgn ?a))",
            "bidirectional": True
        },
        {
            "lhs": "(* 0 1)",
            "rhs": "0",
            "bidirectional": True
        },
        {
            "lhs": "(* ?a ?a)",
            "rhs": "(sqr ?a)",
            "bidirectional": True
        },
        {
            "lhs": "(VecMul ?a ?a)",
            "rhs": "(VecSqr ?a)",
            "bidirectional": True
        }
    ]

    def includes(ruleset, ops):
        return list(
            filter(lambda r: all([op in r["lhs"] + r["rhs"] for op in ops]), ruleset)
        )

    json.dump(
        {"params": {}, "eqs": base_eqs},
        (rulesets_dir / "base.json").open("w"),
        indent=2,
    )
    json.dump(
        {
            "params": {},
            "eqs": base_eqs + includes(mulsgn_ruleset["eqs"], ["MulSgn"]),
        },
        (rulesets_dir / "mulsgn.json").open("w"),
        indent=2,
    )
    json.dump(
        {
            "params": {},
            "eqs": base_eqs + includes(muls_ruleset["eqs"], ["VecMULS"]),
        },
        (rulesets_dir / "muls.json").open("w"),
        indent=2,
    )
    json.dump(
        {
            "params": {},
            "eqs": base_eqs
            + includes(muls_ruleset["eqs"], ["VecMULS"])
            + includes(mulsgn_ruleset["eqs"], ["MulSgn"]),
        },
        (rulesets_dir / "mulsgn_muls.json").open("w"),
        indent=2,
    )

    rulesets = dict_from_dir(Path("instruction-rulesets"))

    # for (n, r) in rulesets.items():
    qr_decomp(
        Path("jobs"),
        3,
        rulesets["base"],
        configs["loop_alt_cost_t360"],
        "alternative",
        key="instruction",
        timeout=7200,
    )


def overview_example():
    make_2d_conv(
        Path("jobs"),
        *[2, 2, 2, 2],
        rulesets["ruleset_timeout86400"],
        configs["all-simple"],
        "alternative",
        key="overview",
    )


def main():
    # overall_performance()
    # pruning_experiments()
    # understand_cost_function()
    # no_eqsat()
    # ruleset_ablation()
    # ruleset_synthesis()
    # scheduler()
    # add_instruction_ruleset()
    test_instruction_ruleset()
    # overview_example()


if __name__ == "__main__":
    main()
