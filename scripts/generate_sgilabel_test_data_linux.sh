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
assert_availability_binary losetup
assert_availability_binary mke2fs
assert_availability_binary mkntfs
assert_availability_binary setfattr
assert_availability_binary truncate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/sgilabel

# Create a sgilabel.
IMAGE_FILE="test_data/sgilabel/sgilabel.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

fdisk -C 8 -H 16 -S 63 ${IMAGE_FILE} <<EOF
x
g
r
n
1

+1024K
w
EOF

sudo losetup -o $(( 5040 * ${SECTOR_SIZE} )) --sizelimit $(( 1024 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

exit ${EXIT_SUCCESS}
