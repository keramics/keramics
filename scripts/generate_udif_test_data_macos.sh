#!/usr/bin/env bash
#
# Script to generate Keramics UDIF test files on Mac OS.
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

mkdir -p test_data/udif

BASE_IMAGE_FILE="test_data/hfs/hfsplus.dmg"

if [ -f "${BASE_IMAGE_FILE}" ]
then
    # Create an ADC compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_adc"

    rm -f ${IMAGE_FILE}.dmg

    hdiutil convert ${BASE_IMAGE_FILE} -format UDCO -o ${IMAGE_FILE}

    # Create a bzip2 compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_bzip2"

    rm -f ${IMAGE_FILE}.dmg

    hdiutil convert ${BASE_IMAGE_FILE} -format UDBZ -o ${IMAGE_FILE}

    # Create a lzfse compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_lzfse"

    rm -f ${IMAGE_FILE}.dmg

    hdiutil convert ${BASE_IMAGE_FILE} -format ULFO -o ${IMAGE_FILE}

    # Create a lzma compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_lzma"

    rm -f ${IMAGE_FILE}.dmg

    hdiutil convert ${BASE_IMAGE_FILE} -format ULMO -o ${IMAGE_FILE}

    # Create a zlib compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_zlib"

    rm -f ${IMAGE_FILE}.dmg

    hdiutil convert ${BASE_IMAGE_FILE} -format UDZO -o ${IMAGE_FILE}

    # Create a zlib compressed UDIF image with a resource fork.
    # Note this works with older versions of hdiutil that support flatten/unflatten.
    #
    # IMAGE_FILE="test_data/udif/hfsplus_rsrc"
    #
    # rm -f ${IMAGE_FILE}.dmg
    #
    # hdiutil convert ${BASE_IMAGE_FILE} -format UDZO -o ${IMAGE_FILE}
    #
    # hdiutil unflatten test.dmg
    # hdiutil flatten -noxml test.dmg

    # Create an AES-128 encrypted zlib compressed UDIF image.
    IMAGE_FILE="test_data/udif/hfsplus_zlib_aes128"

    rm -f ${IMAGE_FILE}.dmg

    echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDZO -stdinpass -o ${IMAGE_FILE}

    # echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDZO -stdinpass -tgtimagekey encrypted-encoding-version=1 -o ${IMAGE_FILE}

    # Create an uncompressed segmented UDIF image.
    #
    # IMAGE_FILE="test_data/udif/hfsplus_segments"
    # IMAGE_SIZE="4M"
    #
    # hdiutil attach -nomount test_data/udif/hfsplus_zlib
    # sudo hdiutil create -srcdevice /dev/rdisk# -format UDIF -segmentSize 10K ${IMAGE_FILE}

    # Create a zlib compressed segmented UDIF image.
    #
    # IMAGE_FILE="test_data/udif/hfsplus_zlib_segments"
    # IMAGE_SIZE="4M"
    #
    # hdiutil attach -nomount test_data/udif/hfsplus_zlib
    # sudo hdiutil create -srcdevice /dev/rdisk# -format UDZO -segmentSize 10K ${IMAGE_FILE}
fi

exit ${EXIT_SUCCESS}
