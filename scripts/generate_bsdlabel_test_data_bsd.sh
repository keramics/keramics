#!/usr/bin/env bash
#
# Script to generate Keramics BSD disklabel test files on FreeBSD.
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

source ./scripts/shared_bsd.sh

assert_availability_binary bsdlabel
assert_availability_binary dd
assert_availability_binary mdconfig
assert_availability_binary newfs

set -e

mkdir -p ${MOUNT_POINT}

mkdir -p test_data/bsdlabel

# Create an image with a BSD disklabel and an UFS file system
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))
SECTOR_SIZE=512

IMAGE_FILE="test_data/bsdlabel/bsdlabel.raw"

echo "Generating: ${IMAGE_FILE}"

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mdconfig -a -t vnode -f ${IMAGE_FILE} -u 9

bsdlabel -w -B md9 auto

newfs -L ufs1_test -O 1 md9a

mount /dev/md9a ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

umount ${MOUNT_POINT}

mdconfig -d -u 9

exit ${EXIT_SUCCESS}
