#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path

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
        if tags is not None and tags[0] == name and ip is not None:
            return ip
    raise Exception(f"Machine {name} not found")


def do_upload(ip, remote_path, clean=False):
    print(f"Using ip: {ip}")
    print("Sending jobs", end="...", flush=True)
    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                "jobs/",
                f"ubuntu@{ip}:{remote_path}",
            ]
        ),
        shell=True,
    )
    print("Done")

    if clean:
        print("Cleaning jobs")
        subprocess.run(["rm", "-r", "jobs"])
        subprocess.run(["mkdir", "jobs"])


def do_download(ip, remote_path, clean=False):
    print(f"Using ip: {ip}")
    print("Downloading completed", end="...", flush=True)

    # normalize the remote path so that rsync doesn't create completed/completed/*
    remote_path = str(Path(remote_path)) + "/"

    subprocess.run(
        " ".join(
            [
                "rsync",
                "-e 'ssh -o StrictHostKeyChecking=no'",
                "-avh",
                f"ubuntu@{ip}:{remote_path}",
                "completed/",
            ]
        ),
        shell=True,
    )

    if clean:
        print("Cleaning completed")
        subprocess.run(["ssh", f"ubuntu@{ip}", "rm", "-r", remote_path])
        subprocess.run(["ssh", f"ubuntu@{ip}", "mkdir", remote_path])


@click.group()
def cli():
    pass


@cli.command()
@click.option("--name", default="exp")
@click.option("--dir", default="~/jobs")
@click.option("--clean", is_flag=True)
def upload(name, dir, clean):
    ip = get_aws_ip_by_name(name)
    do_upload(ip, dir, clean=clean)


@cli.command()
@click.option("--name", default="exp")
@click.option("--dir", default="~/completed")
@click.option("--clean", is_flag=True)
def download(name, dir, clean):
    ip = get_aws_ip_by_name(name)
    do_download(ip, dir, clean=clean)


if __name__ == "__main__":
    cli()
