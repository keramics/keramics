#!/usr/bin/env bash
#
# Script to generate Keramics LVM test files on Linux.
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
assert_availability_binary losetup
assert_availability_binary lvcreate
assert_availability_binary mke2fs
assert_availability_binary pvcreate
assert_availability_binary setfattr
assert_availability_binary truncate
assert_availability_binary vgchange
assert_availability_binary vgcreate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/lvm

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

exit ${EXIT_SUCCESS}
