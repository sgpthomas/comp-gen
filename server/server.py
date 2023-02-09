from pathlib import Path
import shutil
import json
import subprocess
from datetime import datetime
from multiprocessing import Process
import time
import psutil


DEFAULT_CONFIG = {
    "compgen_bin": "dios-lang",
    "dios_bin": "diospyros/dios",
    "dios_example_bin": "diospyros/dios-example-gen",
    "rulesets": {},
    "compile_configs": {}
}


def unique_name(path: Path, suffix: int = 0) -> Path:
    """Find a unique path based on `path` and adding a suffix."""

    new_path = Path(path.parent) / f"{path.name}-{suffix}"
    if not new_path.exists():
        return new_path
    else:
        return unique_name(path, suffix + 1)


class Task:
    def __init__(self, global_config, completed: Path, task_dir: Path):
        assert Task.valid(task_dir)

        self.global_config = global_config
        self.completed = completed

        self.task_dir = task_dir
        self.config = json.load((task_dir / "config.json").open("r"))

        self.benchmark = self.config["benchmark"]
        self.rules_path = global_config["rulesets"][self.config["ruleset"]]
        self.config_path = global_config["compile_configs"][self.config["compile_config"]]
        self.output_dir = task_dir / "results"

    def valid(task_dir):
        """Check if `task_dir` has the right files to be a task."""

        dir_exists = task_dir.exists()
        config_b = (task_dir / "config.json").exists()
        return dir_exists and config_b

    def exec(self):

        # write out the params to a file
        params_path = self.task_dir / "params.json"
        json.dump(self.config["params"], params_path.open("w"))

        # make sure that the results dir exists
        self.output_dir.mkdir(exist_ok=False)

        # prepare log files
        stdout_log = self.output_dir / "stdout.log"
        stderr_log = self.output_dir / "stderr.log"
        stdout_log.touch()
        stderr_log.touch()

        # run the comp-gen binary, storing stdout and stderr in logs
        subprocess.run(
            [
                self.global_config["compgen_bin"],
                "compile", self.benchmark,
                "--dios-bin", self.global_config["dios_bin"],
                "--dios-example-bin", self.global_config["dios_example_bin"],
                "--dios-params", str(params_path),
                "--vector-width", "4",
                "--rules", str(self.rules_path),
                "--config", str(self.config_path),
                "--output-dir", str(self.output_dir)
            ],
            env={"RUST_LOG": "debug,egg=info"},
            stdout=stdout_log.open("w"),
            stderr=stderr_log.open("w")
        )

    def complete(self):
        completed_dir = unique_name(self.completed / self.task_dir.name)
        completed_dir.mkdir(exist_ok=False)

        # copy over results, job config, and params.json
        shutil.copytree(self.output_dir, completed_dir / "results")
        json.dump(self.config, (completed_dir / "config.json").open("w"), indent=2)

        # remove the job directory
        print(self.task_dir)
        shutil.rmtree(self.task_dir)

    def __repr__(self):
        return f"<Task bench={self.benchmark} ruleset={self.rules_path} compile={self.config_path}>"

    def __eq__(self, other):
        return self.task_dir == other.task_dir

    def __hash__(self):
        return hash(self.task_dir)


def setup():
    """Make sure that the directories `jobs` and `completed` exist."""

    completed = Path("completed")
    completed.mkdir(exist_ok=True)
    jobs = Path("jobs")
    jobs.mkdir(exist_ok=True)
    # config_path = Path("config.json")
    # if not config_path.exists():
    return (completed, jobs)


def list_jobs(config, completed, jobs_dir):
    """Read jobs directory and produce a list of tasks to perform."""

    jobs = []
    for dir in jobs_dir.glob("*"):
        if Task.valid(dir):
            task = Task(config, completed, dir)
            jobs += [task]

    return jobs


def commit_config(config):
    """Write out the config to a file."""

    with Path("config.json").open("w") as f:
        json.dump(config, f, indent=2)


def make_job(jobs_dir, benchmark, ruleset_name, compile_config):
    config = None
    if benchmark == "mat-mul":
        config = {
            "benchmark": "mat-mul",
            "ruleset": ruleset_name,
            "compile_config": compile_config,
            "params": {
                "A-rows": 2,
                "A-cols": 2,
                "B-rows": 2,
                "B-cols": 2,
                "reg-size": 4
            }
        }
    else:
        raise Exception("Unknown benchmark")
    
    date_str = datetime.now().strftime("%b%d-%H%M")
    job_dir = unique_name(jobs_dir / f"{date_str}-{benchmark}", 0)
    job_dir.mkdir(exist_ok=False)
    json.dump(config, (job_dir / "config.json").open("w"))
    print(f"Created {benchmark} job at {job_dir}")


def memory_used_by(pid):
    """Return the memory used by process `pid` in MB."""

    process = psutil.Process(pid)
    mem = 0
    for child in process.children(recursive=True):
        mem += child.memory_info().rss / float(10 ** 6)
    return mem


def main():
    (completed, jobs) = setup()

    config_path = Path("config.json")
    if config_path.exists():
        with config_path.open("r") as f:
            config = json.load(f)
    else:
        config = DEFAULT_CONFIG

    make_job(jobs, "mat-mul", "expanding", "phased")

    # print(config)

    try:
        alive = {}
        while True:
            print("Looking for jobs...")
            for job in list_jobs(config, completed, jobs):
                # if we have found a job that we haven't seen before
                # start a new process for it
                if job not in alive:
                    print(f"Found {job}")
                    p = Process(target=job.exec, args=())
                    alive[job] = p
                    alive[job].start()

            # keep track of the things that are dead on this time around
            dead = []

            # check if anything in the currently running processes has finished
            for job, proc in alive.items():
                if proc.is_alive():
                    print(f"{job}: {proc.is_alive()}, {proc.pid}")
                    print("  memory used: ", memory_used_by(proc.pid), "MB")

                    # if the job output dir has been created. start recording memory usage
                    if job.output_dir.exists():
                        memory_csv = job.output_dir / "memory.csv"
                        memory_csv.touch()
                        with memory_csv.open("a") as f:
                            print(
                                "{},{}".format(time.time(), memory_used_by(proc.pid)),
                                file=f,
                                flush=True
                            )
                else:
                    job.complete()
                    dead += [job]

            # remove all the dead processes
            for job in dead:
                del alive[job]

            print(f"Alive jobs: {alive}")
            print("Waiting 2s")
            time.sleep(2)
            

    except KeyboardInterrupt:
        print("Stopping...")

    except Exception as e:
        print(f"Unknown exception: {e}")

    commit_config(config)


if __name__ == "__main__":
    main()
