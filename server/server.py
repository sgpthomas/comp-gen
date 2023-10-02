import json
import os
import random
import shutil
import string
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, Iterator, List, Set

import click
import psutil


def sort_path(path):
    """
    Sort a path by attempting to split it by some common
    seperating characters and converting things into ints.
    """

    stem = path.stem.replace("_", "-").replace("x", "-").split("-")
    stem = list(map(lambda x: int(x) if x.isnumeric() else x, stem))
    return stem


def generate_unique_exp_id(parent: Path) -> Path:
    """
    Generate a unique path relative to a provided parent path.

    This needs to generate a path that is unique across any machine
    that deals with these paths without any communication. While, this
    function isn't perfect because it uses a randomly generated seed alongside
    the date. It should be exceedingly unlikely that two paths clash.
    """

    datestr = datetime.now().strftime("%b%d-%H%M")
    randstr = "".join(random.choices(string.ascii_uppercase + string.digits, k=5))
    ret_path = parent / f"{datestr}-{randstr}"

    # crash if this path exists already.
    # this really shouldn't happen
    assert not ret_path.exists()

    return ret_path


class GlobalConfig:
    def __init__(self, root: Path):
        self.config_path = root / "config.json"
        self.jobs = root / "jobs"
        self.completed = root / "completed"
        self.diospyros = root / ".." / ".." / "custom-diospyros"

        # make sure that the jobs and completed directory exist
        self.jobs.mkdir(exist_ok=True)
        self.completed.mkdir(exist_ok=True)

        # the default values of configuration
        self.env = {}
        self.jobs_at_once = 1

        # compute memory available on machine
        mem_bytes = os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")
        self.machine_memory = mem_bytes / float(1024**3)

    def list_jobs(self) -> Iterator["Job"]:
        """Read jobs directory and produce an iterator of tasks to perform."""

        for task_dir in sorted(self.jobs.glob("*"), key=sort_path):
            if Job.valid(task_dir):
                yield Job(self, task_dir)

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
            for key, value in data["env"].items():
                path = Path(value).expanduser().resolve()
                if path.exists():
                    self.env[key] = path
                else:
                    self.env[key] = value
            self.jobs_at_once = data["jobs_at_once"]
            self.diospyros = data["diospyros"]

    def commit(self):
        """Commit the current configuration to a file."""

        with self.config_path.open("w") as f:
            data = {"env": self.env, "jobs_at_once": self.jobs_at_once}
            json.dump(data, f, indent=2)


class Job:
    def __init__(self, global_config: GlobalConfig, job_dir: Path):
        assert Job.valid(job_dir)

        data = json.load((job_dir / "config.json").open("r"))
        self.global_config = global_config
        self.dir = job_dir
        self.date = data["date"]
        self.name = data["name"]
        self.memory_limit = data["memory_limit"]
        self.command = data["command"]
        self.key: str | None = None
        if "key" in data and isinstance(data["key"], str):
            self.key = data["key"]
        self.after: str | None = None
        if "after" in data and isinstance(data["after"], str):
            self.after = data["after"]
        self.start_time = None
        if "timeout" in data:
            self.timeout = data["timeout"]
        else:
            self.timeout = 60 * 30

    @staticmethod
    def valid(job_dir: Path):
        """Check if `job_dir` contains a valid job."""

        config = job_dir / "config.json"

        if not config.exists():
            return False

        data = json.load(config.open("r"))
        return all(
            map(
                lambda k: k in data,
                ["date", "name", "memory_limit", "command"],
            )
        )

    def exec(self) -> subprocess.Popen:
        # prepare log files
        stdout_log = self.dir / "stdout.log"
        stderr_log = self.dir / "stderr.log"
        stdout_log.touch()
        stderr_log.touch()

        compgen_commit = subprocess.run(
            "git rev-parse HEAD", shell=True, capture_output=True
        )
        dios_commit = subprocess.run(
            "git rev-parse HEAD",
            shell=True,
            capture_output=True,
            cwd=self.global_config.diospyros,
        )
        with (self.dir / "commit.txt").open("w") as f:
            f.writelines(
                [
                    f"comp-gen: {compgen_commit.stdout.strip()}\n",
                    f"diospyros: {dios_commit.stdout.strip()}\n",
                ]
            )

        # save start time
        self.start_time = time.time()

        # run the comp-gen binary, storing stdout and stderr in logs
        self.proc = subprocess.Popen(
            f"{self.command}",
            env=self.global_config.env | dict(os.environ),
            stdout=stdout_log.open("w"),
            stderr=stderr_log.open("w"),
            cwd=self.dir,
        )
        return self.proc

    def complete(self):
        parent_dir = self.global_config.completed / self.name
        for _ in range(5):
            try:
                parent_dir.mkdir(exist_ok=True, parents=True)
                break
            except FileNotFoundError:
                print("Something went wrong. Waiting 5s and then trying again...")
                time.sleep(5)
        results = generate_unique_exp_id(parent_dir)

        # copy over results, job config, and params.json
        shutil.copytree(self.dir, results)

        # remove the job directory
        shutil.rmtree(self.dir)

    def __eq__(self, obj):
        return isinstance(obj, Job) and self.dir == obj.dir

    def __hash__(self):
        return hash(self.dir)

    def __repr__(self):
        return f"<Job {self.dir} {self.date}>"


def memory_used_by(pid) -> float:
    """
    Return the memory used by `pid` and
    all children of `pid` in GB.
    """

    process = psutil.Process(pid)
    mem = process.memory_info().rss
    for child in process.children(recursive=True):
        mem += child.memory_info().rss
    return mem / float(1024**3)


def single_run(config: GlobalConfig, alive: Dict[Job, subprocess.Popen], update=False):
    """
    A single round of the main loop. Each time this is called, it
    1) Tries to start any new jobs that we have the resources for
    2) Updates stats collected about running jobs (killing them if necessary)
    3) Completes jobs by moving them to completed/
    """

    if update:
        subprocess.run("git pull -r", shell=True)
        subprocess.run(
            "cargo build --release --manifest-path=../dios-lang/Cargo.toml", shell=True
        )
        subprocess.run("git pull -r", shell=True, cwd=config.diospyros)
        subprocess.run("make dios dios-example-gen", shell=True, cwd=config.diospyros)

    # try and start any new jobs
    print("Looking for jobs...")
    queued_jobs: List[Job] = list(config.list_jobs())
    # the unique set of keys for queued jobs
    queued_keys: Set[str] = set([j.key for j in queued_jobs if j.key is not None])
    print(f"Considering {len(queued_jobs)} jobs")

    for job in queued_jobs:
        # start a job if:
        if (
            # 1) the job is not already running
            job not in alive
            # 2) we don't have more that `config.jobs_at_once` jobs running
            and len(alive) < config.jobs_at_once
            # 3) the memory limit for this job is smaller than memory for all job
            and job.memory_limit + sum([j.memory_limit for j in alive.keys()])
            < config.machine_memory
            # 4) there are no queued jobs that have key "after"
            #    if job.after is None, then this check will be True
            #    because None is not in queued_keys
            and job.after not in queued_keys
        ):
            print(f"Starting {job}")
            try:
                alive[job] = job.exec()
            except PermissionError:
                print(f"Don't have permission to run command for {job}!")
            except Exception as e:
                print(f"An exception occurred while trying to start job: {job}.")
                print(e)

    # keep track of the things that are dead on this time around
    dead = []

    # check if anything in the currently running processes has finished
    print(f"{len(alive)} alive jobs:")
    for job, proc in alive.items():
        # proc is alive when poll() returns None
        if proc.poll() is None:
            memory_used = memory_used_by(proc.pid)
            print(f" - {job} ({proc.pid}) {memory_used=} GB")

            # where to write the memory report
            memory_csv = job.dir / "memory.csv"

            # this is set when the job is started
            assert job.start_time is not None
            time_diff = time.time() - job.start_time

            # record memory usage
            memory_csv.touch()
            with memory_csv.open("a") as f:
                print("{},{}".format(time_diff, memory_used), file=f, flush=True)

            # if we have exceeded the memory limit, kill this
            # process and write a line out to memory.csv
            if memory_used > int(job.memory_limit):
                print("To much memory used! Killing!", job)
                proc.terminate()
                print("Waiting for process to die...", end="")
                proc.wait()
                print("Dead!")
                with memory_csv.open("a") as f:
                    print("{},{}".format(time_diff, "killed"), file=f, flush=True)

            # if we have exceeded the time limit, kill this
            # process and write a line out to memory.csv
            if job.timeout is not None and time_diff > job.timeout:
                print(f"Timing out after {time_diff} seconds!")
                proc.terminate()
                print("Waiting for process to die", end="...")
                proc.wait()
                print("Dead!!")
                with memory_csv.open("a") as f:
                    print("{},{}".format(time_diff, "timeout"), file=f, flush=True)

        # if the job isn't alive, add it to the dead list
        else:
            print("Job is dead: ", job)
            job.complete()
            dead += [job]

    # remove all the dead processes
    for job in dead:
        del alive[job]


@click.command()
@click.option("--auto-update", is_flag=True)
def main(auto_update):
    config = GlobalConfig(Path("."))

    try:
        alive: Dict[Job, subprocess.Popen] = {}
        i = 0
        while True:
            config.reload()
            single_run(config, alive, update=auto_update and i == 0)

            time.sleep(5)
            i = (i + 1) % 10

    except KeyboardInterrupt:
        print("Stopping...")

    except Exception as e:
        print(f"Unknown exception: {e}")
        raise e


if __name__ == "__main__":
    main()
