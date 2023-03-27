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
        self.env = {}
        self.jobs_at_once = 1

    def list_jobs(self):
        """Read jobs directory and produce an iterator of tasks to perform."""

        for task_dir in self.jobs.glob("*"):
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

    def commit(self):
        """Commit the current configuration to a file."""

        with self.config_path.open("w") as f:
            data = {
                "env": self.env,
                "jobs_at_once": self.jobs_at_once
            }
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
        if "timeout" in data:
            self.timeout = data["timeout"]
        else:
            self.timeout = None

    def valid(job_dir: Path):
        """Check if `job_dir` contains a valid job."""

        config = job_dir / "config.json"

        if not config.exists():
            return False

        data = json.load(config.open("r"))
        return all(map(
            lambda k: k in data,
            ["date", "name", "memory_limit", "command"]
        ))

    def exec(self):
        # prepare log files
        stdout_log = self.dir / "stdout.log"
        stderr_log = self.dir / "stderr.log"
        stdout_log.touch()
        stderr_log.touch()

        # run the comp-gen binary, storing stdout and stderr in logs
        self.proc = subprocess.Popen(
            f"{self.command}",
            env=self.global_config.env,
            stdout=stdout_log.open("w"),
            stderr=stderr_log.open("w"),
            cwd=self.dir
        )
        return self.proc

    def complete(self):
        parent_dir = self.global_config.completed / self.name
        parent_dir.mkdir(exist_ok=True)
        children = filter(
            lambda x: str(x.name).isnumeric(),
            parent_dir.glob("*")
        )
        print(list(map(lambda x: x.name, parent_dir.glob("*"))))
        results = parent_dir / str(len(list(children)))

        assert not results.exists()

        # copy over results, job config, and params.json
        shutil.copytree(self.dir, results)

        # remove the job directory
        shutil.rmtree(self.dir)

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
    return mem / float(10 ** 9)


def single_run(config, alive):
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
    for job, proc in alive.items():
        # proc is alive when poll() returns None
        if proc.poll() is None:
            memory_used = memory_used_by(proc.pid)
            print(f"{job} ({proc.pid}) {memory_used=} GB")

            # where to write the memory report
            memory_csv = job.dir / "memory.csv"

            # record memory usage
            memory_csv.touch()
            with memory_csv.open("a") as f:
                print(
                    "{},{}".format(time.time(), memory_used),
                    file=f,
                    flush=True
                )

            # if we have exceeded the memory limit, kill this
            # process and write a line out to memory.csv
            if memory_used > int(job.memory_limit):
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

    try:
        alive = {}
        while True:
            config.reload()
            single_run(config, alive)

            print(f"Alive jobs: {alive}")
            time.sleep(10)

    except KeyboardInterrupt:
        print("Stopping...")

    except Exception as e:
        print(f"Unknown exception: {e}")
        raise e


if __name__ == "__main__":
    main()
