#!/usr/bin/env bash
#
# Script to generate Keramics test files on Linux.
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

        # Create a file with a long filename
        touch "${MOUNT_POINT}/testdir1/My long, very long file name, so very long"
}

# Creates test file entries.
#
# Arguments:
#   a string containing the mount point of the image file
#
create_test_file_entries_with_extended_attributes()
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

assert_availability_binary cryptsetup
assert_availability_binary dd
assert_availability_binary fdisk
assert_availability_binary genisoimage
assert_availability_binary gdisk
assert_availability_binary losetup
assert_availability_binary lvcreate
assert_availability_binary mke2fs
assert_availability_binary mkfs.fat
assert_availability_binary mkfs.xfs
assert_availability_binary mkntfs
assert_availability_binary pvcreate
assert_availability_binary qemu-img
assert_availability_binary setfattr
assert_availability_binary vgchange
assert_availability_binary vgcreate

set -e

CURRENT_GID=$( id -g );
CURRENT_UID=$( id -u );

mkdir -p test_data

MOUNT_POINT="/mnt/keramics"

sudo mkdir -p ${MOUNT_POINT}

SECTOR_SIZE=512

# Create an ext2 file system.
IMAGE_FILE="test_data/ext/ext2.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/ext

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -I 128 -L ext2_test -q -t ext2 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create an ext3 file system.
IMAGE_FILE="test_data/ext/ext3.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -L ext3_test -q -t ext3 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create an ext4 file system.
IMAGE_FILE="test_data/ext/ext4.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mke2fs -L ext4_test -q -t ext4 ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create a FAT-12 file system.
IMAGE_FILE="test_data/fat/fat12.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/fat

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mkfs.fat -F 12 -n "FAT12_TEST" -S ${SECTOR_SIZE} ${IMAGE_FILE}

sudo mount -o loop,rw,gid=${CURRENT_GID},uid=${CURRENT_UID} ${IMAGE_FILE} ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create an ISO9660 level 3 file system
IMAGE_FILE="test_data/iso9660/level3.iso"

mkdir -p test_data/iso9660

sudo mount -o loop,rw test_data/ext/ext2.raw ${MOUNT_POINT}

genisoimage -input-charset utf8 -iso-level 3 -o ${IMAGE_FILE} ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create a GPT volume system with 2 partitions.
# * the first partition with an ext2 file system.
# * the second partition with a NTFS file system.
IMAGE_FILE="test_data/gpt/gpt.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/gpt

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

gdisk ${IMAGE_FILE} <<EOT
n
1
2048
+1024K
8300
n
2
4096
+1536K
0700
w
y
EOT

sudo losetup -o $(( 2048 * ${SECTOR_SIZE} )) --sizelimit $(( 1024 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

sudo losetup -o $(( 4096 * ${SECTOR_SIZE} )) --sizelimit $(( 1536 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create an empty GPT volume system and a MBR volume system with 1 partition.
# * the partition is a primary partition with an ext2 file system.
IMAGE_FILE="test_data/gpt/empty_with_mbr.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

gdisk ${IMAGE_FILE} <<EOT
o
y
w
y
EOT

# Note that fdisk will write into the GPT partition entries area if the partition start offset is not set correctly.
fdisk -u ${IMAGE_FILE} <<EOT
M
d
n
p
1
48
+256K
w
EOT

sudo losetup -o $(( 48 * ${SECTOR_SIZE} )) --sizelimit $(( 256 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create a LVM volume system with 2 volumes.
# * the first volume with an ext2 file system.
IMAGE_FILE="test_data/lvm/lvm.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))

mkdir -p test_data/lvm

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

sudo losetup /dev/loop99 ${IMAGE_FILE}

sudo pvcreate -q /dev/loop99 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo vgcreate -q test_volume_group /dev/loop99 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo lvcreate --name test_logical_volume1 -q --size 4m --type linear test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/test_volume_group/test_logical_volume1

sudo mount -o loop,rw /dev/test_volume_group/test_logical_volume1 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo lvcreate --name test_logical_volume2 -q --size 4m --type linear test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo vgchange --activate n -q test_volume_group 2>&1 | sed '/is using an old PV header, modify the VG to update/ d;/open failed: No medium found/ d'

sudo losetup -d /dev/loop99

# Create a LUKS 1 encrypted volume system with an ext2 file system.
IMAGE_FILE="test_data/luks/luks1.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/luks

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

cryptsetup --batch-mode --cipher aes-cbc-plain --hash sha1 --type luks1 luksFormat ${IMAGE_FILE} <<EOT
KeramicsLuks1
EOT

sudo cryptsetup luksOpen ${IMAGE_FILE} keramics_luks <<EOT
KeramicsLuks1
EOT

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/mapper/keramics_luks

sudo mount -o loop,rw /dev/mapper/keramics_luks ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sleep 1

sudo cryptsetup luksClose keramics_luks

# Create a MBR volume system with 2 partitions.
# * the first partition is a primary partition with an ext2 file system.
# * the second partition is an extended partition with a NTFS file system.
IMAGE_FILE="test_data/mbr/mbr.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/mbr

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

fdisk -b ${SECTOR_SIZE} -u ${IMAGE_FILE} <<EOT
n
p
1

+1024K

n
e
2


n

+1536K
t
5
7
w
EOT

sudo losetup -o $(( 1 * ${SECTOR_SIZE} )) --sizelimit $(( 1024 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mke2fs -I 128 -L "ext2_test" -q -t ext2 /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

sudo losetup -o $(( 4096 * ${SECTOR_SIZE} )) --sizelimit $(( 1536 * 1024 )) /dev/loop99 ${IMAGE_FILE}

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} /dev/loop99

sudo mount -o loop,rw /dev/loop99 ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

sudo losetup -d /dev/loop99

# Create a NTFS file system.
IMAGE_FILE="test_data/ntfs/ntfs.raw"
IMAGE_SIZE=$(( 4 * 1024 * 1024 ))

mkdir -p test_data/ntfs

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

sudo mkntfs -F -L "ntfs_test" -q -s ${SECTOR_SIZE} ${IMAGE_FILE}

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

# Create a QCOW image with an ext2 file system.
IMAGE_FILE="test_data/qcow/ext2.qcow2"

mkdir -p test_data/qcow

qemu-img convert -f raw -O qcow2 test_data/ext/ext2.raw ${IMAGE_FILE}

# Create QCOW image with a FAT-16 file system.
IMAGE_FILE="test_data/fat/fat16.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mkfs.fat -F 16 -n "FAT16_TEST" -S ${SECTOR_SIZE} ${IMAGE_FILE}

sudo mount -o loop,rw,gid=${CURRENT_GID},uid=${CURRENT_UID} ${IMAGE_FILE} ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

qemu-img convert -f raw -O qcow2 ${IMAGE_FILE} test_data/qcow/fat16.qcow2

rm -f ${IMAGE_FILE}

# Create QCOW image with a FAT-32 file system.
IMAGE_FILE="test_data/fat/fat32.raw"
IMAGE_SIZE=$(( 64 * 1024 * 1024 ))

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

mkfs.fat -F 32 -n "FAT32_TEST" -S ${SECTOR_SIZE} ${IMAGE_FILE}

sudo mount -o loop,rw,gid=${CURRENT_GID},uid=${CURRENT_UID} ${IMAGE_FILE} ${MOUNT_POINT}

create_test_file_entries ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

qemu-img convert -f raw -O qcow2 ${IMAGE_FILE} test_data/qcow/fat32.qcow2

rm -f ${IMAGE_FILE}

# Create a VDI image with an ext2 file system.
IMAGE_FILE="test_data/vdi/ext2.vdi"

mkdir -p test_data/vdi

qemu-img convert -f raw -O vdi test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a VHD image with an ext2 file system.
IMAGE_FILE="test_data/vhd/ext2.vhd"

mkdir -p test_data/vhd

qemu-img convert -f raw -O vpc test_data/ext/ext2.raw ${IMAGE_FILE}

# Create VHDX image with an ext2 file system.
IMAGE_FILE="test_data/vhdx/ext2.vhdx"

mkdir -p test_data/vhdx

qemu-img convert -f raw -O vhdx test_data/ext/ext2.raw ${IMAGE_FILE}

# Create VMDK image with an ext2 file system.
IMAGE_FILE="test_data/vmdk/ext2.vmdk"

mkdir -p test_data/vmdk

qemu-img convert -f raw -O vmdk test_data/ext/ext2.raw ${IMAGE_FILE}

# Create a XFS file system.
IMAGE_FILE="test_data/xfs/xfs.raw"
IMAGE_SIZE=$(( 16 * 1024 * 1024 ))

mkdir -p test_data/xfs

dd if=/dev/zero of=${IMAGE_FILE} bs=${SECTOR_SIZE} count=$(( ${IMAGE_SIZE} / ${SECTOR_SIZE} )) 2> /dev/null

# Note that the environment variables are necessary to allow for a XFS file system < 300 MiB.
export TEST_DEV=1
export TEST_DIR=1
export QA_CHECK_FS=1

mkfs.xfs -b size=4096 -i size=512 -L "xfs_test" -m bigtime=0 -q -s size=${SECTOR_SIZE} ${IMAGE_FILE}

export TEST_DEV=
export TEST_DIR=
export QA_CHECK_FS=

sudo mount -o loop,rw ${IMAGE_FILE} ${MOUNT_POINT}

sudo chown ${USER} ${MOUNT_POINT}

create_test_file_entries_with_extended_attributes ${MOUNT_POINT}

sudo umount ${MOUNT_POINT}

exit ${EXIT_SUCCESS}
