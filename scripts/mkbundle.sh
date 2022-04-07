# From https://github.com/virt-do/kaps/blob/main/hack/mkbundle.sh

# The URL of a container image archive should be supplied in parameter
# Example with alpine : https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz
if [[ $# -ne 1 ]]; then
    echo "Missing container image URL"
    exit 1
fi

DEST="ctr-bundle"
IMAGE_ARCHIVE_URL=$1
IMAGE_ARCHIVE_NAME="imageArchive"

rm -rf $DEST $IMAGE_ARCHIVE_NAME && mkdir -p "$DEST"/rootfs

# Download and untar the image
curl -sSL $IMAGE_ARCHIVE_URL -o $IMAGE_ARCHIVE_NAME
tar xf $IMAGE_ARCHIVE_NAME -C "$DEST"/rootfs
rm $IMAGE_ARCHIVE_NAME

pushd "$DEST" > /dev/null
# Generate a runtime spec
runc spec --rootless
popd > /dev/null
