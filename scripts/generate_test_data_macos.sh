#!/usr/bin/env bash
#
# Script to generate Keramics test files on Mac OS.
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

EXIT_SUCCESS=0
EXIT_FAILURE=1

AFSCTOOL=`which afsctool`

# Checks the availability of a binary and exits if not available.
#
# Arguments:
#   a string containing the name of the binary
#
assert_availability_binary()
{
    local BINARY=$1

    which ${BINARY} > /dev/null 2>&1
    if test $? -ne ${EXIT_SUCCESS}
    then
        echo "Missing binary: ${BINARY}"
        echo ""

        exit ${EXIT_FAILURE}
    fi
}

# Creates file entries.
#
# Arguments:
#   a string containing the mount point
#
create_file_entries()
{
    MOUNT_POINT=$1

    # Create an empty file
    touch ${MOUNT_POINT}/emptyfile

    # Create a directory
    mkdir ${MOUNT_POINT}/testdir1

    # Create a file that can be stored as inline data
    echo "Keramics" > ${MOUNT_POINT}/testdir1/testfile1

    # Create a file that cannot be stored as inline data
    cp LICENSE ${MOUNT_POINT}/testdir1/TestFile2

    # Create a hard link to a file
    ln ${MOUNT_POINT}/testdir1/testfile1 ${MOUNT_POINT}/file_hardlink1

    # Create a symbolic link to a file
    ln -s ${MOUNT_POINT}/testdir1/testfile1 ${MOUNT_POINT}/file_symboliclink1

    # Create a symbolic link to a directory
    ln -s ${MOUNT_POINT}/testdir1 ${MOUNT_POINT}/directory_symboliclink1

    # Create a file with an UTF-8 NFC encoded filename
    touch `printf "${MOUNT_POINT}/nfc_t\xc3\xa9stfil\xc3\xa8"`

    # Create a file with an UTF-8 NFD encoded filename
    touch `printf "${MOUNT_POINT}/nfd_te\xcc\x81stfile\xcc\x80"`

    # Create a file with an UTF-8 NFD encoded filename
    touch `printf "${MOUNT_POINT}/nfd_\xc2\xbe"`

    # Create a file with an UTF-8 NFKD encoded filename
    touch `printf "${MOUNT_POINT}/nfkd_3\xe2\x81\x844"`

    # Create a file with filename that requires case folding if
    # the file system is case-insensitive
    touch `printf "${MOUNT_POINT}/case_folding_\xc2\xb5"`

    # Create a file with a forward slash in the filename
    touch `printf "${MOUNT_POINT}/forward:slash"`

    # Create a symbolic link to a file with a forward slash in the filename
    ln -s ${MOUNT_POINT}/forward:slash ${MOUNT_POINT}/file_symboliclink2

    # Create a file with a resource fork with content
    touch ${MOUNT_POINT}/testdir1/resourcefork1
    echo "My resource fork" > ${MOUNT_POINT}/testdir1/resourcefork1/..namedfork/rsrc

    # Create a file with an extended attribute with content
    touch ${MOUNT_POINT}/testdir1/xattr1
    xattr -w myxattr1 "My 1st extended attribute" ${MOUNT_POINT}/testdir1/xattr1

    # Create a directory with an extended attribute with content
    mkdir ${MOUNT_POINT}/testdir1/xattr2
    xattr -w myxattr2 "My 2nd extended attribute" ${MOUNT_POINT}/testdir1/xattr2

    # Create a file with an extended attribute that is not stored inline
    read -d "" -n 8192 -r LARGE_XATTR_DATA < LICENSE
    touch ${MOUNT_POINT}/testdir1/large_xattr
    xattr -w mylargexattr "${LARGE_XATTR_DATA}" ${MOUNT_POINT}/testdir1/large_xattr

    if test -x ${AFSCTOOL}
    then
        # Create a file that uses HFS+ compression (decmpfs) compression method 3
        echo "My compressed file" > ${MOUNT_POINT}/testdir1/compressed1
        ${AFSCTOOL} -c -T ZLIB ${MOUNT_POINT}/testdir1/compressed1

        # Create a file that uses HFS+ compression (decmpfs) compression method 4
        ditto --nohfsCompression LICENSE ${MOUNT_POINT}/testdir1/compressed2
        ${AFSCTOOL} -c -T ZLIB ${MOUNT_POINT}/testdir1/compressed2

        # Create a file that uses HFS+ compression (decmpfs) compression method 7
        echo "My compressed file" > ${MOUNT_POINT}/testdir1/compressed3
        ${AFSCTOOL} -c -T LZVN ${MOUNT_POINT}/testdir1/compressed3

        # Create a file that uses HFS+ compression (decmpfs) compression method 8
        ditto --nohfsCompression LICENSE ${MOUNT_POINT}/testdir1/compressed4
        ${AFSCTOOL} -c -T LZVN ${MOUNT_POINT}/testdir1/compressed4

        # Create a file that uses HFS+ compression (decmpfs) compression method 11
        echo "My compressed file" > ${MOUNT_POINT}/testdir1/compressed5
        ${AFSCTOOL} -c -T LZFSE ${MOUNT_POINT}/testdir1/compressed5

        # Create a file that uses HFS+ compression (decmpfs) compression method 12
        ditto --nohfsCompression LICENSE ${MOUNT_POINT}/testdir1/compressed6
        ${AFSCTOOL} -c -T LZFSE ${MOUNT_POINT}/testdir1/compressed6
    fi

    # Note that compressed UDIF images don't allow for block or character device files.

    # Create a pipe (FIFO) file
    mkfifo ${MOUNT_POINT}/testdir1/pipe1
}

detach_image()
{
    DISK_NODE=$(hdiutil info | awk -v img="$1" '
        $0 ~ img {found=1}
        found && /^\/dev\/disk[0-9]+/ {sub(/^\/dev\//, "", $1); print $1; exit}
    ')
    set +e

    sync

    for ((attempt=1; attempt<=5; attempt++))
    do
        hdiutil detach "${DISK_NODE}" -force 2>/dev/null

        if test $? -eq 0
        then
            break
        fi
        echo ""
        echo "${DISK_NODE} busy, waiting 10 seconds."
        sleep 10
    done

    set -e
}

assert_availability_binary diskutil
assert_availability_binary hdiutil
assert_availability_binary mkfifo
assert_availability_binary mknod
assert_availability_binary sw_vers

set -e

mkdir -p test_data

# Create an image with an APM partition table and a HFS+ file system
IMAGE_FILE="test_data/apm/apm"
IMAGE_SIZE="4M"

mkdir -p test_data/apm
rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'HFS+' -layout 'SPUD' -size ${IMAGE_SIZE} -type UDIF -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.dmg

# Create a sparse image with a HFS+ file system
IMAGE_FILE="test_data/sparseimage/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/sparseimage
rm -f ${IMAGE_FILE}.sparseimage

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparseimage -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.sparseimage

BASE_IMAGE_FILE=${IMAGE_FILE}

IMAGE_FILE="test_data/sparseimage/hfsplus_aes128"

rm -f ${IMAGE_FILE}.sparseimage

echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSP -stdinpass -o ${IMAGE_FILE}

# echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE} -encryption AES-128 -format UDSP -stdinpass -tgtimagekey encrypted-encoding-version=1 -o ${IMAGE_FILE}

# Create a sparse bundle with a HFS+ file system
IMAGE_FILE="test_data/sparsebundle/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/sparsebundle
rm -rf ${IMAGE_FILE}.sparsebundle

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSEBUNDLE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparsebundle -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.sparsebundle

# Create a raw image with a HFS+ file system
IMAGE_FILE="test_data/hfs/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/hfs
rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type UDIF -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg -noautoopen -nobrowse

create_file_entries "/Volumes/hfsplus_test"

detach_image ${IMAGE_FILE}.dmg

# Create compressed UDIF images
mkdir -p test_data/udif

BASE_IMAGE_FILE=${IMAGE_FILE}

# Create an ADC compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_adc"

rm -f ${IMAGE_FILE}.dmg

hdiutil convert ${BASE_IMAGE_FILE}.dmg -format UDCO -o ${IMAGE_FILE}

# Create a bzip2 compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_bzip2"

rm -f ${IMAGE_FILE}.dmg

hdiutil convert ${BASE_IMAGE_FILE}.dmg -format UDBZ -o ${IMAGE_FILE}

# Create a lzfse compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_lzfse"

rm -f ${IMAGE_FILE}.dmg

hdiutil convert ${BASE_IMAGE_FILE}.dmg -format ULFO -o ${IMAGE_FILE}

# Create a lzma compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_lzma"

rm -f ${IMAGE_FILE}.dmg

hdiutil convert ${BASE_IMAGE_FILE}.dmg -format ULMO -o ${IMAGE_FILE}

# Create a zlib compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_zlib"

rm -f ${IMAGE_FILE}.dmg

hdiutil convert ${BASE_IMAGE_FILE}.dmg -format UDZO -o ${IMAGE_FILE}

# Create a zlib compressed UDIF image with a resource fork.
# Note this works with older versions of hdiutil that support flatten/unflatten.
#
# IMAGE_FILE="test_data/udif/hfsplus_rsrc"
#
# rm -f ${IMAGE_FILE}.dmg
#
# hdiutil convert ${BASE_IMAGE_FILE}.dmg -format UDZO -o ${IMAGE_FILE}
#
# hdiutil unflatten test.dmg
# hdiutil flatten -noxml test.dmg

# Create an AES-128 encrypted zlib compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_zlib_aes128"

rm -f ${IMAGE_FILE}.dmg

echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE}.dmg -encryption AES-128 -format UDZO -stdinpass -o ${IMAGE_FILE}

# echo -n KeRaMiCs | hdiutil convert ${BASE_IMAGE_FILE}.dmg -encryption AES-128 -format UDZO -stdinpass -tgtimagekey encrypted-encoding-version=1 -o ${IMAGE_FILE}

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

# Create a raw image with an APFS container and single volume with a case-insensitive file system
IMAGE_FILE="test_data/apfs/apfs"
IMAGE_SIZE="4M"

mkdir -p test_data/apfs
rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'APFS' -size ${IMAGE_SIZE} -type UDIF -volname apfs_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg -noautoopen -nobrowse

create_file_entries "/Volumes/apfs_test"

detach_image ${IMAGE_FILE}.dmg

exit ${EXIT_SUCCESS}
