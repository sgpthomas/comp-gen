#!/usr/bin/env python3

import itertools
import json
import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Callable

import click

JOBS = {}


def unique_name(path: Path, suffix: int = 0) -> Path:
    """Find a unique path based on `path` and adding a suffix."""

    new_path = Path(path.parent) / f"{path.name}-{suffix}"
    if not new_path.exists():
        return new_path
    else:
        return unique_name(path, suffix + 1)


def job(name=None):
    def inner(func):
        job_name = name if name is not None else func.__name__

        def f(*args):
            print(f"Creating {job_name} jobs")
            func(*args)

        JOBS[job_name] = f

    return inner


def create_compile_config(compile, dest):
    if isinstance(compile, Path):
        shutil.copy(compile, dest)
    elif isinstance(compile, dict):
        json.dump(compile, dest.open("w"), indent=2)
    else:
        raise Exception(f"Unsupported `compile` type! {type(compile)}")


def compile_name(compile):
    if isinstance(compile, Path):
        return str(compile)
    elif isinstance(compile, dict):
        return f"config_a{compile['alpha']}_b{compile['beta']}.json"
    else:
        raise Exception(f"Unsupported `compile` type! {type(compile)}")


def make_datestr() -> str:
    return datetime.now().strftime("%b%d-%H%M")


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
            "compile.json": compile_name(compile),
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
    create_compile_config(compile, job_dir / "compile.json")

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
            "compile.json": compile_name(compile),
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
    create_compile_config(compile, job_dir / "compile.json")

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
            "compile.json": compile_name(compile),
            "costfn": costfn,
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {"reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    create_compile_config(compile, job_dir / "compile.json")

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
            "compile.json": compile_name(compile),
            "costfn": costfn,
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    params = {"N": N, "reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    create_compile_config(compile, job_dir / "compile.json")

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


def make_config(alpha=15, beta=6, timeout=180):
    return {
        "total_node_limit": 2000000000,
        "total_iter_limit": 4000,
        "timeout": timeout,
        "dry_run": False,
        "dump_rules": True,
        "debug": False,
        "reuse_egraphs": True,
        "cd_filter": None,
        "require_all_vars": False,
        "scheduler": "simple",
        "alpha": alpha,
        "beta": beta,
        "phase": {
            "phases": [
                {
                    "phases": [
                        {
                            "name": "pre-compile",
                            "cd": [None, alpha],
                            "ca": [beta, None],
                            "node_limit": 500000,
                            "timeout": 30,
                            "iter_limit": 2,
                            "fresh_egraph": True,
                            "disabled": False,
                        },
                        {
                            "name": "compile",
                            "cd": [alpha, None],
                            "ca": [None, None],
                            "timeout": 30,
                            "iter_limit": 2,
                            "disabled": False,
                        },
                        {
                            "name": "litvec",
                            "cd": [0.39, 0.41],
                            "ca": [0.20, 0.22],
                            "timeout": 30,
                            "iter_limit": 2,
                            "disabled": False,
                        },
                    ],
                    "loops": 20,
                },
                {
                    "name": "opt",
                    "cd": [None, alpha],
                    "ca": [None, beta],
                    "fresh_egraph": True,
                    "iter_limit": 10,
                    "scheduler": "simple",
                    "disabled": False,
                },
            ]
        },
    }


@job()
def overall_performance():
    """
    This measures the overall performance of the compiler in terms of
    1) How good are the results of the compiler?
    2) How fast is the compiler?
    3) How well does the compiler scale?

    This is a run with all of the benchmarks that Dios uses.
    """

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
    cs = [make_config(alpha=15, beta=6, timeout=180)]

    # create all the jobs
    for size, c in itertools.product(mat_mul_sizes, cs):
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=c["timeout"] * 20,
        )

    for size, c in itertools.product(conv_2d_sizes, cs):
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=c["timeout"] * 20,
        )

    for _, c in itertools.product(q_prod_params, cs):
        q_prod(
            Path("jobs"),
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=c["timeout"] * 20,
        )

    for size, c in itertools.product(qr_decomp_sizes, cs):
        qr_decomp(
            Path("jobs"),
            size,
            ruleset,
            c,
            "alternative",
            key="performance",
            timeout=c["timeout"] * 20,
        )


@job()
def diospyros():
    """
    Run diospyros experiments.
    """

    datestr = make_datestr()

    job_dir = unique_name(Path("jobs") / f"{datestr}-diospyros", 0)

    config = {
        "date": datestr,
        "name": "diospyros",
        "memory_limit": 220,
        "command": "./run.sh",
        "key": "diospyros",
        "metadata": {"timeout": 180},
    }

    run_sh = "\n".join(
        [
            "#!/usr/bin/env bash",
            "",
            'export PATH="/root/.cargo/bin:/root/racket/bin:$PATH"',
            "root_dir=$(pwd)",
            "git clone https://github.com/cucapra/diospyros.git",
            "cd diospyros",
            "make",
            " ".join(
                [
                    "python3",
                    "evaluation/eval_benchmarks.py",
                    "--timeout 180",
                    "--skiprun",
                    '-o "$root_dir/results"',
                ]
            ),
            "cd $root_dir",
            "rm -rf diospyros",
        ]
    )

    job_dir.mkdir(exist_ok=False)
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)
    with (job_dir / "run.sh").open("w") as f:
        f.writelines(run_sh)
    os.chmod(str(job_dir / "run.sh"), 0o777)


@job()
def pruning_experiments():
    """
    This experiment is meant to show that pruning dramatically decreases
    the how long it takes to find a low-cost program.
    """

    print("Creating pruning experiments")

    params = [
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
            timeout=timeout * 5,
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
            timeout=timeout * 5,
        )


@job()
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


@job()
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


@job()
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

    config = configs["loop_alt_cost_t1800"]

    def read_time(p):
        config = json.load((p.parents[0] / "config.json").open("r"))
        return config["metadata"]["timeout"]

    rules = dict_from_dir(
        Path("completed") / "synthesis", pat="**/ruleset.json", key=read_time
    )
    rs = list(map(lambda x: rules[x], [60, 600, 6000, 60000]))

    exps = itertools.product(conv_2d_sizes, rs)
    for size, r in exps:
        make_2d_conv(
            Path("jobs"), *size, r, config, "alternative", key="ruleset_ablation"
        )


@job()
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
    for t, eqsat in exps:
        make_synthesis(Path("jobs"), t, eqsat_iter=eqsat[0], eqsat_timeout=eqsat[1])


@job()
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


@job()
def add_instruction_ruleset():
    """
    We make these claims that it's trivial to change the spec to explore the
    design space of different instructions. Test this by generating rulesets
    with no fused ops, with just MULS, and with both MAC, MULS.
    """

    binops = ["/", "+", "*", "-"]
    # baseline + muls + mulsgn
    make_synthesis(
        Path("jobs"),
        60000,
        binops=binops + ["sqrtsgn"],
        triops=["mac", "muls"],
        timeout=6000000,
    )
    # baseline + muls
    # make_synthesis(Path("jobs"), 60000, binops=binops, triops=["mac", "muls"], timeout=6000000)
    # baseline + mulsgn
    # make_synthesis(Path("jobs"), 60000, binops=binops + ["~*"], triops=["mac"], timeout=6000000)


@job()
def test_instruction_ruleset():
    """
    Run q-prod, qr-decomp with no fused ops, with muls, and with both fused ops
    """
    global rulesets

    rulesets_dir = Path("instruction-rulesets")
    base_path = rulesets["ruleset_timeout86400"]

    muls_path = Path("completed") / "synthesis" / "10" / "ruleset.json"
    base_ruleset = json.load(base_path.open("r"))
    muls_ruleset = json.load(muls_path.open("r"))

    extra = [
        {
            "lhs": "(* (sqrt ?a) (sgn (neg ?b)))",
            "rhs": "(sqrtsgn ?a ?b)",
            "bidirectional": True,
        },
        {
            "lhs": "(Vec (sqrtsgn ?a ?b))",
            "rhs": "(VecSqrtSgn (Vec ?a) (Vec ?b))",
            "bidirectional": True,
        },
    ]
    base_eqs = base_ruleset["eqs"]

    def includes(ruleset, ops):
        return list(
            filter(lambda r: all([op in r["lhs"] + r["rhs"] for op in ops]), ruleset)
        )

    subprocess.run("rm instruction-rulesets/*", shell=True)

    json.dump(
        {"params": {}, "eqs": base_eqs},
        (rulesets_dir / "base.json").open("w"),
        indent=2,
    )
    json.dump(
        {
            "params": {},
            "eqs": base_eqs + extra,
        },
        (rulesets_dir / "sqrtsgn.json").open("w"),
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
            "eqs": base_eqs + includes(muls_ruleset["eqs"], ["VecMULS"]) + extra,
        },
        (rulesets_dir / "sqrtsgn_muls.json").open("w"),
        indent=2,
    )

    rulesets = dict_from_dir(Path("instruction-rulesets"))

    for n, r in rulesets.items():
        qr_decomp(
            Path("jobs"),
            3,
            r,
            configs["loop_alt_cost_t360"],
            "alternative",
            key="instruction",
            timeout=7200,
        )


@job()
def overview_example():
    make_2d_conv(
        Path("jobs"),
        *[2, 2, 2, 2],
        rulesets["ruleset_timeout86400"],
        configs["all-backoff"],
        "alternative",
        key="overview",
    )


@job()
def optimization_effect():
    """
    What is the overall effect of the optimization phase?

    Test this by running a pass with compilation loop timeout of 180, and 180
    seconds for optimization.
    """

    print("Creating overall performance jobs")

    mat_mul_sizes = [
        # [2, 2, 2, 2],
        # [2, 3, 3, 3],
        # [3, 3, 3, 3],
        [4, 4, 4, 4],
        [8, 8, 8, 8],
        [10, 10, 10, 10],
        [16, 16, 16, 16],
        # [18, 18, 18, 18],
        [20, 20, 20, 20],
    ]
    conv_2d_sizes = [
        # [3, 3, 2, 2],
        # [3, 3, 3, 3],
        # [3, 5, 3, 3],
        [4, 4, 3, 3],
        [8, 8, 3, 3],
        [10, 10, 2, 2],
        # [10, 10, 3, 3],
        [10, 10, 4, 4],
        [16, 16, 2, 2],
        # [16, 16, 3, 3],
        [16, 16, 4, 4],
        # [18, 18, 2, 2],
        [18, 18, 3, 3],
        # [18, 18, 4, 4],
    ]
    # q_prod_params = [0]
    qr_decomp_sizes = [
        3,
        # 4
    ]
    ruleset = rulesets["ruleset_timeout86400"]
    cs = [
        configs["loop_alt_cost_t180_w_opt"],
        configs["loop_alt_cost_t180_n_opt"],
    ]

    # create all the jobs
    for size, c in itertools.product(mat_mul_sizes, cs):
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="optimization",
            timeout=json.load(c.open("r"))["timeout"] * 3,
        )

    for size, c in itertools.product(conv_2d_sizes, cs):
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="optimization",
            timeout=json.load(c.open("r"))["timeout"] * 3,
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


@job()
def large_kernels():
    print("Creating large kernel jobs")

    mat_mul_sizes = [
        # [25, 25, 25, 25],
        # [30, 30, 30, 30],
        # [40, 40, 40, 40],
    ]
    conv_2d_sizes = [
        # [20, 20, 5, 5],
        # [30, 30, 5, 5],
        [32, 32, 5, 5],
        [34, 34, 5, 5],
        [36, 36, 5, 5],
        [38, 38, 5, 5],
        # [40, 40, 5, 5],
    ]
    ruleset = rulesets["ruleset_timeout86400"]
    cs = [
        configs["loop_alt_cost_t18000"],
    ]

    # create all the jobs
    for size, c in itertools.product(mat_mul_sizes, cs):
        mat_mul(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="large",
            timeout=json.load(c.open("r"))["timeout"] * 7,
        )

    for size, c in itertools.product(conv_2d_sizes, cs):
        make_2d_conv(
            Path("jobs"),
            *size,
            ruleset,
            c,
            "alternative",
            key="large",
            timeout=json.load(c.open("r"))["timeout"] * 7,
        )


@job()
def alpha_beta_ablation():
    conv_2d_size = [16, 16, 4, 4]
    ruleset = rulesets["ruleset_timeout86400"]
    cs = dict_from_dir(Path("../experiments/configs/ablation"))
    for c in cs.values():
        # beta = float(c.stem.split("_")[2][1:])
        # if beta in [2022]:
        make_2d_conv(
            Path("jobs"),
            *conv_2d_size,
            ruleset,
            c,
            "alternative",
            key="alpha-beta",
            timeout=json.load(c.open("r"))["timeout"] * 5,
        )


@job()
def one_off():
    size = [4]
    ruleset = rulesets["ruleset_timeout86400"]
    c = make_config(alpha=15, beta=6)

    qr_decomp(
        Path("jobs"),
        *size,
        ruleset,
        c,
        "alternative",
        key="one_off",
        timeout=c["timeout"] * 7,
    )


@job()
def estimate(args):
    """
    Run cycle estimation on
    """

    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(Path("jobs") / f"{date_str}-estimate", 0)
    job_dir.mkdir(exist_ok=False)

    config = {
        "date": date_str,
        "name": "estimate",
        "memory_limit": 220,
        "timeout": 86400,
        "command": "./run.sh",
        "metadata": {
            "params": str(args),
        },
    }
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)

    command = [
        "export LM_LICENSE_FILE=27010@10.0.2.2",
        "cd /root/comp-gen/server",
        f"./process.py all completed/",
        f"./estimate.py many latest --key {args} --xtensa /root/xtensa --dios /root/diospyros",
    ]

    with (job_dir / "run.sh").open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", "\n".join(command)]))
    os.chmod(str(job_dir / "run.sh"), 0o777)


@click.command()
@click.argument("job_name", nargs=-1)
def cli(job_name):
    available_jobs = "\n".join(map(lambda x: f" - {x}", JOBS.keys()))

    if len(job_name) == 0:
        print(f"Available jobs:\n{available_jobs}")
        return

    for name in job_name:
        if ":" in name:
            job, args = name.split(":")
            args = (args,)
        else:
            job = name
            args = ()

        if job in JOBS:
            JOBS[job](*args)
        else:
            raise Exception(
                f"`{job}` is not a known job! Available jobs:\n{available_jobs}"
            )


if __name__ == "__main__":
    cli()
