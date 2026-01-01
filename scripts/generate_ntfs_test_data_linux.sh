#!/usr/bin/env bash
#
# Script to generate Keramics NTFS file system test files on Linux.
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
assert_availability_binary mkntfs

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/ntfs

# Create a NTFS file system.
IMAGE_FILE="test_data/ntfs/ntfs.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

create_test_file_entries_with_long_file_name ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

exit ${EXIT_SUCCESS}
