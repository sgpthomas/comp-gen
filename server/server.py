from pathlib import Path
import shutil
import json
import subprocess
import time
import psutil


def unique_name(path: Path, suffix: int = 0) -> Path:
    """Find a unique path based on `path` and adding a suffix."""

    new_path = Path(path.parent) / f"{path.name}-{suffix}"
    if not new_path.exists():
        return new_path
    else:
        return unique_name(path, suffix + 1)


class GlobalConfig:
    def __init__(self, root: Path):
        self.config_path = root / "config.json"
        self.jobs = root / "jobs"
        self.completed = root / "completed"

        # make sure that the jobs and completed directory exist
        self.jobs.mkdir(exist_ok=True)
        self.completed.mkdir(exist_ok=True)

        # the default values of configuration
        self.compgen_bin = "dios-lang/target/release/dios-lang"
        self.dios_bin = "diospyros/dios"
        self.dios_example_bin = "diospyros/dios-example-gen"
        self.rulesets = {}
        self.compile_configs = {}
        self.jobs_at_once = 1

    def list_jobs(self):
        """Read jobs directory and produce an iterator of tasks to perform."""

        for task_dir in self.jobs.glob("*"):
            if Task.valid(task_dir):
                yield Task(self, task_dir)

    def reload(self):
        """Reload the configuration from `config_path`."""

        # if the config_path doesn't exist, commit the current
        # configuration (which will just be the default values)
        # and return
        if not self.config_path.exists():
            self.commit()
            return

        with self.config_path.open("r") as f:
            data = json.load(f)
            self.compgen_bin = data["compgen_bin"]
            self.dios_bin = data["dios_bin"]
            self.dios_example_bin = data["dios_example_bin"]
            self.rulesets = data["rulesets"]
            self.compile_configs = data["compile_configs"]
            self.jobs_at_once = data["jobs_at_once"]

    def commit(self):
        """Commit the current configuration to a file."""

        with self.config_path.open("w") as f:
            data = {
                "compgen_bin": self.compgen_bin,
                "dios_bin": self.dios_bin,
                "dios_example_bin": self.dios_example_bin,
                "rulesets": self.rulesets,
                "compile_configs": self.compile_configs
            }
            json.dump(data, f, indent=2)


class Task:
    def __init__(self, global_config: GlobalConfig, task_dir: Path):
        assert Task.valid(task_dir)

        self.global_config = global_config

        self.task_dir = task_dir
        self.config = json.load((task_dir / "config.json").open("r"))

        self.benchmark = self.config["benchmark"]
        self.rules_path = global_config.rulesets[self.config["ruleset"]]
        self.config_path = global_config.compile_configs[
            self.config["compile_config"]
        ]
        self.cost_function = self.config["cost_function"]
        self.output_dir = task_dir / "results"

        # once started, this holds the executing child process
        self.proc = None

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

        cmd = [
                self.global_config.compgen_bin,
                "compile", self.benchmark,
                "--dios-bin", self.global_config.dios_bin,
                "--dios-example-bin", self.global_config.dios_example_bin,
                "--dios-params", str(params_path),
                "--vector-width", "4",
                "--rules", str(self.rules_path),
                "--config", str(self.config_path),
                "--output-dir", str(self.output_dir),
            ]
        if self.cost_function == "alternative":
            cmd += ["--alt_cost"]

        # run the comp-gen binary, storing stdout and stderr in logs
        self.proc = subprocess.Popen(
            cmd,
            env={"RUST_LOG": "debug,egg=info"},
            stdout=stdout_log.open("w"),
            stderr=stderr_log.open("w")
        )
        return self.proc

    def complete(self):
        completed_dir = unique_name(
            self.global_config.completed / self.task_dir.name
        )
        completed_dir.mkdir(exist_ok=False)

        # copy over results, job config, and params.json
        shutil.copytree(self.output_dir, completed_dir / "results")
        json.dump(
            self.config,
            (completed_dir / "config.json").open("w"),
            indent=2
        )

        # remove the job directory
        print(self.task_dir)
        shutil.rmtree(self.task_dir)

    def __repr__(self):
        return " ".join([
            f"<Task bench={self.benchmark}",
            f"name={self.task_dir}>",
        ])

    def __eq__(self, other):
        return self.task_dir == other.task_dir

    def __hash__(self):
        return hash(self.task_dir)


def memory_used_by(pid):
    """
    Return the memory used by `pid` and
    all children of `pid` in GB.
    """

    process = psutil.Process(pid)
    mem = process.memory_info().rss
    for child in process.children(recursive=True):
        mem += child.memory_info().rss
    return mem / float(10 ** 9)


def single_run(config, alive):
    print("Looking for jobs...")

    # reload the config, to catch any potential changes
    config.reload()

    for job in config.list_jobs():
        # if we have found a job that we haven't seen before
        # start a new process for it
        if job not in alive and len(alive) < config.jobs_at_once:
            print(f"Found {job}")
            # p = Process(target=job.exec, args=())
            alive[job] = job.exec()

    # keep track of the things that are dead on this time around
    dead = []

    # check if anything in the currently running processes has finished
    for job, proc in alive.items():
        # proc is alive when poll() returns None
        if proc.poll() is None:
            memory_used = memory_used_by(proc.pid)
            print(f"{job} ({proc.pid}) {memory_used=} GB")

            # where to write the memory report
            memory_csv = job.output_dir / "memory.csv"

            # if the job output dir has been created.
            # start recording memory usage
            if job.output_dir.exists():
                memory_csv.touch()
                with memory_csv.open("a") as f:
                    print(
                        "{},{}".format(time.time(), memory_used),
                        file=f,
                        flush=True
                    )

            # if we have exceeded the memory limit, kill this
            # process and write a line out to memory.csv
            if memory_used > int(job.config["memory_limit"]):
                print("To much memory used! Killing!", job)
                proc.terminate()
                print("Waiting for process to die...", end="")
                proc.wait()
                print("Dead!")
                with memory_csv.open("a") as f:
                    print(
                        "{},{}".format(time.time(), "killed"),
                        file=f,
                        flush=True
                    )

        # if the job isn't alive, add it to the dead list
        else:
            print("Job is dead: ", job)
            job.complete()
            dead += [job]

    # remove all the dead processes
    for job in dead:
        del alive[job]


def main():
    config = GlobalConfig(Path("."))
    config.reload()

    try:
        alive = {}
        while True:
            single_run(config, alive)

            print(f"Alive jobs: {alive}")
            time.sleep(5)

    except KeyboardInterrupt:
        print("Stopping...")

    except Exception as e:
        print(f"Unknown exception: {e}")
        raise e


if __name__ == "__main__":
    main()
