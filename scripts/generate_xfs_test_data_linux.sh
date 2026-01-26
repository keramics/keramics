#!/usr/bin/env bash
#
# Script to generate Keramics XFS test files on Linux.
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
assert_availability_binary mkfs.xfs
assert_availability_binary setfattr
assert_availability_binary truncate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/xfs

# Create a XFS file system.
IMAGE_FILE="test_data/xfs/xfs.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

# Note that these environment variables are necessary to allow for a XFS file system < 300 MiB.
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
