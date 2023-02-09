from pathlib import Path
import shutil
import json
import subprocess


DEFAULT_CONFIG = {
    "compgen_bin": "dios-lang",
    "dios_bin": "diospyros/dios",
    "dios_example_bin": "diospyros/dios-example-gen"
}


class Task:
    def __init__(self, global_config, completed: Path, task_dir: Path):
        assert task_dir.exists()
        assert (task_dir / "params.json").exists()
        assert (task_dir / "rules.json").exists()
        assert (task_dir / "compile.json").exists()

        self.global_config = global_config
        self.completed = completed
        self.task_dir = task_dir
        self.config = json.load((task_dir / "config.json").open("r"))
        self.benchmark = self.config["benchmark"]
        self.params_path = task_dir / "params.json"
        self.rules_path = task_dir / "rules.json"
        self.config_path = task_dir / "compile.json"
        self.output_dir = task_dir / "results"

    def valid(task_dir):
        """Check if `task_dir` has the right files to be a task."""

        dir_exists = task_dir.exists()
        params_b = (task_dir / "params.json").exists()
        rules_b = (task_dir / "rules.json").exists()
        config_b = (task_dir / "compile.json").exists()
        return dir_exists and params_b and rules_b and config_b

    def exec(self):
        subprocess.run([
            self.global_config["compgen_bin"],
            "compile", self.benchmark,
            "--dios-bin", self.global_config["dios_bin"],
            "--dios-example-bin", self.global_config["dios_example_bin"],
            "--dios-params", str(self.params_path),
            "--vector-width", "4",
            "--rules", str(self.rules_path),
            "--config", str(self.config_path),
            "--output-dir", str(self.output_dir)
        ], env={
            "RUST_LOG": "debug,egg=info"
        })
        print("=~=~=~= done! =~=~=~=")

    def complete(self):
        completed_dir = self.completed / self.task_dir.name
        completed_dir.mkdir(exist_ok=False)
        shutil.copy(self.output_dir, self.params_path, completed_dir)

    def __repr__(self):
        return f"<Task bench={self.benchmark}>"


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
            print(task)
            jobs += [task]

    return jobs


def commit_config(config):
    """Write out the config to a file."""

    with Path("config.json").open("w") as f:
        json.dump(config, f, indent=2)


def main():
    (completed, jobs) = setup()

    config_path = Path("config.json")
    if config_path.exists():
        with config_path.open("r") as f:
            config = json.load(f)
    else:
        config = DEFAULT_CONFIG

    print(config)
    for job in list_jobs(config, completed, jobs):
        # job.exec()
        job.complete()

    commit_config(config)


if __name__ == "__main__":
    main()
