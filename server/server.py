import json
import shutil
import subprocess
import time
from pathlib import Path

import psutil


def unique_name(path: Path, suffix: int = 0) -> Path:
    """Find a unique path based on `path` and adding a suffix."""

    new_path = Path(path.parent) / f"{path.name}-{suffix}"
    if not new_path.exists():
        return new_path
    else:
        return unique_name(path, suffix + 1)


def sort_path(path):
    """
    Sort a path by attempting to split it by some common
    seperating characters and converting things into ints.
    """

    # path.replace()
    stem = path.stem.replace("_", "-").replace("x", "-").split("-")
    stem = list(map(lambda x: int(x) if x.isnumeric() else x, stem))
    return stem


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

    def list_jobs(self):
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
            map(lambda k: k in data, ["date", "name", "memory_limit", "command"])
        )

    def exec(self):
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
            env=self.global_config.env,
            stdout=stdout_log.open("w"),
            stderr=stderr_log.open("w"),
            cwd=self.dir,
        )
        return self.proc

    def complete(self):
        parent_dir = self.global_config.completed / self.name
        parent_dir.mkdir(exist_ok=True)
        children = filter(lambda x: str(x.name).isnumeric(), parent_dir.glob("*"))
        print(list(map(lambda x: x.name, parent_dir.glob("*"))))
        results = parent_dir / str(len(list(children)))

        assert not results.exists()

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


def memory_used_by(pid):
    """
    Return the memory used by `pid` and
    all children of `pid` in GB.
    """

    process = psutil.Process(pid)
    mem = process.memory_info().rss
    for child in process.children(recursive=True):
        mem += child.memory_info().rss
    return mem / float(10**9)


def single_run(config, alive, update=False):
    if update:
        subprocess.run("git pull -r", shell=True)
        subprocess.run(
            "cargo build --release --manifest-path=../dios-lang/Cargo.toml", shell=True
        )
        subprocess.run("git pull -r", shell=True, cwd=config.diospyros)
        subprocess.run("make dios dios-example-gen", shell=True, cwd=config.diospyros)

    print("Looking for jobs...")

    # reload the config, to catch any potential changes
    config.reload()

    for job in config.list_jobs():
        print(f"considering: {job}")
        # if we have found a job that we haven't seen before
        # start a new process for it
        if job not in alive and len(alive) < config.jobs_at_once:
            print(f"Found {job}")
            alive[job] = job.exec()

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


def main():
    config = GlobalConfig(Path("."))

    try:
        alive = {}
        i = 0
        while True:
            config.reload()
            single_run(config, alive, update=i == 0)

            time.sleep(5)
            i = (i + 1) % 10

    except KeyboardInterrupt:
        print("Stopping...")

    except Exception as e:
        print(f"Unknown exception: {e}")
        raise e


if __name__ == "__main__":
    main()
