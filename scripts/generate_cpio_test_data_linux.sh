#!/usr/bin/env bash
#
# Script to generate Keramics cpio test files on Linux.
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

assert_availability_binary cpio

set -e

sudo mkdir -p "${MOUNT_POINT}"

mkdir -p test_data/cpio

BASE_IMAGE_FILE="test_data/ext/ext2.raw"

if [ -f "${BASE_IMAGE_FILE}" ]
then
    sudo mount -o loop,ro "${BASE_IMAGE_FILE}" "${MOUNT_POINT}"

    # TODO: generate both a big-endian and little-endian "bin" cpio archive file.

    # Create a binary format ("bin") cpio archive file.
    ARCHIVE_FILE="test_data/cpio/bin.cpio"

    find "${MOUNT_POINT}" -depth | cpio -o -H bin > ${ARCHIVE_FILE}

    # Create a POSIX.1 portable format ("odc") cpio archive file.
    ARCHIVE_FILE="test_data/cpio/odc.cpio"

    find "${MOUNT_POINT}" -depth | cpio -o -H odc > ${ARCHIVE_FILE}

    # Create a SVR4 portable format ("newc") with checksum (Sum32) cpio archive file.
    ARCHIVE_FILE="test_data/cpio/newc.cpio"

    find "${MOUNT_POINT}" -depth | cpio -o -H crc > ${ARCHIVE_FILE}

    # TODO: hpbin
    # TODO: hpodc

    sudo umount "${MOUNT_POINT}"
fi

exit ${EXIT_SUCCESS}
