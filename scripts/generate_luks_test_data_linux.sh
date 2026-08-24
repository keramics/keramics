#!/usr/bin/env bash
#
# Script to generate Keramics LUKS test files on Linux.
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

assert_availability_binary cryptsetup
assert_availability_binary dd
assert_availability_binary fallocate
assert_availability_binary mke2fs
assert_availability_binary setfattr
assert_availability_binary truncate

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/luksde

# Create a LUKS 1 encrypted volume system with an ext2 file system.
IMAGE_FILE="test_data/luksde/luks1.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

echo -n KeRaMiCs | cryptsetup --batch-mode --cipher aes-cbc-plain --hash sha1 --pbkdf-force-iterations 12345 --type luks1 luksFormat ${IMAGE_FILE} -

echo -n KeRaMiCs | sudo cryptsetup luksOpen ${IMAGE_FILE} keramics_luks -

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/mapper/keramics_luks

sudo mount -o loop,rw /dev/mapper/keramics_luks ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sleep 1

sudo cryptsetup luksClose keramics_luks

# Create a LUKS 2 encrypted volume system with an ext2 file system.
IMAGE_FILE="test_data/luksde/luks2.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

echo -n KeRaMiCs | cryptsetup --align-payload=1 --batch-mode --cipher aes-cbc-plain --hash sha1 --luks2-metadata-size=16k --luks2-keyslots-size=256k --type luks2 luksFormat ${IMAGE_FILE} -

echo -n KeRaMiCs | sudo cryptsetup luksOpen ${IMAGE_FILE} keramics_luks -

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/mapper/keramics_luks

sudo mount -o loop,rw /dev/mapper/keramics_luks ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sleep 1

sudo cryptsetup luksClose keramics_luks

exit ${EXIT_SUCCESS}
