#!/usr/bin/env bash

set -e

cont=$(buildah from scratch)
contmnt=$(buildah mount $cont)

packages=(coreutils git make cmake gcc "gcc-c++" \
                    clang-devel z3 z3-devel fontconfig-devel \
                    libjpeg-devel python3-pip libnsl tar \
                    libxcrypt-compat pango-devel eigen3-devel)

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

# install packages from the cache
dnf install --installroot $contmnt \
    --repo fedora --releasever 38 \
    --setopt cachedir="$HOME/.var/cache/dnf" \
    --setopt keepcache=True \
    --setopt install_weak_deps=false -y \
    -C \
    ${packages[@]}

# fix the z3 install so that z3.rs can find it
echo "Linking /usr/include/z3/*.h -> /usr/include"
buildah run $cont -- ln -sf /usr/include/z3/z3++.h /usr/include/z3++.h
buildah run $cont -- ln -sf /usr/include/z3/z3.h /usr/include/z3.h
buildah run $cont -- ln -sf /usr/include/z3/z3_algebraic.h /usr/include/z3_algebraic.h
buildah run $cont -- ln -sf /usr/include/z3/z3_api.h /usr/include/z3_api.h
buildah run $cont -- ln -sf /usr/include/z3/z3_ast_containers.h /usr/include/z3_ast_containers.h
buildah run $cont -- ln -sf /usr/include/z3/z3_fixedpoint.h /usr/include/z3_fixedpoint.h
buildah run $cont -- ln -sf /usr/include/z3/z3_fpa.h /usr/include/z3_fpa.h
buildah run $cont -- ln -sf /usr/include/z3/z3_macros.h /usr/include/z3_macros.h
buildah run $cont -- ln -sf /usr/include/z3/z3_optimization.h /usr/include/z3_optimization.h
buildah run $cont -- ln -sf /usr/include/z3/z3_polynomial.h /usr/include/z3_polynomial.h
buildah run $cont -- ln -sf /usr/include/z3/z3_rcf.h /usr/include/z3_rcf.h
buildah run $cont -- ln -sf /usr/include/z3/z3_spacer.h /usr/include/z3_spacer.h
buildah run $cont -- ln -sf /usr/include/z3/z3_v1.h /usr/include/z3_v1.h
buildah run $cont -- ln -sf /usr/include/z3/z3_version.h /usr/include/z3_version.h
echo "Done"
buildah run $cont -- ls /usr/include/

# fix ld-lsb not existing
echo "Linking /lib64/ld-linux-x86-64.so.2 -> /lib64/ld-lsb-x86-64.so.3"
buildah run $cont -- ln -sf /lib64/ld-linux-x86-64.so.2 /lib64/ld-lsb-x86-64.so.3
echo "Done"

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install-rustup.sh
chmod +x /tmp/install-rustup.sh
CARGO_HOME="$contmnt/root/.cargo" RUSTUP_HOME="$contmnt/root/.rustup" /tmp/install-rustup.sh -y --no-modify-path

# install racket
wget https://download.racket-lang.org/installers/8.9/racket-8.9-x86_64-linux-cs.sh -O /tmp/install-racket.sh
chmod +x /tmp/install-racket.sh
sh /tmp/install-racket.sh --in-place --dest "$contmnt/root/racket"
rm /tmp/install-racket.sh

# cleanup dnf files to shrink the size of the image
dnf -y clean all --installroot $contmnt --releasever 38

# unmount container, we are done installing things into it from the outside
# the rest of the work happens inside the container
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

buildah commit $cont isaria-aec
buildah rm $cont

podman push isaria-aec ghcr.io/sgpthomas/isaria-aec:latest
