#!/usr/bin/env python3

import os
import subprocess
import json
import time
import glob
import sys
from datetime import datetime
from pathlib import Path


def configure_new_trial():
    today = datetime.now()
    d1 = today.strftime("%m-%d-%H%M")
    print(f"Configuring new trial: data/{d1}")
    data_dir = Path(f"data/{d1}")
    data_dir.mkdir(parents=True, exist_ok=True)

    dios_configs = glob.glob("configs/config_c*_sr*_us*_sf*.json")
    synth_configs = glob.glob("synth_configs/abs_timeout_*.json")

    experiments = []
    for sc in synth_configs:
        for dc in dios_configs:
            experiments.append(f"./run.sh synth out.json --config {dc} --ruler {sc}")

    status = {
        "machines": [],
        "remaining": experiments,
        "count": 0,
        "finished": []
    }
    with open(data_dir / "status.json", "w") as f:
        json.dump(status, f, indent=2)
    

def launch_aws_machines(name_prefix, count=1):
    results = []
    for n in range(count):
        name = f"{{Key=Name,Value={name_prefix}_{n}}}"
        cmd = " ".join([
            "aws", "ec2", "run-instances",
            "--image-id", "ami-0b9702e87d5fdd741",
            "--security-group-ids", "sg-0da7694c867e84ae3",
            "--key-name", "thelio",
            "--count", "1",
            "--instance-type", "r5.4xlarge",
            "--no-cli-pager",
            "--tag-specifications", "'ResourceType=instance,Tags=[{}]'".format(name),
        ])
        
        try:
            x = json.load(os.popen(cmd))
            results.append(x)
        except json.decoder.JSONDecodeError:
            print("Failed to launch this machine")
        # x = cmd
    return results


def describe_machines(name):
    cmd = " ".join([
        "aws", "ec2", "describe-instances",
        "--query", "'Reservations[*].Instances[].[Tags[?Key == `Name`].Value | [0], InstanceType, InstanceId, State.Name, PublicIpAddress]'"
    ])
    result = json.load(os.popen(cmd))
    running = filter(lambda x: x[0].startswith(name) and x[3] in [ "running" ], result)
    return map(
        lambda x: {"name": x[0], "type": x[1], "id": x[2], "state": x[3], "ip": x[4], "files": False, "experiment": None},
        running
    )


def launch():
    data = Path(sys.argv[2])
    assert(data.exists())
    config = json.load(open(data / "status.json"))

    print("Launching machines...")
    results = launch_aws_machines(data.name, count=int(sys.argv[3]))
    print("Finding machines...")
    for i in range(5):
        print(f"Attempt {i}")
        machines = list(describe_machines(data.name))
        if len(machines) == len(results):
            config["machines"] = machines
            with open(data / "status.json", "w") as f:
                json.dump(config, f, indent=2)
            break
        else:
            time.sleep(2)


def reset_machines():
    data = Path(sys.argv[2])
    assert(data.exists())
    config = json.load(open(data / "status.json"))

    for m in config["machines"]:
        if m["experiment"] is not None:
            print("I need all the machines to be quiet to reset.")
            sys.exit(-1)

    new_machines = list(describe_machines(data.name))
    config["machines"] = new_machines
    with open(data / "status.json", "w") as f:
        json.dump(config, f, indent=2)


def setup():
    data = Path(sys.argv[2])
    assert(data.exists())
    config = json.load(open(data / "status.json"))

    print("Compiling...")
    os.system("cd ../../dios-lang && cargo build --release")

    machines = config["machines"]
    for m in machines:
        if m["state"] == "running" and not m["files"]:
            print(f"Setting up {m['name']}...")
            rsync_cmd = "rsync -e 'ssh -o StrictHostKeyChecking=no' -vP"
            try:
                subprocess.check_output(
                    f"{rsync_cmd} -r configs ubuntu@{m['ip']}:/home/ubuntu/",
                    shell=True
                )
                subprocess.check_output(
                    f"{rsync_cmd} -r synth_configs ubuntu@{m['ip']}:/home/ubuntu/",
                    shell=True
                )
                subprocess.check_output(
                    f"{rsync_cmd} run.sh ubuntu@{m['ip']}:/home/ubuntu/",
                    shell=True
                )
                subprocess.check_output(
                    f"{rsync_cmd} ../../dios-lang/target/release/dios-lang ubuntu@{m['ip']}:/home/ubuntu/",
                    shell=True
                )
                m["files"] = True
                with open(data / "status.json", "w") as f:
                    json.dump(config, f, indent=2)
            except subprocess.SubprocessError:
                print(f"Error for {m}")


def run():
    data = Path(sys.argv[2])
    assert(data.exists())
    config = json.load(open(data / "status.json"))

    machines = config["machines"]
    remaining = config["remaining"]

    # launch experiments
    for m in machines:
        if m["files"] and len(remaining) > 0 and m["experiment"] is None:
            print(f"Launching experiment on {m['name']}...")
            cmd = f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} '{remaining[0]}'"
            print(cmd)
            if os.system(cmd) == 0:
                m["experiment"] = remaining.pop(0)

            with open(data / "status.json", "w") as f:
                json.dump(config, f, indent=2)


def check():
    data = Path(sys.argv[2])
    assert(data.exists())
    config = json.load(open(data / "status.json"))

    for m in config["machines"]:
        if m["experiment"] is not None:
            print(f"Checking machine: {m['name']}")
            # check to see if dios-lang is running
            try:
                cmd = f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} 'pgrep -x dios-lang'"
                pgrep = subprocess.check_output(cmd, shell=True, timeout=5)

                cmd = f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} 'tail stderr.log'"
                log = subprocess.check_output(cmd, shell=True, timeout=5)
                print("=== stderr.log ===")
                print(log.decode("utf-8"))
            except subprocess.TimeoutExpired:
                # skip this interation
                continue
            except subprocess.CalledProcessError:
                # dios lang is not running
                cmd = f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} 'ls out.json'"
                ret = os.system(cmd)
                if ret == 0:
                    # the results exist!
                    name = data / str(config['count'])
                    rsync_cmd = "rsync -e 'ssh -o StrictHostKeyChecking=no' -vP"
                    os.system(f"{rsync_cmd} ubuntu@{m['ip']}:/home/ubuntu/out.json {name}.json")
                    os.system(f"{rsync_cmd} -vP ubuntu@{m['ip']}:/home/ubuntu/stdout.log {name}_stdout.log")
                    os.system(f"{rsync_cmd} -vP ubuntu@{m['ip']}:/home/ubuntu/stderr.log {name}_stderr.log")
                    os.system(f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} 'rm out.json'")
                    config['count'] += 1
                    config["finished"].append({"name": f"{name.name}.json", "config": m["experiment"]})
                    m["experiment"] = None
                else:
                    # no results exist :(. process was probably oom killed.
                    config['count'] += 1
                    config["finished"].append({"name": None, "config": m["experiment"], "oom_killed": True})
                    m["experiment"] = None

                # commit config
                with open(data / "status.json", "w") as f:
                    json.dump(config, f, indent=2)
                
            # pgrep = os.system(f"ssh -o StrictHostKeyChecking=no ubuntu@{m['ip']} 'pgrep -x dios-lang'")
            # print(f"{pgrep=}")
            # if int(pgrep.strip()) == 256:
                    


def watch():
    data = Path(sys.argv[2])
    assert(data.exists())

    config = json.load(open(data / "status.json"))
    while len(config["remaining"]) != 0 \
          or any([m["experiment"] is not None for m in config["machines"]]):
        print("Trying check...")
        check()
        print("Trying run...")
        run()
        config = json.load(open(data / "status.json"))
        print("Waiting 10...")
        time.sleep(10)
    print("Done!!")


def main():
    # configure a new trial
    if len(sys.argv) > 1 and sys.argv[1] == "new":
        configure_new_trial()

    # launch machines for a trial
    elif len(sys.argv) > 3 and sys.argv[1] == "launch":
        launch()

    # update machine list
    elif len(sys.argv) > 2 and sys.argv[1] == "reset-machines":
        reset_machines()

    # copy over necessary files to machines
    elif len(sys.argv) > 2 and sys.argv[1] == "setup":
        setup()

    # run experiments
    elif len(sys.argv) > 2 and sys.argv[1] == "run":
        run()

    # check experiments
    elif len(sys.argv) > 2 and sys.argv[1] == "check":
        check()

    # watch experiments
    elif len(sys.argv) > 2 and sys.argv[1] == "watch":
        watch()

    else:
        print(f"Usage: {sys.argv[0]} [new|launch <data_dir> <count>]")
        sys.exit(-1)


if __name__ == "__main__":
    main()
