#!/usr/bin/env bash
#
# Script to generate Keramics test files on Linux.
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
assert_availability_binary mke2fs
assert_availability_binary mkntfs
assert_availability_binary qemu-img
assert_availability_binary setfattr
assert_availability_binary truncate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/ext
mkdir -p test_data/qcow
mkdir -p test_data/vdi
mkdir -p test_data/vhd
mkdir -p test_data/vhdx
mkdir -p test_data/vmdk

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

exit ${EXIT_SUCCESS}
