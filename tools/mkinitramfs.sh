#!/usr/bin/bash
DEST=${1:-"alpine-minirootfs"}

pushd "$DEST"
cat > init <<EOF
#! /bin/sh
#
# /init executable file in the initramfs 
#
mount -t devtmpfs dev /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys
ip link set up dev lo

exec /opt/kaps run --bundle /ctr-bundle
# exec /sbin/getty -n -l /bin/sh 115200 /dev/console
# poweroff -f
EOF

chmod +x init

find . -print0 |
    cpio --null --create --verbose --owner root:root --format=newc |
    xz -9 --format=lzma  > ../initramfs.img

popd