#!/usr/bin/env python3

import json
import subprocess

import click


def get_aws_ip_by_name(name):
    data = json.loads(
        subprocess.run(
            [
                "aws",
                "ec2",
                "describe-instances",
                "--query",
                "Reservations[*].Instances[].[Tags[?Key == `Name`].Value, PublicIpAddress]",
                "--no-cli-pager",
            ],
            capture_output=True,
        ).stdout
    )
    for tags, ip in data:
        if tags[0] == name and ip is not None:
            return ip
    raise Exception(f"Machine {name} not found")


def do_send(name, remote_path="~"):
    ip = get_aws_ip_by_name(name)
    print(f"Using ip: {ip}")
    # send the complete jobs to the server so that the number's it generates
    # are the same as the ones we have
    print("Sending complete jobs", end="...", flush=True)
    print(f"remote path: {remote_path}")
    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                "--exclude",
                "*.log",
                "--exclude",
                "*.rkt",
                "completed/",
                f"ubuntu@{ip}:{remote_path}/completed",
            ]
        ),
        shell=True,
    )
    print("Done")
    print("Sending jobs", end="...", flush=True)
    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                "--exclude",
                "*.rkt",
                "jobs/",
                f"ubuntu@{ip}:{remote_path}/jobs",
            ]
        ),
        shell=True,
    )
    print("Done")
    print("Cleaning jobs")
    subprocess.run(["rm", "-r", "jobs"])
    subprocess.run(["mkdir", "jobs"])


def do_retreive(name, remote_path="~"):
    ip = get_aws_ip_by_name(name)

    print("Syncing in-progress jobs", end="...")
    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                "--delete",
                f"ubuntu@{ip}:{remote_path}/jobs/",
                "in-progress",
            ]
        ),
        shell=True,
    )
    print("Done")

    print("Syncing completed", end="...")
    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                f"ubuntu@{ip}:{remote_path}/completed/",
                "completed",
            ]
        ),
        shell=True,
    )
    print("Done")


def do_both(name, remote_path):
    do_send(name, remote_path)
    do_retreive(name, remote_path)


@click.group()
def cli():
    pass


@cli.command()
@click.option("--name", default="exp")
@click.option("--dir", default="~/comp-gen/server")
def send(name, dir):
    do_send(name, remote_path=dir)


@cli.command()
@click.option("--name", default="exp")
@click.option("--dir", default="~/comp-gen/server")
def retreive(name, dir):
    do_retreive(name, remote_path=dir)


@cli.command()
@click.option("--name", default="exp")
@click.option("--dir", default="~/comp-gen/server")
def both(name, dir):
    do_both(name, remote_path=dir)


if __name__ == "__main__":
    cli()
