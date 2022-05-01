#!/bin/bash

set -e

mkdir -p build
cd build

# kaps
if [[ ! -d "kaps" ]]; then
    git clone https://github.com/virt-do/kaps.git
else
    echo "kaps already cloned"
fi

echo "Building kaps"
cd kaps/hack
if [[ ! -d "ctr-bundle" ]]; then
    ./mkbundle.sh
else 
    echo "kaps bundle already exists"
fi
cd ..
cargo build --release --target=x86_64-unknown-linux-musl
cd ..
echo "kaps done"

# lumper
if [[ ! -d "lumper" ]]; then
    git clone https://github.com/virt-do/lumper.git
else
    echo "lumper already exists"
fi

echo "Building lumper"
cd lumper/kernel
if [[ ! -d "linux-cloud-hypervisor" ]]; then
    sudo ./mkkernel.sh
else
    echo "kernel already built"
fi
cd ../rootfs
if [[ ! -d "alpine-minirootfs" ]]; then
    sudo ./mkrootfs.sh
else
    echo "rootfs already built"
fi
cd ..
cargo build --release
cd ..
echo "lumper done"
