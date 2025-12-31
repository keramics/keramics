#!/usr/bin/env bash
#
# Script to generate Keramics ISO9660 test files on Linux.
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

assert_availability_binary genisoimage

set -e

sudo mkdir -p ${MOUNT_POINT}

mkdir -p test_data/iso9660

# Create an ISO9660 level 3 file system
IMAGE_FILE="test_data/iso9660/level3.iso"

sudo mount -o loop,rw test_data/ext/ext2.raw ${MOUNT_POINT}

genisoimage -input-charset utf8 -iso-level 3 -o ${IMAGE_FILE} ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

exit ${EXIT_SUCCESS}
