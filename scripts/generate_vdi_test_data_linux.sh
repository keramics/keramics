#!/usr/bin/env bash
#
# Script to generate Keramics VDI test files on Linux.
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

assert_availability_binary qemu-img

set -e

mkdir -p test_data/vdi

BASE_IMAGE_FILE="test_data/ext/ext2.raw"

if [ -f "${BASE_IMAGE_FILE}" ]
then
    # Create a VDI image with an ext2 file system.
    IMAGE_FILE="test_data/vdi/ext2.vdi"

    qemu-img convert -f raw -O vdi "${BASE_IMAGE_FILE}" "${IMAGE_FILE}"
fi

exit ${EXIT_SUCCESS}
