#!/usr/bin/env bash

# cont="working-container"
cont=$(buildah from scratch)
contmnt=$(buildah mount $cont)

dnf install --installroot $contmnt \
    --repo fedora \
    --releasever 38 \
    coreutils bash git make cmake gcc "gcc-c++" clang-devel z3 z3-devel fontconfig-devel libjpeg-devel \
    python3-pip libnsl tar libxcrypt-compat \
    --setopt install_weak_deps=false -y

# fix the z3 install so that z3.rs can find it
buildah run $cont -- ln -sf /usr/include/z3/*.h /usr/include/

# fix ld-lsb not existing
buildah run $cont -- ln -sf /lib64/ld-linux-x86-64.so.2 /lib64/ld-lsb-x86-64.so.3

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install-rustup.sh
chmod +x /tmp/install-rustup.sh
CARGO_HOME="$contmnt/root/.cargo" RUSTUP_HOME="$contmnt/root/.rustup" /tmp/install-rustup.sh -y --no-modify-path

# install racket
wget https://download.racket-lang.org/installers/8.9/racket-8.9-x86_64-linux-cs.sh -O /tmp/install-racket.sh
chmod +x /tmp/install-racket.sh
sh /tmp/install-racket.sh --in-place --dest "$contmnt/root/racket"
rm /tmp/install-racket.sh

buildah unmount $cont

# build tools inside of the container
buildah copy $cont ./aec/install.sh /usr/bin
buildah run $cont -- chmod +x /usr/bin/install.sh
buildah run $cont -- /usr/bin/install.sh

# copy over server config
buildah copy $cont ./aec/server-config.json /root/comp-gen/server/config.json
buildah copy $cont ./aec/start-server.sh /usr/bin/start-server.sh

# set command
buildah config --cmd /usr/bin/start-server.sh $cont

# specify some metadata
buildah config \
        --created-by "sgpthomas" \
        --author "sgt@cs.utexas.edu" \
        --label org.opencontainers.image.source="https://github.com/sgpthomas/comp-gen" \
        $cont

buildah commit working-container isaria-aec
podman push isaria-aec ghcr.io/sgpthomas/isaria-aec:latest
