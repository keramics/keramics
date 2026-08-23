#!/usr/bin/env bash
#
# Script to generate Keramics sparseimage test files on Mac OS.
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

source ./scripts/shared_macos.sh

assert_availability_binary diskutil
assert_availability_binary hdiutil
assert_availability_binary mkfifo
assert_availability_binary mknod
assert_availability_binary sw_vers

set -e

mkdir -p test_data/sparseimage

# Create a sparseimage with a HFS+ file system
IMAGE_FILE="test_data/sparseimage/hfsplus"
IMAGE_SIZE="4M"

rm -f ${IMAGE_FILE}.sparseimage

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparseimage -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.sparseimage

BASE_IMAGE_FILE="${IMAGE_FILE}.sparseimage"

# Create an AES-128 encrypted sparseimage with a HFS+ file system
IMAGE_FILE="test_data/sparseimage/hfsplus_aes128"

rm -f ${IMAGE_FILE}.sparseimage

echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSP -stdinpass -o ${IMAGE_FILE}

# echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSP -stdinpass -tgtimagekey encrypted-encoding-version=1 -o ${IMAGE_FILE}

exit ${EXIT_SUCCESS}
