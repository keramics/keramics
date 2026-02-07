#!/usr/bin/env bash
#
# Script to generate Keramics HFS file system test files on Linux.
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

assert_availability_binary dd
assert_availability_binary hformat
assert_availability_binary mkfs.hfsplus

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/hfs

set +e

sudo modprobe hfs
if [ $? -eq 0 ];
then
	set -e

	# Create a HFS standard file system.
	IMAGE_FILE="test_data/hfs/hfs.raw"
	IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
	SECTOR_SIZE=512

	dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

	hformat -f -l "hfs_test" ${IMAGE_FILE} 0

	sudo mount -o loop,rw,gid=${CURRENT_GID},uid=${CURRENT_UID} ${IMAGE_FILE} ${MOUNT_POINT}

	sudo chown ${USERNAME} ${MOUNT_POINT}

	create_test_file_entries ${MOUNT_POINT}

	sudo umount ${MOUNT_POINT}

	set +e
fi

sudo modprobe hfsplus
if [ $? -eq 0 ];
then
	set -e

	# Create a HFS+ file system.

	IMAGE_FILE="test_data/hfs/hfsplus.raw"
	IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
	SECTOR_SIZE=512

	dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

	mkfs.hfsplus -v "hfsplus_test" ${IMAGE_FILE}

	sudo mount -o loop,rw,gid=${CURRENT_GID},uid=${CURRENT_UID} ${IMAGE_FILE} ${MOUNT_POINT}

	sudo chown ${USERNAME} ${MOUNT_POINT}

	create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

	sudo umount ${MOUNT_POINT}

	set +e
fi

exit ${EXIT_SUCCESS}
