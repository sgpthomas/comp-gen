#!/usr/bin/env bash

set -e

mkdir -p output

if which podman &>/dev/null; then
    container_manager="podman"
elif which docker &>/dev/null; then
    container_manager="docker"
else
    echo "Neither 'podman' nor 'docker' were found on the path."
    exit -1
fi

case `uname -m` in
    "arm64")
        image="ghcr.io/sgpthomas/isaria-aec-client/arm"
        ;;
    "x86_64")
        image="ghcr.io/sgpthomas/isaria-aec-client"
        ;;
    *)
        echo "Unrecognized architecture. Defaulting to x86_64"
        image="ghcr.io/sgpthomas/isaria-aec-client"
        ;;
esac

# start the container in the background
if ! podman container exists isaria; then
    echo "No isaria container found. Starting a new one"
    $container_manager container run -it -d --name isaria $image bash

    $container_manager container exec isaria git pull -r

    # copy ssh key into container
    $container_manager container exec isaria mkdir -p /home/aec/.ssh
    $container_manager container cp aec.pem isaria:/home/aec/.ssh/aec.pem
    $container_manager container exec isaria ssh-add /home/aec/.ssh/aec.pem

    # setup jobs / completed
    $container_manager container exec isaria mkdir server/jobs server/completed
fi

# run the command in the container
$container_manager container exec isaria $@

# copy results of output from container to host
$container_manager container cp isaria:output .
