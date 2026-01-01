#!/usr/bin/env bash
#
# Shared functionality for scripts to generate Keramics test files on Linux.
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

CURRENT_GID=$( id -g );
CURRENT_UID=$( id -u );

MOUNT_POINT="/mnt/keramics"

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

# Creates test file entries.
#
# Arguments:
#   a string containing the mount point of the image file
#
create_test_file_entries()
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
}

# Creates test file entries with extended attributes.
#
# Arguments:
#   a string containing the mount point of the image file
#
create_test_file_entries_with_extended_attributes()
{
	MOUNT_POINT=$1

	create_test_file_entries ${MOUNT_POINT}

	# Create a hard link to a file
	ln ${MOUNT_POINT}/testdir1/testfile1 ${MOUNT_POINT}/file_hardlink1

	# Create a symbolic link to a file
	ln -s ${MOUNT_POINT}/testdir1/testfile1 ${MOUNT_POINT}/file_symboliclink1

	# Create a hard link to a directory
	# ln: hard link not allowed for directory

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

	# Create a file with an extended attribute
	touch ${MOUNT_POINT}/testdir1/xattr1
	setfattr -n "user.myxattr1" -v "My 1st extended attribute" ${MOUNT_POINT}/testdir1/xattr1

	# Create a directory with an extended attribute
	mkdir ${MOUNT_POINT}/testdir1/xattr2
	setfattr -n "user.myxattr2" -v "My 2nd extended attribute" ${MOUNT_POINT}/testdir1/xattr2

	# Create a file with an initial (implict) sparse extent
	truncate -s $(( 1 * 1024 * 1024 )) ${MOUNT_POINT}/testdir1/initial_sparse1
	echo "File with an initial sparse extent" >> ${MOUNT_POINT}/testdir1/initial_sparse1

	# Create a file with a trailing (implict) sparse extent
	echo "File with a trailing sparse extent" > ${MOUNT_POINT}/testdir1/trailing_sparse1
	truncate -s $(( 1 * 1024 * 1024 )) ${MOUNT_POINT}/testdir1/trailing_sparse1

	# Create a file with an uninitialized extent
	fallocate -x -l 4096 ${MOUNT_POINT}/testdir1/uninitialized1
	echo "File with an uninitialized extent" >> ${MOUNT_POINT}/testdir1/uninitialized1

	# Create a block device file
	# Need to run mknod with sudo otherwise it errors with: Operation not permitted
	sudo mknod ${MOUNT_POINT}/testdir1/blockdev1 b 24 57

	# Create a character device file
	# Need to run mknod with sudo otherwise it errors with: Operation not permitted
	sudo mknod ${MOUNT_POINT}/testdir1/chardev1 c 13 68

	# Create a pipe (FIFO) file
	mknod ${MOUNT_POINT}/testdir1/pipe1 p
}

# Creates test file entries with a long file name.
#
# Arguments:
#   a string containing the mount point of the image file
#
create_test_file_entries_with_long_file_name()
{
	MOUNT_POINT=$1

	create_test_file_entries ${MOUNT_POINT}

        # Create a file with a long filename
        touch "${MOUNT_POINT}/testdir1/My long, very long file name, so very long"
}
