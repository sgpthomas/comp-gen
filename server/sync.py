#!/usr/bin/env python3

import json
import subprocess
import sys
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
            print(f"Resolved {name} to {ip}")
            return ip
    raise Exception(f"Machine {name} not found")


def do_upload(ip, remote_path, clean=False):
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


def resolve_name_ip(name, ip):
    """
    If ip is defined, always use ip.
    If name is defined, but ip is not defined,
    then resolve name by using awscli.

    Otherwise error out.
    """

    if ip is not None:
        return ip
    elif name is not None:
        try:
            return get_aws_ip_by_name(name)
        except FileNotFoundError:
            print(
                "`aws` was not found.\nInstall it to be able to use the --name option."
            )
            sys.exit(-1)
    else:
        raise Exception("You need to pass in either --name or --ip")


@click.group()
def cli():
    pass


@cli.command()
@click.option("--name")
@click.option("--ip")
@click.option("--dir", default="~/jobs")
@click.option("--clean", is_flag=True)
def upload(name, ip, dir, clean):
    ip = resolve_name_ip(name, ip)
    do_upload(ip, dir, clean=clean)


@cli.command()
@click.option("--name")
@click.option("--ip")
@click.option("--dir", default="~/completed")
@click.option("--clean", is_flag=True)
def download(name, ip, dir, clean):
    ip = resolve_name_ip(name, ip)
    do_download(ip, dir, clean=clean)


if __name__ == "__main__":
    cli()
