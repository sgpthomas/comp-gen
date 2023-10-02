#!/usr/bin/env bash

set -e

cont=$(buildah from scratch)
contmnt=$(buildah mount $cont)

packages=(bash coreutils sudo make cmake shadow-utils \
               git R texlive-scheme-full python3-pip rsync neovim \
               openssl-devel libcurl libcurl-devel harfbuzz-devel fribidi-devel \
               udunits2-devel freetype-devel libpng-devel libtiff-devel \
               libjpeg-turbo-devel gdal gdal-devel proj proj-devel \
               geos geos-devel)


# if the -r flag is passed, redownload packages
case "$1" in
    -r|--refresh)
        if [ "$#" -gt 0 ]; then
            dnf install --installroot $contmnt \
                --repo fedora --releasever 38 \
                --setopt cachedir="$HOME/.var/cache/dnf" \
                --setopt install_weak_deps=false -y \
                --downloadonly \
                ${packages[@]}
        fi
    ;;
esac

buildah config --env TZ="Etc/UTC" $cont

# install packages from the cache
dnf install --installroot $contmnt \
    --repo fedora --releasever 38 \
    --setopt cachedir="$HOME/.var/cache/dnf" \
    --setopt keepcache=true \
    --setopt install_weak_deps=false -y \
    -C \
    ${packages[@]}

echo "Done installing packages"

# make aec user, give it sudo access, and make it the default user for the container,
# give wheel group password-less access to sudo
buildah run $cont -- useradd aec
buildah run $cont -- usermod -a -G wheel aec
buildah run $cont -- sh -c 'echo "%wheel ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers'
buildah config --user aec $cont

# configure bash prompt
buildah run $cont -- sh -c 'echo "export PS1=\"[\\u \\W]\\$ \"" >> /home/aec/.bashrc'

# enable color output
buildah run $cont -- sh -c 'echo "export TERM=xterm-256color" >> /home/aec/.bashrc'
buildah config --env TERM=xterm-256color $cont
buildah run $cont -- sh -c 'alias="ls --color=auto" >> /home/aec/.bashrc'

# clone comp-gen repo and set it as default
buildah run $cont -- git clone https://github.com/sgpthomas/comp-gen.git /home/aec/comp-gen

# set comp-gen dir as root
buildah config --workingdir /home/aec $cont

# install aws cli in case anybody wants to use it
buildah run $cont -- curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
buildah run $cont -- unzip awscliv2.zip
buildah run $cont -- sudo ./aws/install
buildah run $cont -- rm -r aws awscliv2.zip

# set comp-gen dir as root
buildah config --workingdir /home/aec/comp-gen $cont

# install R and python deps
buildah run $cont -- pip3 install click psutil pandas dfply
buildah run $cont -- sudo ./server/figs/R/install.R
