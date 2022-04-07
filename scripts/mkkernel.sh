# From https://github.com/virt-do/lumper/blob/main/kernel/mkkernel.sh

CONFIG_URL="https://raw.githubusercontent.com/virt-do/lumper/main/kernel/linux-config-x86_64"
LINUX_REPO=linux-cloud-hypervisor

if [ ! -d $LINUX_REPO ]
then
    echo "Cloning linux-cloud-hypervisor git repository..."
    git clone --depth 1 "https://github.com/cloud-hypervisor/linux.git" -b "ch-5.14" $LINUX_REPO
fi

pushd $LINUX_REPO > /dev/null
pwd
wget -qO .config $CONFIG_URL
make bzImage -j `nproc`
popd > /dev/null
