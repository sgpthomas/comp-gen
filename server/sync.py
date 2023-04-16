#!/usr/bin/env python3

import subprocess
import click
import json


def get_aws_ip_by_name(name):
    data = json.loads(subprocess.run([
        "aws", "ec2",
        "describe-instances",
        "--query",
        "Reservations[*].Instances[].[Tags[?Key == `Name`].Value, PublicIpAddress]",
        "--no-cli-pager"
    ], capture_output=True).stdout)
    for (tags, ip) in data:
        if tags[0] == name and ip is not None:
            return ip
    raise Exception(f"Machine {name} not found")


def do_send(name):
    ip = get_aws_ip_by_name(name)
    # send the complete jobs to the server so that the number's it generates
    # are the same as the ones we have
    print("Sending complete jobs", end="...")
    subprocess.run(" ".join([
        "rsync", "-e 'ssh -o StrictHostKeyChecking=no'",
        "-avh",
        "--exclude", "*.log",
        "--exclude", "*.rkt",
        "completed/",
        f"ubuntu@{ip}:~/comp-gen/server/completed"
    ]), shell=True)
    print("Done")
    print("Sending jobs", end="...")
    subprocess.run(" ".join([
        "rsync", "-e 'ssh -o StrictHostKeyChecking=no'",
        "-avh",
        "--exclude", "*.rkt",
        "jobs/",
        f"ubuntu@{ip}:~/comp-gen/server/jobs"
    ]), shell=True)
    print("Done")
    print("Cleaning jobs")
    subprocess.run(["rm", "-r", "jobs"])
    subprocess.run(["mkdir", "jobs"])


def do_retreive(name):
    ip = get_aws_ip_by_name(name)

    print("Syncing in-progress jobs", end="...")
    subprocess.run(" ".join([
        "rsync", "-e 'ssh -o StrictHostKeyChecking=no'",
        "-avh", "--delete",
        f"ubuntu@{ip}:~/comp-gen/server/jobs/",
        "in-progress",
    ]), shell=True)
    print("Done")

    print("Syncing completed", end="...")
    subprocess.run(" ".join([
        "rsync", "-e 'ssh -o StrictHostKeyChecking=no'",
        "-avh",
        f"ubuntu@{ip}:~/comp-gen/server/completed/",
        "completed",
    ]), shell=True)
    print("Done")


def do_both(name):
    do_send(name)
    do_retreive(name)


@click.group()
def cli():
    pass


@cli.command()
@click.option("--name", default="exp")
def send(name):
    do_send(name)


@cli.command()
@click.option("--name", default="exp")
def retreive(name):
    do_retreive(name)


@cli.command()
@click.option("--name", default="exp")
def both(name):
    do_both(name)


if __name__ == "__main__":
    cli()
