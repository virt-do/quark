# From https://github.com/virt-do/lumper/blob/main/rootfs/mkrootfs.sh

DEST="alpine-minirootfs"
IMAGE_ARCHIVE_URL="https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz"
IMAGE_ARCHIVE_NAME="imageArchive"

rm -rf $DEST $IMAGE_ARCHIVE_NAME && mkdir $DEST

# Download and untar the image
curl -sSL $IMAGE_ARCHIVE_URL -o $IMAGE_ARCHIVE_NAME
tar xf $IMAGE_ARCHIVE_NAME -C $DEST
rm $IMAGE_ARCHIVE_NAME

pushd "$DEST" > /dev/null
# Create an init file in the rootfs
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
poweroff -f
EOF

chmod +x init

popd > /dev/null

# Here we do not create the rootfs image because we need to add some files in it later