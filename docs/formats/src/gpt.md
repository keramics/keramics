# GUID Partition Table (GPT) format

The GUID Partition Table (GPT) is a partitioning schema that is the successor
to the [Master Boot Record (MBR) Partition Table](mbr.md) for Intel x86 based
computers.

## Overview

A GUID Partition Table (GPT) consists of:

* A protective or hybrid Master Boot Record (MBR) stored in block (LBA) 0
* A GPT partition table header stored in block (LBA) 1
* GPT partition entries stored in blocks (LBA) 2 - 33
* paritions area
  * GPT partitions
  * MBR partitions if hybrid MBR/GPT
* backup GPT partition entries (typically stored the blocks (LBA) before the last block -33 - -2)
* A backup GPT partition table header (typically stored in the last block (LBA) -1)

The GPT partition table header signature can be used to determine the block
(LBA) (or sector) size.

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | N/A
| Character strings | UTF-16 little-endian without byte order mark (BOM)

## Master Boot Record (MBR)

### Hybrid Master Boot Record (MBR)

In hybrid configuration both GPT and MBR are used concurrently. Depending on
the operating system one might have precedence over the other.

### Protective Master Boot Record (MBR)

The Protective Master Boot Record (MBR) is an MBR with a single partition of
type "EFI GPT protective partition" (0xee) that allocated as much of the drive
as possible.

## GPT partition table header

The GPT partition table header is 92 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "EFI PART" | Signature
| 8 | 2 | 0 | Minor format version
| 10 | 2 | 1 | Major format version
| 12 | 4 | 92 | Header data size, which contains the size of the GPT partition table header data
| 16 | 4 | | Header data checksum
| 20 | 4 | 0 | Unknown (Reserved)
| 24 | 8 | | Partition header block number (LBA)
| 32 | 8 | | Backup partition header block number (LBA)
| 40 | 8 | | Partitions area start block number (LBA)
| 48 | 8 | | Partitions area end block number (LBA), where the block number is included in the partitions area block range
| 56 | 16 | | Disk identifier (GUID)
| 72 | 8 | | Partition entries start block number (LBA)
| 80 | 4 | | Number of partition entries
| 84 | 4 | 128 | Partition entry data size
| 88 | 4 | | Partition entries data checksum
| 92 | ... | 0 | Unknown (Reserved)

> Note that the partition entries start block number (LBA) of the backup
> partition table header will point to the backup partition entries.

> Note that the number of partition entries value contains the number of
> available partition entries not the number of used partition entries. Empty
> partition entries have a unused entry partition type identifier.

### Checksum calculation

The [CRC-32 algorithm](https://www.ietf.org/rfc/rfc1952.txt) with polynominal
0x04c11db7 and initial value of 0 is used to calculate the checksums.

The checksum is calculated over the 92 bytes of the table header data, where the
header data checkum value is considered to be 0 during calculation.

## GPT partition entries

### GPT Partition entry

The GPT partition entry is 128 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | [Partition type](#partition_types) identifier (GUID)
| 16 | 16 | | Partition identifier (GUID)
| 32 | 8 | | Partition start block number (LBA)
| 40 | 8 | | Partition end block number (LBA), where the block number is included in the partition block range
| 48 | 8 | | [Attribute flags](#partition_attribute_flags)
| 56 | 72 | | Partition name, which contains a UTF-16 little-endian string

### <a name="partition_types"></a>Partition types

| Value | Identifier | Description
| --- | --- | ---
| 00000000-0000-0000-0000-000000000000 | | Unused entry
| 024dee41-33e7-11d3-9d69-0008c781f39f | | MBR partition scheme
| c12a7328-f81f-11d2-ba4b-00a0c93ec93b | | EFI System partition
| 21686148-6449-6e6f-744e-656564454649 | | BIOS boot partition
| d3bfe2de-3daf-11df-ba40-e3a556d89593 | | Intel Fast Flash (iFFS) partition (for Intel Rapid Start technology)
| f4019732-066e-4e12-8273-346c5641494f | | Sony boot partition
| bfbfafe7-a34f-448a-9a5b-6213eb736c22 | | Lenovo boot partition
| <td colspan="3"> *Windows*
| e3c9e316-0b5c-4db8-817d-f92df00215ae | | Microsoft Reserved Partition (MSR)
| ebd0a0a2-b9e5-4433-87c0-68b6b72699c7 | | Basic data partition
| 5808c8aa-7e8f-42e0-85d2-e1e90434cfb3 | | Logical Disk Manager (LDM) metadata partition
| af9b60a0-1431-4f62-bc68-3311714a69ad | | Logical Disk Manager data partition
| de94bba4-06d1-4d40-a16a-bfd50179d6ac | | Windows Recovery Environment
| 37affc90-ef7d-4e96-91c3-2d7ae055b174 | | IBM General Parallel File System (GPFS) partition
| e75caf8f-f680-4cee-afa3-b001e56efc2d | | Storage Spaces partition
| <td colspan="3"> *HP-UX*
| 75894c1e-3aeb-11d3-b7c1-7b03a0000000 | | Data partition
| e2a1e728-32e3-11d6-a682-7b03a0000000 | | Service Partition
| <td colspan="3"> *Linux*
| 0fc63daf-8483-4772-8e79-3d69d8477de4 | | Linux filesystem data
| a19d880f-05fc-4d3b-a006-743f0f84911e | | RAID partition
| 44479540-f297-41b2-9af7-d131d5f0458a | | Root partition (x86)
| 4f68bce3-e8cd-4db1-96e7-fbcaf984b709 | | Root partition (x86-64)
| 69dad710-2ce4-4e3c-b16c-21a1d49abed3 | | Root partition (32-bit ARM)
| b921b045-1df0-41c3-af44-4c6f280d3fae | | Root partition (64-bit ARM/AArch64)
| 0657fd6d-a4ab-43c4-84e5-0933c84b4f4f | | Swap partition
| e6d6d379-f507-44c2-a23c-238f2a3df928 | | Logical Volume Manager (LVM) partition
| 933ac7e1-2eb4-4f13-b844-0e14e2aef915 | | /home partition
| 3b8f8425-20e0-4f3b-907f-1a25a76f98e8 | | /srv (server data) partition
| 7ffec5c9-2d00-49b7-8941-3ea10a5586b7 | | Plain dm-crypt partition
| ca7d7ccb-63ed-4c53-861c-1742536059cc | | LUKS partition
| 8da63339-0007-60c0-c436-083ac8230908 | | Reserved
| <td colspan="3"> *FreeBSD*
| 83bd6b9d-7f41-11dc-be0b-001560b84f0f | | Boot partition
| 516e7cb4-6ecf-11d6-8ff8-00022d09712b | | Data partition
| 516e7cb5-6ecf-11d6-8ff8-00022d09712b | | Swap partition
| 516e7cb6-6ecf-11d6-8ff8-00022d09712b | | Unix File System (UFS) partition
| 516e7cb8-6ecf-11d6-8ff8-00022d09712b | | Vinum volume manager partition
| 516e7cba-6ecf-11d6-8ff8-00022d09712b | | ZFS partition
| <td colspan="3"> *Darwin / Mac OS*
| 48465300-0000-11aa-aa11-00306543ecac | | Hierarchical File System Plus (HFS+) partition
| 7c3457ef-0000-11aa-aa11-00306543ecac | | Apple APFS
| 55465300-0000-11aa-aa11-00306543ecac | | Apple UFS container
| 6a898cc3-1dd2-11b2-99a6-080020736631 | | ZFS
| 52414944-0000-11aa-aa11-00306543ecac | | Apple RAID partition
| 52414944-5f4f-11aa-aa11-00306543ecac | | Apple RAID partition, offline
| 426f6f74-0000-11aa-aa11-00306543ecac | | Apple Boot partition (Recovery HD)
| 4c616265-6c00-11aa-aa11-00306543ecac | | Apple Label
| 5265636f-7665-11aa-aa11-00306543ecac | | Apple TV Recovery partition
| 53746f72-6167-11aa-aa11-00306543ecac | | Apple Core Storage (i.e. Lion FileVault) partition
| b6fa30da-92d2-4a9a-96f1-871ec6486200 | | SoftRAID_Status
| 2e313465-19b9-463f-8126-8a7993773801 | | SoftRAID_Scratch
| fa709c7e-65b1-4593-bfd5-e71d61de9b02 | | SoftRAID_Volume
| bbba6df5-f46f-4a89-8f59-8765b2727503 | | SoftRAID_Cache
| <td colspan="3"> *Solaris / illumos*
| 6a82cb45-1dd2-11b2-99a6-080020736631 | | Boot partition
| 6a85cf4d-1dd2-11b2-99a6-080020736631 | | Root partition
| 6a87c46f-1dd2-11b2-99a6-080020736631 | | Swap partition
| 6a8b642b-1dd2-11b2-99a6-080020736631 | | Backup partition
| 6a898cc3-1dd2-11b2-99a6-080020736631 | | /usr partition
| 6a8ef2e9-1dd2-11b2-99a6-080020736631 | | /var partition
| 6a90ba39-1dd2-11b2-99a6-080020736631 | | /home partition
| 6a9283a5-1dd2-11b2-99a6-080020736631 | | Alternate sector
| 6a8d2ac7-1dd2-11b2-99a6-080020736631 | | Reserved partition
| 6a945a3b-1dd2-11b2-99a6-080020736631 | | Reserved partition
| 6a96237f-1dd2-11b2-99a6-080020736631 | | Reserved partition
| 6a9630d1-1dd2-11b2-99a6-080020736631 | | Reserved partition
| 6a980767-1dd2-11b2-99a6-080020736631 | | Reserved partition
| <td colspan="3"> *NetBSD*
| 49f48d32-b10e-11dc-b99b-0019d1879648 | | Swap partition
| 49f48d5a-b10e-11dc-b99b-0019d1879648 | | FFS partition
| 49f48d82-b10e-11dc-b99b-0019d1879648 | | LFS partition
| 49f48daa-b10e-11dc-b99b-0019d1879648 | | RAID partition
| 2db519c4-b10f-11dc-b99b-0019d1879648 | | Concatenated partition
| 2db519ec-b10f-11dc-b99b-0019d1879648 | | Encrypted partition
| <td colspan="3"> *Chrome OS*
| fe3a2a5d-4f32-41a7-b725-accc3285a309 | | Chrome OS kernel
| 3cb8e202-3b7e-47dd-8a3c-7ff2a13cfcec | | Chrome OS rootfs
| 2e0a753d-9e48-43b0-8337-b15192cb1b5e | | Chrome OS future use
| <td colspan="3"> *Container Linux by CoreOS*
| 5dfbf5f4-2848-4bac-aa5e-0d9a20b745a6 | | /usr partition (coreos-usr)
| 3884dd41-8582-4404-b9a8-e9b84f2df50e | | Resizable rootfs (coreos-resize)
| c95dc21a-df0e-4340-8d7b-26cbfa9a03e0 | | OEM customizations (coreos-reserved)
| be9067b9-ea49-4f15-b4f6-f36f8c9e1818 | | Root filesystem on RAID (coreos-root-raid)
| <td colspan="3"> *Haiku*
| 42465331-3ba3-10f1-802a-4861696b7521 | | Haiku BFS
| <td colspan="3"> *MidnightBSD*
| 85d5e45e-237c-11e1-b4b3-e89a8f7fc3a7 | | Boot partition
| 85d5e45a-237c-11e1-b4b3-e89a8f7fc3a7 | | Data partition
| 85d5e45b-237c-11e1-b4b3-e89a8f7fc3a7 | | Swap partition
| 0394ef8b-237e-11e1-b4b3-e89a8f7fc3a7 | | Unix File System (UFS) partition
| 85d5e45c-237c-11e1-b4b3-e89a8f7fc3a7 | | Vinum volume manager partition
| 85d5e45d-237c-11e1-b4b3-e89a8f7fc3a7 | | ZFS partition
| <td colspan="3"> *Ceph*
| 45b0969e-9b03-4f30-b4c6-b4b80ceff106 | | Journal
| 45b0969e-9b03-4f30-b4c6-5ec00ceff106 | | dm-crypt journal
| 4fbd7e29-9d25-41b8-afd0-062c0ceff05d | | OSD
| 4fbd7e29-9d25-41b8-afd0-5ec00ceff05d | | dm-crypt OSD
| 89c57f98-2fe5-4dc0-89c1-f3ad0ceff2be | | Disk in creation
| 89c57f98-2fe5-4dc0-89c1-5ec00ceff2be | | dm-crypt disk in creation
| cafecafe-9b03-4f30-b4c6-b4b80ceff106 | | Block
| 30cd0809-c2b2-499c-8879-2d6b78529876 | | Block DB
| 5ce17fce-4087-4169-b7ff-056cc58473f9 | | Block write-ahead log
| fb3aabf9-d25f-47cc-bf5e-721d1816496b | | Lockbox for dm-crypt keys
| 4fbd7e29-8ae0-4982-bf9d-5a8d867af560 | | Multipath OSD
| 45b0969e-8ae0-4982-bf9d-5a8d867af560 | | Multipath journal
| cafecafe-8ae0-4982-bf9d-5a8d867af560 | | Multipath block
| 7f4a666a-16f3-47a2-8445-152ef4d03f6c | | Multipath block
| ec6d6385-e346-45dc-be91-da2a7c8b3261 | | Multipath block DB
| 01b41e1b-002a-453c-9f17-88793989ff8f | | Multipath block write-ahead log
| cafecafe-9b03-4f30-b4c6-5ec00ceff106 | | dm-crypt block
| 93b0052d-02d9-4d8a-a43b-33a3ee4dfbc3 | | dm-crypt block DB
| 306e8683-4fe2-4330-b7c0-00a917c16966 | | dm-crypt block write-ahead log
| 45b0969e-9b03-4f30-b4c6-35865ceff106 | | dm-crypt LUKS journal
| cafecafe-9b03-4f30-b4c6-35865ceff106 | | dm-crypt LUKS block
| 166418da-c469-4022-adf4-b30afd37f176 | | dm-crypt LUKS block DB
| 86a32090-3647-40b9-bbbd-38d8c573aa86 | | dm-crypt LUKS block write-ahead log
| 4fbd7e29-9d25-41b8-afd0-35865ceff05d | | dm-crypt LUKS OSD
| <td colspan="3"> *OpenBSD*
| 824cc7a0-36a8-11e3-890a-952519ad3f61 | | Data partition
| <td colspan="3"> *QNX*
| cef5a9ad-73bc-4601-89f3-cdeeeee321a1 | | Power-safe (QNX6) file system
| <td colspan="3"> *Plan 9*
| c91818f9-8025-47af-89d2-f030d7000c2c | | Plan 9 partition
| <td colspan="3"> *VMware ESX*
| 9d275380-40ad-11db-bf97-000c2911d1b8 | | vmkcore (coredump partition)
| aa31e02a-400f-11db-9590-000c2911d1b8 | | VMFS filesystem partition
| 9198effc-31c0-11db-8f78-000c2911d1b8 | | VMware Reserved
| <td colspan="3"> *Android-IA*
| 2568845d-2332-4675-bc39-8fa5a4748d15 | | Bootloader
| 114eaffe-1552-4022-b26e-9b053604cf84 | | Bootloader2
| 49a4d17f-93a3-45c1-a0de-f50b2ebe2599 | | Boot
| 4177c722-9e92-4aab-8644-43502bfd5506 | | Recovery
| ef32a33b-a409-486c-9141-9ffb711f6266 | | Misc
| 20ac26be-20b7-11e3-84c5-6cfdb94711e9 | | Metadata
| 38f428e6-d326-425d-9140-6e0ea133647c | | System
| a893ef21-e428-470a-9e55-0668fd91a2d9 | | Cache
| dc76dda9-5ac1-491c-af42-a82591580c0d | | Data
| ebc597d0-2053-4b15-8b64-e0aac75f4db1 | | Persistent
| c5a0aeec-13ea-11e5-a1b1-001e67ca0c3c | | Vendor
| bd59408b-4514-490d-bf12-9878d963f378 | | Config
| 8f68cc74-c5e5-48da-be91-a0c8c15e9c80 | | Factory
| 9fdaa6ef-4b3f-40d2-ba8d-bff16bfb887b | | Factory (alt)
| 767941d0-2085-11e3-ad3b-6cfdb94711e9 | | Fastboot / Tertiary
| ac6d7924-eb71-4df8-b48d-e267b27148ff | | OEM
| <td colspan="3"> *Android 6.0+ ARM*
| 19a710a2-b3ca-11e4-b026-10604b889dcf | | Android Meta
| 193d1ea4-b3ca-11e4-b075-10604b889dcf | | Android EXT
| <td colspan="3"> *Open Network Install Environment (ONIE)*
| 7412f7d5-a156-4b13-81dc-867174929325 | | Boot
| d4e6e2cd-4469-46f3-b5cb-1bff57afc149 | | Config
| <td colspan="3"> *PowerPC*
| 9e1a2d38-c612-4316-aa26-8b49521e5a8b | | PReP boot
| <td colspan="3"> *freedesktop.org OSes (Linux, etc.)*
| bc13c2ff-59e6-4262-a352-b275fd6f7172 | | Shared boot loader configuration
| <td colspan="3"> *Atari TOS*
| 734e5afe-f61a-11e6-bc64-92361f002671 | | Basic data partition (GEM, BGM, F32)

### <a name="partition_attribute_flags"></a>Partition attribute flags

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 1 bit | | Partition is required by the platform, e.g. an OEM partition
| 0.1 | 1 bit | | EFI firmware should ignore the content of the partition
| 0.2 | 1 bit | | Partition contains bootable legacy BIOS, equivalent to MBR active flag
| 0.3 | 45 bits | | Unknown (Reserved)
| 6.0 | 16 bits | | Flags specific to the partition type

#### Microsoft basic partition type attribute flags

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 7.4 | 1 bit | | Partition is read-only
| 7.5 | 1 bit | | Partition is a shadow copy (of another partition)
| 7.6 | 1 bit | | Partition is hidden
| 7.7 | 1 bit | | Partition should not have a drive letter assigned (no auto-mount)

#### ChromeOS partition type attribute flags

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 6.0 | 4 bits | | Priority, where 15 is thehighest priority, 1 is the lowest and 0 indicates the partition is not bootable
| 6.4 | 4 bits | | Number of tries to attempt to boot from the partition
| 7.0 | 1 bit | | Partition was previously successfully booted from
