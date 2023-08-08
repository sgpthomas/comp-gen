#!/usr/bin/env bash

# cont="working-container"
cont=$(buildah from scratch)
contmnt=$(buildah mount $cont)

dnf install --installroot $contmnt \
    --releasever 38 \
    coreutils bash curl git make cmake gcc "gcc-c++" clang-devel \
    --setopt install_weak_deps=false -y

buildah unmount $cont

buildah copy $cont ./aec/install.sh /usr/bin
buildah run $cont -- chmod +x /usr/bin/install.sh
buildah run $cont -- /usr/bin/install.sh
