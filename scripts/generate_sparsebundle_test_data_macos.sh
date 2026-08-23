#!/usr/bin/env bash
#
# Script to generate Keramics sparsebundle test files on Mac OS.
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

mkdir -p test_data/sparsebundle

# Create a sparsebundle with a HFS+ file system
IMAGE_FILE="test_data/sparsebundle/hfsplus"
IMAGE_SIZE="4M"

rm -rf ${IMAGE_FILE}.sparsebundle

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSEBUNDLE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparsebundle -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.sparsebundle

BASE_IMAGE_FILE="${IMAGE_FILE}.sparsebundle"

# Create an AES-128 encrypted sparsebundle with a HFS+ file system
IMAGE_FILE="test_data/sparsebundle/hfsplus_aes128"

rm -rf ${IMAGE_FILE}.sparsebundle

echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSB -stdinpass -o ${IMAGE_FILE}

# echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSP -stdinpass -tgtimagekey encrypted-encoding-version=1 -o ${IMAGE_FILE}

exit ${EXIT_SUCCESS}
