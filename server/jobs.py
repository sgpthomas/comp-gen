#!/usr/bin/env python3

import itertools
import json
import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Any, Callable, Dict, List, Tuple

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

        def f(*args, **kwargs):
            print(f"Creating {job_name} jobs")
            func(*args, **kwargs)

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


def make_run_script(job_dir: Path, command: List[str], script_name: str = "run.sh"):
    """
    Make a script that executes `command' and place it into `job_dir'.
    """

    with (job_dir / script_name).open("w") as f:
        f.writelines("\n".join(["#!/usr/bin/env bash", "", "\n".join(command)]))
    os.chmod(str(job_dir / script_name), 0o777)


def make_job_dir(
    jobs_dir: Path, name: str, config: Dict, command: List[str] | None = None
) -> Path:
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(jobs_dir / f"{date_str}-{name}", 0)
    job_dir.mkdir(exist_ok=False)

    config = {"date": date_str, "name": name, **config}
    json.dump(config, (job_dir / "config.json").open("w"), indent=2)

    if command is not None:
        make_run_script(job_dir, command)

    return job_dir


def compgen_run(benchmark: str, costfn: str, vector_width: int = 4) -> str:
    return " ".join(
        [
            "RUST_LOG=debug,egg=info",
            "$compgen_bin",
            "compile",
            benchmark,
            "--dios-bin",
            "$dios_bin",
            "--dios-example-bin",
            "$dios_example_bin",
            "--dios-params",
            "params.json",
            "--vector-width",
            str(vector_width),
            "--rules",
            "rules.json",
            "--config",
            "compile.json",
            "--output-dir",
            "results",
            "--costfn",
            costfn,
        ]
    )


def make_config(alpha=15, beta=6, timeout=180, prune=True) -> Dict[str, Any]:
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
                            "fresh_egraph": prune,
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
                    "fresh_egraph": prune,
                    "iter_limit": 10,
                    "scheduler": "simple",
                    "disabled": False,
                },
            ]
        },
    }


def mat_mul(
    jobs_dir: Path,
    size: Tuple[int, int, int, int],
    ruleset: Path,
    compile: Dict[str, Any],
    costfn: str = "alternative",
    key=None,
    timeout=60 * 30,
    memlimit=220,
    after: str | None = None,
):
    a_rows, a_cols, b_rows, b_cols = size
    job_dir = make_job_dir(
        jobs_dir,
        f"mat-mul_{a_rows}x{a_cols}_{b_rows}x{b_cols}",
        {
            "key": key,
            "memory_limit": memlimit,
            "timeout": timeout,
            "command": "./run.sh",
            "after": after,
            "metadata": {
                "rules.json": str(ruleset),
                "compile.json": compile_name(compile),
                "costfn": costfn,
            },
        },
        [compgen_run("mat-mul", costfn)],
    )
    shutil.copy(ruleset, job_dir / "rules.json")
    create_compile_config(compile, job_dir / "compile.json")

    params = {
        "A-rows": a_rows,
        "A-cols": a_cols,
        "B-rows": b_rows,
        "B-cols": b_cols,
        "reg-size": 4,
    }
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)


def make_2d_conv(
    jobs_dir: Path,
    size: Tuple[int, int, int, int],
    ruleset,
    compile: Dict[str, Any],
    costfn: str = "alternative",
    key=None,
    timeout=60 * 30,
    memlimit=220,
    after: str | None = None,
    metadata: Dict[str, Any] | None = None,
):
    irows, icols, frows, fcols = size
    if metadata is None:
        metadata = {}
    job_dir = make_job_dir(
        jobs_dir,
        f"2d-conv_{irows}x{icols}_{frows}x{fcols}",
        {
            "key": key,
            "memory_limit": memlimit,
            "timeout": timeout,
            "command": "./run.sh",
            "after": after,
            "metadata": {
                "rules.json": str(ruleset),
                "compile.json": compile_name(compile),
                "costfn": costfn,
                **metadata,
            },
        },
        [compgen_run("2d-conv", costfn)],
    )

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


def q_prod(
    jobs_dir: Path,
    ruleset,
    compile: Dict[str, Any],
    costfn: str = "alternative",
    key=None,
    timeout=60 * 30,
    memlimit=220,
    after: str | None = None,
):
    job_dir = make_job_dir(
        jobs_dir,
        "q-prod",
        {
            "key": key,
            "memory_limit": memlimit,
            "timeout": timeout,
            "command": "./run.sh",
            "after": after,
            "metadata": {
                "rules.json": str(ruleset),
                "compile.json": compile_name(compile),
                "costfn": costfn,
            },
        },
        [compgen_run("q-prod", costfn)],
    )

    params = {"reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    create_compile_config(compile, job_dir / "compile.json")


def qr_decomp(
    jobs_dir: Path,
    N: int,
    ruleset,
    compile: Dict[str, Any],
    costfn: str = "alternative",
    key: str | None = None,
    timeout=60 * 30,
    memlimit: int = 220,
    after: str | None = None,
):
    job_dir = make_job_dir(
        jobs_dir,
        f"qr-decomp_{N}",
        {
            "key": key,
            "memory_limit": memlimit,
            "timeout": timeout,
            "command": "./run.sh",
            "after": after,
            "metadata": {
                "rules.json": str(ruleset),
                "compile.json": compile_name(compile),
                "costfn": costfn,
            },
        },
        [compgen_run("qr-decomp", costfn)],
    )

    params = {"N": N, "reg-size": 4}
    json.dump(params, (job_dir / "params.json").open("w"), indent=2)

    shutil.copy(ruleset, job_dir / "rules.json")
    create_compile_config(compile, job_dir / "compile.json")


def make_synthesis(
    jobs_dir: Path,
    synth_timeout: int,
    name: str = "synthesis",
    eqsat_iter: int = 3,
    eqsat_timeout: int = 60,
    binops: List[str] | None = None,
    triops: List[str] | None = None,
    memlimit: int = 450,
    after: str | None = None,
):
    synth_config = json.load((Path("synthesis") / "base.json").open("r"))
    synth_config["ruler_config"]["abs_timeout"] = synth_timeout
    synth_config["ruler_config"]["eqsat_iter_limit"] = eqsat_iter
    synth_config["ruler_config"]["eqsat_time_limit"] = eqsat_timeout
    if binops is not None:
        synth_config["binops"] = binops
    if triops is not None:
        synth_config["triops"] = triops

    job_dir = make_job_dir(
        jobs_dir,
        f"{name}-{synth_timeout}",
        {
            "memory_limit": memlimit,
            "timeout": synth_timeout * 5,
            "command": "./run.sh",
            "after": after,
            "metadata": {
                "timeout": synth_timeout,
                "eqsat_iter_limit": eqsat_iter,
                "eqsat_timeout": eqsat_timeout,
                "binops": binops,
                "triops": triops,
            },
        },
        [
            " ".join(
                [
                    "RUST_LOG=debug,egg=info,z3=off",
                    "$compgen_bin",
                    "synth",
                    "ruleset.json",
                    "--config",
                    "synth.json",
                ]
            )
        ],
    )

    json.dump(synth_config, (job_dir / "synth.json").open("w"), indent=2)


def dict_from_dir(path: Path, pat: str = "*.json", key=None) -> Dict[str, Path]:
    path = path.expanduser().resolve()
    assert path.exists()

    res = {}

    for p in path.glob(pat):
        k = p.stem
        if key is not None and isinstance(key, Callable):
            k = str(key(p))
        res[k] = p.expanduser().resolve()

    return res


# rulesets = dict_from_dir(Path("../experiments/rulesets"))
# configs = dict_from_dir(Path("../experiments/configs"))


@job()
def overall_performance(
    rulesets: Dict[str, Path] | None = None, after=None, memlimit=220, **_
):
    """
    This measures the overall performance of the compiler in terms of
    1) How good are the results of the compiler?
    2) How fast is the compiler?
    3) How well does the compiler scale?

    This is a run with all of the benchmarks that Dios uses.
    """

    assert rulesets is not None

    mat_mul_sizes = [
        (2, 2, 2, 2),
        (2, 3, 3, 3),
        (3, 3, 3, 3),
        (4, 4, 4, 4),
        (8, 8, 8, 8),
        (10, 10, 10, 10),
        (16, 16, 16, 16),
        (18, 18, 18, 18),
        (20, 20, 20, 20),
    ]
    conv_2d_sizes = [
        (3, 3, 2, 2),
        (3, 3, 3, 3),
        (3, 5, 3, 3),
        (4, 4, 3, 3),
        (8, 8, 3, 3),
        (10, 10, 2, 2),
        (10, 10, 3, 3),
        (10, 10, 4, 4),
        (16, 16, 2, 2),
        (16, 16, 3, 3),
        (16, 16, 4, 4),
        (18, 18, 2, 2),
        (18, 18, 3, 3),
        (18, 18, 4, 4),
    ]
    qr_decomp_sizes = [3, 4]
    ruleset = rulesets["ruleset_timeout86400"]

    # create all the jobs
    for size in mat_mul_sizes:
        mat_mul(
            Path("jobs"),
            size,
            ruleset,
            make_config(alpha=15, beta=6, timeout=180),
            key="performance",
            timeout=3000,
            memlimit=memlimit,
            after=after,
        )

    for size in conv_2d_sizes:
        make_2d_conv(
            Path("jobs"),
            size,
            ruleset,
            make_config(alpha=15, beta=6, timeout=180),
            key="performance",
            timeout=3000,
            memlimit=memlimit,
            after=after,
        )

    q_prod(
        Path("jobs"),
        ruleset,
        make_config(alpha=15, beta=6, timeout=180),
        key="performance",
        timeout=3000,
        memlimit=memlimit,
        after=after,
    )

    for size in qr_decomp_sizes:
        qr_decomp(
            Path("jobs"),
            size,
            ruleset,
            make_config(alpha=15, beta=6, timeout=180),
            key="performance",
            timeout=10000,
            memlimit=memlimit,
            after=after,
        )


@job()
def diospyros(after=None, memlimit=220, **_):
    """
    Run diospyros experiments.
    """

    job_dir = make_job_dir(
        Path("jobs"),
        "diospyros",
        {
            "memory_limit": memlimit,
            "timeout": 10000,
            "command": "./run.sh",
            "after": after,
            "key": "diospyros",
            "metadata": {"timeout": 180},
        },
        [
            'export PATH="/root/.cargo/bin:/root/racket/bin:$PATH"',
            "root_dir=$(pwd)",
            "git clone https://github.com/cucapra/diospyros.git",
            "cp asplos_parameters.json diospyros/evaluation/asplos_parameters.json",
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
        ],
    )

    # we are using more sizes than diospyros originally ran
    # update the asplos-parameters.json file to reflect the
    # sizes that we actually want to run

    mat_mul_sizes = [
        (2, 2, 2, 2),
        (2, 3, 3, 3),
        (3, 3, 3, 3),
        (4, 4, 4, 4),
        (8, 8, 8, 8),
        (10, 10, 10, 10),
        (16, 16, 16, 16),
        (18, 18, 18, 18),
        (20, 20, 20, 20),
    ]
    conv_2d_sizes = [
        (3, 3, 2, 2),
        (3, 3, 3, 3),
        (3, 5, 3, 3),
        (4, 4, 3, 3),
        (8, 8, 3, 3),
        (10, 10, 2, 2),
        (10, 10, 3, 3),
        (10, 10, 4, 4),
        (16, 16, 2, 2),
        (16, 16, 3, 3),
        (16, 16, 4, 4),
        (18, 18, 2, 2),
        (18, 18, 3, 3),
        (18, 18, 4, 4),
    ]
    qr_decomp_sizes = [3, 4]

    experiment_params = {"mat-mul": [], "2d-conv": [], "q-prod": [], "qr-decomp": []}
    for s in mat_mul_sizes:
        arows, acols, brows, bcols = s
        experiment_params["mat-mul"] += [
            {
                "A-rows": arows,
                "A-cols": acols,
                "B-rows": brows,
                "B-rows": bcols,
                "reg-size": 4,
            }
        ]

    for s in conv_2d_sizes:
        irows, icols, frows, fcols = s
        experiment_params["2d-conv"] += [
            {
                "input-rows": irows,
                "input-cols": icols,
                "filter-rows": frows,
                "filter-cols": fcols,
                "reg-size": 4,
            }
        ]

    experiment_params["q-prod"] += [{"reg-size": 4}]

    for N in qr_decomp_sizes:
        experiment_params["qr-decomp"] += [
            {
                "N": N,
                "reg-size": 4,
            }
        ]

    json.dump(
        experiment_params,
        (job_dir / "asplos_parameters.json").open("w"),
        indent=2,
    )


@job()
def pruning(rulesets: Dict[str, Path] | None = None, after=None, memlimit=100, **_):
    """
    This experiment is meant to show that pruning dramatically decreases
    the how long it takes to find a low-cost program.
    """

    assert rulesets is not None

    params = [
        (3, 3, 2, 2),
        (3, 3, 3, 3),
        (3, 5, 3, 3),
        (4, 4, 3, 3),
        (8, 8, 3, 3),
        (10, 10, 2, 2),
        (10, 10, 3, 3),
        (10, 10, 4, 4),
        (16, 16, 2, 2),
        (16, 16, 3, 3),
        (16, 16, 4, 4),
        (18, 18, 2, 2),
        (18, 18, 3, 3),
        (18, 18, 4, 4),
    ]
    timeout = 360

    for p in params:
        # pruning config
        make_2d_conv(
            Path("jobs"),
            p,
            rulesets["ruleset_timeout86400"],
            make_config(timeout=timeout, prune=True),
            key="pruning",
            after=after,
            memlimit=memlimit,
            timeout=timeout * 7,
            metadata={"pruning": True},
        )
        # no pruning config
        make_2d_conv(
            Path("jobs"),
            p,
            rulesets["ruleset_timeout86400"],
            make_config(timeout=timeout, prune=False),
            key="pruning",
            after=after,
            memlimit=memlimit,
            timeout=timeout * 7,
            metadata={"pruning": False},
        )


@job()
def ruleset_synthesis(after=None, memlimit=220, **_):
    """
    Synthesize rulesets with different timeouts.
    """

    timeouts = [
        60,
        600,
        6000,
        60000,
    ]

    for t in timeouts:
        make_synthesis(
            Path("jobs"),
            t,
            name="synthesis",
            binops=["+", "-", "*", "/"],
            triops=["mac"],
            eqsat_iter=3,
            eqsat_timeout=60,
            after=after,
            memlimit=memlimit,
        )


@job()
def ruleset_ablation(
    rulesets: Dict[str, Path] | None = None, after=None, memlimit=220, **_
):
    """
    Measure the performance difference between using different rulesets.
    """

    conv_2d_sizes = [
        (3, 3, 2, 2),
        (3, 3, 3, 3),
        (3, 5, 3, 3),
        (4, 4, 3, 3),
        (8, 8, 3, 3),
        (10, 10, 2, 2),
        (10, 10, 3, 3),
        (10, 10, 4, 4),
        (16, 16, 2, 2),
        (16, 16, 3, 3),
        (16, 16, 4, 4),
        (18, 18, 2, 2),
        (18, 18, 3, 3),
        (18, 18, 4, 4),
    ]

    if rulesets is not None:
        rs = list(
            map(
                lambda x: rulesets[x],
                ["timeout_60", "timeout_600", "timeout_6000", "timeout_60000"],
            )
        )
    else:
        raise NotImplementedError()

    exps = itertools.product(conv_2d_sizes, rs)
    for size, r in exps:
        make_2d_conv(
            Path("jobs"),
            size,
            r,
            make_config(timeout=1800),
            key="ruleset_ablation",
            after=after,
            memlimit=memlimit,
        )


@job()
def new_instructions_ruleset(after=None, memlimit=450, **_):
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
        name="new_instructions",
        binops=binops + ["sqrtsgn"],
        triops=["mac", "muls"],
        after=after,
        memlimit=memlimit,
    )
    # baseline + muls
    make_synthesis(
        Path("jobs"),
        60000,
        name="new_instructions",
        binops=binops,
        triops=["mac", "muls"],
        after=after,
        memlimit=memlimit,
    )
    # baseline + mulsgn
    make_synthesis(
        Path("jobs"),
        60000,
        name="new_instructions",
        binops=binops + ["~*"],
        triops=["mac"],
        after=after,
        memlimit=memlimit,
    )


@job()
def test_instruction_ruleset(
    rulesets: Dict[str, Path] | None = None, after=None, memlimit=220, **_
):
    """
    Run q-prod, qr-decomp with no fused ops, with muls, and with both fused ops
    """

    assert rulesets is not None

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

    for _, r in rulesets.items():
        qr_decomp(
            Path("jobs"),
            3,
            r,
            make_config(timeout=360),
            "alternative",
            key="instruction",
            timeout=7200,
        )


@job()
def large_kernels(
    rulesets: Dict[str, Path] | None = None, after=None, memlimit=220, **_
):
    assert rulesets is not None

    mat_mul_sizes = [
        (25, 25, 25, 25),
        (30, 30, 30, 30),
        (40, 40, 40, 40),
    ]
    conv_2d_sizes = [
        (20, 20, 5, 5),
        (30, 30, 5, 5),
        (32, 32, 5, 5),
        (34, 34, 5, 5),
        (36, 36, 5, 5),
        (38, 38, 5, 5),
        (40, 40, 5, 5),
    ]
    ruleset = rulesets["ruleset_timeout86400"]

    # create all the jobs
    for size in mat_mul_sizes:
        mat_mul(
            Path("jobs"),
            size,
            ruleset,
            make_config(timeout=18000),
            key="large",
            timeout=130000,
            after=after,
            memlimit=memlimit,
        )

    for size in conv_2d_sizes:
        make_2d_conv(
            Path("jobs"),
            size,
            ruleset,
            make_config(timeout=18000),
            key="large",
            timeout=130000,
            after=after,
            memlimit=memlimit,
        )


@job()
def alpha_beta_ablation(
    rulesets: Dict[str, Path] | None = None, after=None, memlimit=220, **_
):
    assert rulesets is not None

    conv_2d_size = (16, 16, 4, 4)
    ruleset = rulesets["ruleset_timeout86400"]
    alphas = [
        -4045,
        -4040,
        -4035,
        -15,
        -10,
        -5,
        -1,
        -0.5,
        0.5,
        1,
        5,
        10,
        15,
        4035,
        4040,
        4045,
    ]
    betas = [0, 1, 2, 3, 4, 5, 6, 10, 15, 20, 30, 2020, 2025]
    for beta in betas:
        for alpha in alphas:
            make_2d_conv(
                Path("jobs"),
                conv_2d_size,
                ruleset,
                make_config(alpha=alpha, beta=beta),
                key="alpha-beta",
                timeout=1000,
                after=after,
                memlimit=memlimit,
            )


@job()
def estimate(args, after=None, memlimit=220, **_):
    """
    Run cycle estimation on jobs that the key: `args`
    """

    make_job_dir(
        Path("jobs"),
        "estimate",
        {
            "timeout": 86400,
            "command": "./run.sh",
            "memory_limit": memlimit,
            "after": after,
            "metadata": {
                "params": str(args),
            },
        },
        [
            "export LM_LICENSE_FILE=27010@10.0.2.2",
            "cd /root/comp-gen/server",
            f"./process.py all completed/",
            f"./estimate.py many latest --key {args} --xtensa /root/xtensa --dios /root/diospyros",
        ],
    )


@click.command()
@click.argument("job_name", type=str)
@click.option("--rulesets", type=click.Path(exists=True), default="rulesets")
@click.option("--after", type=str)
@click.option("--memlimit", type=int)
def cli(job_name: str, rulesets: str, after: str, memlimit: int):
    available_jobs = "\n".join(map(lambda x: f" - {x}", JOBS.keys()))

    if ":" in job_name:
        job, args = job_name.split(":")
        args = (args,)
    else:
        job = job_name
        args = ()

    if job not in available_jobs:
        print(f"`{job}` is not a known job! Available jobs:\n{available_jobs}")
        return

    kwargs = {
        "rulesets": dict_from_dir(Path(rulesets)),
    }

    if after is not None:
        kwargs = {**kwargs, "after": after}

    if memlimit is not None:
        kwargs = {**kwargs, "memlimit": memlimit}

    if job in JOBS:
        JOBS[job](*args, **kwargs)
    else:
        print(f"`{job}` is not a known job! Available jobs:\n{available_jobs}")
        return


if __name__ == "__main__":
    cli()
