#!/usr/bin/env bash
#
# Script to generate Keramics test files on Linux.
#
# Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License. You may
# obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.

source ./scripts/shared_linux.sh

assert_availability_binary cryptsetup
assert_availability_binary dd
assert_availability_binary fdisk
assert_availability_binary gdisk
assert_availability_binary losetup
assert_availability_binary lvcreate
assert_availability_binary mke2fs
assert_availability_binary mkfs.xfs
assert_availability_binary mkntfs
assert_availability_binary pvcreate
assert_availability_binary qemu-img
assert_availability_binary setfattr
assert_availability_binary vgchange
assert_availability_binary vgcreate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/ext
mkdir -p test_data/gpt
mkdir -p test_data/lvm
mkdir -p test_data/luks
mkdir -p test_data/mbr
mkdir -p test_data/qcow
mkdir -p test_data/vdi
mkdir -p test_data/vhd
mkdir -p test_data/vhdx
mkdir -p test_data/vmdk
mkdir -p test_data/xfs

# Create an ext2 file system.
IMAGE_FILE="test_data/ext/ext2.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -I 128 -L ext2_test -q -t ext2 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create an ext3 file system.
IMAGE_FILE="test_data/ext/ext3.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -L ext3_test -q -t ext3 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create an ext4 file system.
IMAGE_FILE="test_data/ext/ext4.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -L ext4_test -q -t ext4 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create a GPT volume system with 2 partitions.
# * the first partition with an ext2 file system.
# * the second partition with a NTFS file system.
IMAGE_FILE="test_data/gpt/gpt.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

gdisk ${IMAGE_FILE} <<EOT
n
1
2048
+1024K
8300
n
2
4096
+1536K
0700
w
y
EOT

sudo losetup -o $(( 2048 * ${SECTOR_SIZE} )) --sizelimit $(( 1024 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

sudo losetup -o $(( 4096 * ${SECTOR_SIZE} )) --sizelimit $(( 1536 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

create_test_file_entries_with_long_file_name ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create an empty GPT volume system and a MBR volume system with 1 partition.
# * the partition is a primary partition with an ext2 file system.
IMAGE_FILE="test_data/gpt/empty_with_mbr.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

gdisk ${IMAGE_FILE} <<EOT
o
y
w
y
EOT

# Note that fdisk will write into the GPT partition entries area if the partition start offset is not set correctly.
fdisk -u ${IMAGE_FILE} <<EOT
M
d
n
p
1
48
+256K
w
EOT

sudo losetup -o $(( 48 * ${SECTOR_SIZE} )) --sizelimit $(( 256 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create a LVM volume system with 2 volumes.
# * the first volume with an ext2 file system.
IMAGE_FILE="test_data/lvm/lvm.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

sudo losetup /dev/loop99 ${IMAGE_FILE}

sudo pvcreate -q /dev/loop99 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo vgcreate -q test_volume_group /dev/loop99 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo lvcreate --name test_logical_volume1 -q --size 4m --type linear test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/test_volume_group/test_logical_volume1

sudo mount -o loop,rw /dev/test_volume_group/test_logical_volume1 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo lvcreate --name test_logical_volume2 -q --size 4m --type linear test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo vgchange --activate n -q test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo losetup -d /dev/loop99

# Create a LUKS 1 encrypted volume system with an ext2 file system.
IMAGE_FILE="test_data/luks/luks1.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

cryptsetup --batch-mode --cipher aes-cbc-plain --hash sha1 --type luks1 luksFormat ${IMAGE_FILE} <<EOT
KeramicsLuks1
EOT

sudo cryptsetup luksOpen ${IMAGE_FILE} keramics_luks <<EOT
KeramicsLuks1
EOT

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/mapper/keramics_luks

sudo mount -o loop,rw /dev/mapper/keramics_luks ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sleep 1

sudo cryptsetup luksClose keramics_luks

# Create a MBR volume system with 2 partitions.
# * the first partition is a primary partition with an ext2 file system.
# * the second partition is an extended partition with a NTFS file system.
IMAGE_FILE="test_data/mbr/mbr.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

fdisk -b ${SECTOR_SIZE} -u ${IMAGE_FILE} <<EOT
n
p
1

+1024K

n
e
2


n

+1536K
t
5
7
w
EOT

sudo losetup -o $(( 1 * ${SECTOR_SIZE} )) --sizelimit $(( 1024 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

sudo losetup -o $(( 4096 * ${SECTOR_SIZE} )) --sizelimit $(( 1536 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

create_test_file_entries_with_long_file_name ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create an EWF image with an ext2 file system.
set +e

which ewfacquire > /dev/null 2>&1
RESULT=$?

set -e

if test ${RESULT} -eq ${EXIT_SUCCESS}
then
	ewfacquire -u -c best -C case -D description -e examiner -E evidence -M logical -N notes -t test_data/ewf/ext2 test_data/ext/ext2.raw
fi

# Create a QCOW image with an ext2 file system.
IMAGE_FILE="test_data/qcow/ext2.qcow2"

qemu-img convert -f raw -O qcow2 test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a split raw image with an ext2 file system.
IMAGE_FILE="test_data/splitraw/ext2.raw."
SEGMENT_SIZE=$(( 1 * 1024 * 1024 ))

split --bytes=${SEGMENT_SIZE} --numeric-suffixes=0 --suffix-length=3 test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a VDI image with an ext2 file system.
IMAGE_FILE="test_data/vdi/ext2.vdi"

qemu-img convert -f raw -O vdi test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a VHD image with an ext2 file system.
IMAGE_FILE="test_data/vhd/ext2.vhd"

qemu-img convert -f raw -O vpc test_data/ext/ext2.raw ${IMAGE_FILE}

# Create VHDX image with an ext2 file system.
IMAGE_FILE="test_data/vhdx/ext2.vhdx"

qemu-img convert -f raw -O vhdx test_data/ext/ext2.raw ${IMAGE_FILE}

# Create VMDK image with an ext2 file system.
IMAGE_FILE="test_data/vmdk/ext2.vmdk"

qemu-img convert -f raw -O vmdk test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a XFS file system.
IMAGE_FILE="test_data/xfs/xfs.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

# Note that the environment variables are necessary to allow for a XFS file system < 300 MiB.
export TEST_DEV=1
export TEST_DIR=1
export QA_CHECK_FS=1

mkfs.xfs -b size=4096 -i size=512 -L "xfs_test" -m bigtime=0 -q -s size=${SECTOR_SIZE} ${IMAGE_FILE}

export TEST_DEV=
export TEST_DIR=
export QA_CHECK_FS=

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

exit ${EXIT_SUCCESS}
