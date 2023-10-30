#!/usr/bin/env python3

import json
import os
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


def job_query(ip, remote_path_template, all):
    """
    Return information about running jobs on server running at `ip`.
    """

    jq_pattern = "{owner:.owner, name:.name, file:input_filename}"
    jq_cmd = subprocess.run(
        f'ssh ubuntu@{ip} "sh -c \'jq -c \\"{jq_pattern}\\" {remote_path_template}\'"',
        shell=True,
        capture_output=True,
    )
    jobs = [json.loads(s) for s in jq_cmd.stdout.decode("utf-8").splitlines()]

    if all:
        return jobs
    else:
        owner = f"{os.getlogin()}@{os.uname().nodename}"
        return list(filter(lambda j: j["owner"] == owner, jobs))


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


def do_download(ip, remote_path, clean=False, all=False):
    # normalize the remote path so that rsync doesn't create completed/completed/*
    remote_path = str(Path(remote_path))
    jobs = job_query(ip, f"{remote_path}/**/**/config.json", all)
    paths = [Path(j["file"]).parent for j in jobs]

    for p in paths:
        subprocess.run(
            " ".join(
                [
                    "rsync",
                    "-e 'ssh -o StrictHostKeyChecking=no'",
                    "-avh",
                    f"ubuntu@{ip}:{p}",
                    "completed",
                ]
            ),
            shell=True,
        )
        if clean:
            subprocess.run(f"ssh ubuntu@{ip} 'rm -rfv {p}'", shell=True)


def do_check(ip, remote_path, all=False):
    # normalize the remote path so that rsync doesn't create completed/completed/*
    remote_path = str(Path(remote_path))
    jobs = job_query(ip, f"{remote_path}/**/config.json", all)

    podman_status_cmd = subprocess.run(
        f'ssh ubuntu@{ip} "sh -c \'podman ps --format "{{{{.Exited}}}}" --filter name="isaria"\'"',
        shell=True,
        capture_output=True,
    )
    alive = podman_status_cmd.stdout.decode("utf-8").strip() == "false"
    in_progress = len(jobs) > 0

    if alive:
        if in_progress:
            print(f"{len(jobs)} jobs in progress")
            sys.exit(-1)
        else:
            print("Completed")
            sys.exit(0)
    else:
        print(f"Isaria experiment server not detected")
        sys.exit(-2)


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
@click.option("--all", is_flag=True)
def download(name, ip, dir, clean, all):
    ip = resolve_name_ip(name, ip)
    do_download(ip, dir, clean=clean, all=all)


@cli.command()
@click.option("--name")
@click.option("--ip")
@click.option("--dir", default="~/jobs")
@click.option("--all", is_flag=True)
def check(name, ip, dir, all):
    ip = resolve_name_ip(name, ip)
    do_check(ip, dir, all)


if __name__ == "__main__":
    cli()
