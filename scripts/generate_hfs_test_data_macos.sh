#!/usr/bin/env bash
#
# Script to generate Keramics HFS file system test files on Mac OS.
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

mkdir -p test_data/hfs

# Create a raw image with a HFS+ file system
IMAGE_FILE="test_data/hfs/hfsplus"
IMAGE_SIZE="4M"

rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type UDIF -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.dmg

exit ${EXIT_SUCCESS}
