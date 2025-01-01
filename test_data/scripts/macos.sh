#!/usr/bin/env bash
#
# Script to generate Keramics test files on Mac OS.
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

EXIT_SUCCESS=0
EXIT_FAILURE=1

AFSCTOOL="/usr/local/bin/afsctool"

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

	# Create a file
	echo "Keramics" > ${MOUNT_POINT}/testdir1/testfile1

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
	fi

	# Note that compressed UDIF images don't allow for block or character device files.

	# Create a pipe (FIFO) file
	mkfifo ${MOUNT_POINT}/testdir1/pipe1
}

assert_availability_binary diskutil
assert_availability_binary hdiutil
assert_availability_binary mkfifo
assert_availability_binary mknod
assert_availability_binary sw_vers

set -e

DEVICE_NUMBER=`diskutil list | grep -e '^/dev/disk' | tail -n 1 | sed 's?^/dev/disk??;s? .*$??'`

mkdir -p test_data

# Create an image with an APM partition table and a HFS+ file system
VOLUME_DEVICE_NUMBER=$(( ${DEVICE_NUMBER} + 1 ))

IMAGE_FILE="test_data/apm/apm"
IMAGE_SIZE="4M"

mkdir -p test_data/apm
rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'HFS+' -layout 'SPUD' -size ${IMAGE_SIZE} -type UDIF -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg

create_file_entries "/Volumes/hfsplus_test"

hdiutil detach disk${VOLUME_DEVICE_NUMBER}

# Create a sparse image with a HFS+ file system
VOLUME_DEVICE_NUMBER=$(( ${DEVICE_NUMBER} + 1 ))

IMAGE_FILE="test_data/sparseimage/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/sparseimage
rm -f ${IMAGE_FILE}.sparseimage

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparseimage

create_file_entries "/Volumes/hfsplus_test"

hdiutil detach disk${VOLUME_DEVICE_NUMBER}

# Create a sparse bundle with a HFS+ file system
VOLUME_DEVICE_NUMBER=$(( ${DEVICE_NUMBER} + 1 ))

IMAGE_FILE="test_data/sparsebundle/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/sparsebundle
rm -rf ${IMAGE_FILE}.sparsebundle

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type SPARSEBUNDLE -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.sparsebundle

create_file_entries "/Volumes/hfsplus_test"

hdiutil detach disk${VOLUME_DEVICE_NUMBER}

# Create a raw image with a HFS+ file system
VOLUME_DEVICE_NUMBER=$(( ${DEVICE_NUMBER} + 1 ))

IMAGE_FILE="test_data/hfs/hfsplus"
IMAGE_SIZE="4M"

mkdir -p test_data/hfs
rm -f ${IMAGE_FILE}.dmg

hdiutil create -fs 'HFS+' -size ${IMAGE_SIZE} -type UDIF -volname hfsplus_test ${IMAGE_FILE}

hdiutil attach ${IMAGE_FILE}.dmg

create_file_entries "/Volumes/hfsplus_test"

# Create an ADC compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_adc"

mkdir -p test_data/udif
rm -f ${IMAGE_FILE}.dmg

hdiutil create -format UDCO -srcfolder "/Volumes/hfsplus_test" ${IMAGE_FILE}

# Create a bzip2 compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_bzip2"

mkdir -p test_data/udif
rm -f ${IMAGE_FILE}.dmg

hdiutil create -format UDBZ -srcfolder "/Volumes/hfsplus_test" ${IMAGE_FILE}

# Create a lzfse compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_lzfse"

mkdir -p test_data/udif
rm -f ${IMAGE_FILE}.dmg

hdiutil create -format ULFO -srcfolder "/Volumes/hfsplus_test" ${IMAGE_FILE}

# Create a lzma compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_lzma"

mkdir -p test_data/udif
rm -f ${IMAGE_FILE}.dmg

hdiutil create -format ULMO -srcfolder "/Volumes/hfsplus_test" ${IMAGE_FILE}

# Create a zlib compressed UDIF image.
IMAGE_FILE="test_data/udif/hfsplus_zlib"

mkdir -p test_data/udif
rm -f ${IMAGE_FILE}.dmg

hdiutil create -format UDZO -srcfolder "/Volumes/hfsplus_test" ${IMAGE_FILE}

hdiutil detach disk${VOLUME_DEVICE_NUMBER}

exit ${EXIT_SUCCESS}
