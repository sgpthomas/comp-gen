#!/usr/bin/env bash

# cont="working-container"
cont=$(buildah from scratch)
contmnt=$(buildah mount $cont)

dnf install --installroot $contmnt \
    --releasever 38 \
    coreutils bash git make cmake gcc "gcc-c++" clang-devel z3 font-config-devel \
    --setopt install_weak_deps=false -y

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install-rustup.sh
CARGO_HOME="$contmnt/root/.cargo" RUSTUP_HOME="$contmnt/root/.rustup" /tmp/install-rustup.sh -y --no-modify-path

# install racket
wget https://download.racket-lang.org/installers/8.9/racket-8.9-x86_64-linux-cs.sh -O /tmp/install-racket.sh
sh /tmp/install-racket.sh --in-place --dest "$contmnt/root/racket"
rm /tmp/install-racket.sh

buildah unmount $cont

buildah copy $cont ./aec/install.sh /usr/bin
buildah run $cont -- chmod +x /usr/bin/install.sh
buildah run $cont -- /usr/bin/install.sh
