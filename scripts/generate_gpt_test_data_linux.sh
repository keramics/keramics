#!/usr/bin/env bash
#
# Script to generate Keramics GPT test files on Linux.
#
# Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

assert_availability_binary dd
assert_availability_binary fallocate
assert_availability_binary fdisk
assert_availability_binary gdisk
assert_availability_binary losetup
assert_availability_binary mke2fs
assert_availability_binary mkntfs
assert_availability_binary setfattr
assert_availability_binary truncate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/gpt

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

exit ${EXIT_SUCCESS}
