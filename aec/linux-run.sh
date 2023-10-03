#!/usr/bin/env bash

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
        image="ghcr.io/sgpthomas/isaria-aec-client/arm"
        ;;
    *)
        echo "Unrecognized architecture. Defaulting to x86_64"
        image="ghcr.io/sgpthomas/isaria-aec-client/arm"
        ;;
esac

$container_manager run -v "./output:/home/aec/comp-gen/output:U" $image $@
