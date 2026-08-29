# X File System (XFS)

The X File System (XFS) is a file system that originates from SGI but is used in various Linux
distributions like RHEL. Some sources indicate that X was a a placeholder for a name that never
given.

## Overview

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch) and fraction of second in number of nanoseconds, or in bigtime (number of nanoseconds since December 13, 1901 20:45:52 UTC) |
| Character strings | UTF-8 or a narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a system defined codepage |

## Terminology

### Absolute and relative inode numbers

A relative inode number is an inode number used within a specific allocation group. An absolute
inode number combines the allocation group number and the relative inode number.

```python
number_of_relative_inode_number_bits = (
    allocation_group_size_log2 + number_of_inodes_per_block_log2
)
absolute_inode_number = (
    (allocation_group_number << number_of_relative_inode_number_bits) | relative_inode_number
)
```

### File system block number {#file_system_block_number}

A relative block number is a block number relative to the start of an allocation group. A file
system block number (xfs_fsblock_t) combines the allocation group number and the relative block
number.

```python
number_of_relative_block_number_bits = allocation_group_size_log2

file_system_block_number = (
    (allocation_group_number << number_of_relative_block_number_bits) | relative_block_number
)
file_offset = (allocation_group_block_number + relative_block_number) * block_size
```

## The allocation group

An allocation group consists of:

* a sector containing a superblock
* a sector containing free block information
* a sector containing inode B+ tree information
* a sector containing internal free list
* blocks containing
  * root of the inode B+ tree
  * root of the free space B+ tree
  * free list
  * inodes table

## The superblock

The XFS superblock (xfs_sb_t) is (at least) 512 bytes of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "XFSB" | Signature |
| 4 | 4 | | Block size, which is typicaly 4096 bytes (4 KiB) and can range from 512 to 65536 bytes |
| 8 | 8 | | Total number of blocks |
| 16 | 8 | | Number of real-time (device) blocks |
| 24 | 8 | | Number of real-time (device) extents |
| 32 | 16 | | File system (or volume) identifier, which contains an UUID |
| 48 | 8 | | Journal block number, which contains a [file system block number](#file_system_block_number) or 0 if the journal is stored on a separate device |
| 56 | 8 | | Root directory (absolute) inode number, which contains -1 (0xffffffffffffffff) if not set |
| 64 | 8 | | Real-time bitmap extents inode number, which contains -1 (0xffffffffffffffff) if not set |
| 72 | 8 | | Real-time bitmap summary inode number, which contains -1 (0xffffffffffffffff) if not set |
| 80 | 4 | | Real-time extent size, in number of blocks |
| 84 | 4 | | Allocation group size, in number of blocks |
| 88 | 4 | | Number of allocation groups |
| 92 | 4 | | Real-time bitmap size, in number of blocks |
| 96 | 4 | | Journal size, in number of blocks |
| 100 | 2 | | [Format version and feature flags](#format_version_and_feature_flags) |
| 102 | 2 | | Sector size (in bytes) |
| 104 | 2 | | Inode size (in bytes), which can range from 256 to 2048 |
| 106 | 2 | | Number of inodes per block |
| 108 | 12 | | Volume label (or name) |
| 120 | 1 | | Block size in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 121 | 1 | | Sector size in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 122 | 1 | | Inode size in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 123 | 1 | | Number of inodes per block in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 124 | 1 | | Allocation group size in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 125 | 1 | | Number of real-time (device) extents in log2, where value = ( 2 ^ value in log2 ) or 0 if value in log2 is 0 |
| 126 | 1 | | Creation flag, which contains a value to indicate file system is being created |
| 127 | 1 | | Inodes percentage, which contains the percentage of the maximum space of the volume to use for inodes |
| <td colspan="4">*Only used in the first superblock*</td> |
| 128 | 8 | | Number of inodes |
| 136 | 8 | | Number of free inodes |
| 144 | 8 | | Number of free data blocks |
| 152 | 8 | | Number of free real-time extents |
| <td colspan="4">*Only used if the XFS_SB_VERSION_QUOTABIT feature flag is set*</td> |
| 160 | 8 | | User quota inode number |
| 168 | 8 | | Group (or project) quota inode number |
| 176 | 2 | | [Quota flags](#quota_flags) |
| <td colspan="4">*Common*</td> |
| 178 | 1 | | [Miscellaneous flags](#miscellaneous_flags) |
| 179 | 1 | 0 | Unknown (reserved or shared version number) |
| <td colspan="4">*Only used if the XFS_SB_VERSION_ALIGNBIT feature flag is set*</td> |
| 180 | 4 | | Inode chunk alignment size, in number of blocks |
| <td colspan="4">*Common*</td> |
| 184 | 4 | | Stripe (or RAID) unit size, in number of blocks |
| 188 | 4 | | Stripe (or RAID) width, in number of blocks |
| 192 | 1 | | Directory block size in log2 |
| 193 | 1 | | Journal device sector size in log2 |
| 194 | 2 | | Journal device sector size (in bytes) |
| <td colspan="4">*Only used if the XFS_SB_VERSION_LOGV2BIT feature flag is set*</td> |
| 196 | 4 | | Journal device stripe or RAID unit size |
| <td colspan="4">*Common*</td> |
| 200 | 4 | | [Secondary feature flags](#secondary_feature_flags) |
| 204 | 4 | | Copy of secondary feature flags, which was introduced to work-around 64-bit alignment errors |
| <td colspan="4">*If superblock format version >= 5 (XFS_SB_VERSION_5)*</td> |
| 208 | 4 | | [(Read-write) compatible feature flags](#compatible_feature_flags) |
| 212 | 4 | | [Read-only compatible feature flags](#read_only_compatible_feature_flags) |
| 216 | 4 | | [(Read-write) incompatible feature flags](#incompatible_feature_flags) |
| 220 | 4 | | [Journal (read-write) incompatible feature flags](#journal_incompatible_feature_flags) |
| 224 | 4 | | Checksum of the superblock |
| 228 | 4 | | Unknown (Sparse inode chunk alignment in number of blocks) |
| 232 | 4 | | Project quota inode number |
| 236 | 8 | | Journal log sequence number (LSN) of the last superblock update |
| <td colspan="4">*Only used if the XFS_SB_FEAT_INCOMPAT_META_UUID incompatible feature flag is set*</td> |
| 244 | 16 | | Metadata identifier, which contains an UUID |
| <td colspan="4">*Only used if the XFS_SB_FEAT_RO_COMPAT_RMAPBT incompatible feature flag is set*</td> |
| 260 | 8 | | Real-time Reverse Mapping B+tree inode number |
| 268 | 244 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

> Note that the allocation group size and allocation group size in log2 are not necessarily
> equivalent.

### Format version and feature flags {#format_version_and_feature_flags}

The 4 LSB contain the version the remaining bits are used to store feature flags.

<!-- rumdl-disable MD033 MD056 -->

| Version | Identifier | Introduced in |
| --- | --- | --- |
| <td colspan="3">*First generation*</td> |
| 1 | XFS_SB_VERSION_1 | Introduced in Irix 5.3 |
| 2 | XFS_SB_VERSION_2 | Introduced in Irix 6.2, added extended attribute support |
| 3 | XFS_SB_VERSION_3 | Introduced in Irix 6.2, added inode version 2 support |
| <td colspan="3">*Second generation*</td> |
| 4 | XFS_SB_VERSION_4 | Introduced in Irix 6.2, added directory version 2 support |
| <td colspan="3">*Third generation*</td> |
| 5 | XFS_SB_VERSION_5 | Intoduced in Linux 3.10 |

| Value | Identifier | Description |
| --- | --- | --- |
| <td colspan="3">*Introduced in XFS_SB_VERSION_2*</td> |
| 0x0010 | XFS_SB_VERSION_ATTRBIT | Inodes support extended attributes |
| <td colspan="3">*Introduced in XFS_SB_VERSION_3*</td> |
| 0x0020 | XFS_SB_VERSION_NLINKBIT | Inodes use a 32-bit number of links value |
| <td colspan="3">*Introduced in XFS_SB_VERSION_4*</td> |
| 0x0040 | XFS_SB_VERSION_QUOTABIT | Quotas enabled |
| 0x0080 | XFS_SB_VERSION_ALIGNBIT | Use inode chunk alignment |
| 0x0100 | XFS_SB_VERSION_DALIGNBIT | Has underlying stripe or RAID. The Stripe (or RAID) unit size and width values in the superblock should be set |
| 0x0200 | XFS_SB_VERSION_SHAREDBIT | Unknown (set if reserved shared version is used) |
| 0x0400 | XFS_SB_VERSION_LOGV2BIT | Has version 2 journaling logs |
| 0x0800 | XFS_SB_VERSION_SECTORBIT | Sector size is not 512 bytes |
| 0x1000 | XFS_SB_VERSION_EXTFLGBIT | Unwritten extents are used, which should always be set |
| 0x2000 | XFS_SB_VERSION_DIRV2BIT | Version 2 directories are used |
| 0x4000 | XFS_SB_VERSION_BORGBIT | Unknown (ASCII only case-insensitive) |
| 0x8000 | XFS_SB_VERSION_MOREBITSBIT | Secondary feature flags are used |

<!-- rumdl-enable MD033 MD056 -->

### Secondary feature flags {#secondary_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | XFS_SB_VERSION2_RESERVED1BIT | Unknown (reserved) |
| 0x00000002 | XFS_SB_VERSION2_LAZYSBCOUNTBIT | Has lazy global counters. Free space and inode values are only tracked in the primary superblock |
| 0x00000004 | XFS_SB_VERSION2_RESERVED4BIT | Unknown (reserved) |
| 0x00000008 | XFS_SB_VERSION2_ATTR2BIT | Version 2 extended attributes are used |
| 0x00000010 | XFS_SB_VERSION2_PARENTBIT | Inodes have a parent pointer |
| | | |
| 0x00000080 | XFS_SB_VERSION2_PROJID32BIT | Has 32-bit project identifiers |
| 0x00000100 | XFS_SB_VERSION2_CRCBIT | Has metadata checksums |
| 0x00000200 | XFS_SB_VERSION2_FTYPE | Directory entries contain a file type |

### Miscellaneous flags {#miscellaneous_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | XFS_SBF_READONLY | Read-only file system |

### Quota flags {#quota_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | XFS_UQUOTA_ACCT | User quota accounting is enabled |
| 0x0002 | XFS_UQUOTA_ENFD | User quotas are enforced |
| 0x0004 | XFS_UQUOTA_CHKD | User quotas have been checked and updated on disk |
| 0x0008 | XFS_PQUOTA_ACCT | Project quota accounting is enabled |
| 0x0010 | XFS_OQUOTA_ENFD | Other (group/project) quotas are enforced |
| 0x0020 | XFS_OQUOTA_CHKD | Other (group/project) quotas have been checked |
| 0x0040 | XFS_GQUOTA_ACCT | Group quota accounting is enabled |
| 0x0080 | XFS_GQUOTA_ENFD | Group quotas are enforced |
| 0x0100 | XFS_GQUOTA_CHKD | Group quotas have been checked |
| 0x0200 | XFS_PQUOTA_ENFD | Project quotas are enforced |
| 0x0400 | XFS_PQUOTA_CHKD | Project quotas have been checked |

### Compatible feature flags {#compatible_feature_flags}

Current no compatible feature flags are defined.

### Read-only compatible feature flags {#read_only_compatible_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | XFS_SB_FEAT_RO_COMPAT_FINOBT | Has free inode btree |
| 0x00000002 | XFS_SB_FEAT_RO_COMPAT_RMAPBT | Has reverse map btree |
| 0x00000004 | XFS_SB_FEAT_RO_COMPAT_REFLINK | Has reflinked files |
| 0x00000008 | XFS_SB_FEAT_RO_COMPAT_INOBTCNT | Has inobt block counts |

### Incompatible feature flags {#incompatible_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | XFS_SB_FEAT_INCOMPAT_FTYPE | Has filetype in dirent |
| 0x00000002 | XFS_SB_FEAT_INCOMPAT_SPINODES | Has sparse inode chunks |
| 0x00000004 | XFS_SB_FEAT_INCOMPAT_META_UUID | Use a metadata identifier |
| 0x00000008 | XFS_SB_FEAT_INCOMPAT_BIGTIME | Inode (v3) contains bigtime timestamps |
| 0x00000010 | XFS_SB_FEAT_INCOMPAT_NEEDSREPAIR | Needs repair |
| 0x00000020 | XFS_SB_FEAT_INCOMPAT_NREXT64 | Inode (v3) contains a 64-bit number of data extents and 32-bit number of (extended) attribute extent values |
| 0x00000040 | XFS_SB_FEAT_INCOMPAT_EXCHRANGE | Has exchangerange |
| 0x00000080 | XFS_SB_FEAT_INCOMPAT_PARENT | Has parent directory reference attributes |
| 0x00000100 | XFS_SB_FEAT_INCOMPAT_METADIR | Has metadata directory (tree) |
| 0x00000200 | XFS_SB_FEAT_INCOMPAT_ZONED | Has zoned RT allocator |
| 0x00000400 | XFS_SB_FEAT_INCOMPAT_ZONE_GAPS | RTGs have LBA gaps |

### Journal incompatible feature flags {#journal_incompatible_feature_flags}

Current no journal incompatible feature flags are defined.

## Free block information

The free block information stores references:

* the block offset B+ tree, that tracks the free space by block number
* the block count B+ tree, that tracks the size of the free space block

The free block information (xfs_agf_t) is 64 or 224 bytes of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "XAGF" | Signature |
| 4 | 4 | 1 | Version |
| 8 | 4 | | Sequence number, which contains the allocation group number of the corresponding sector |
| 12 | 4 | | Unknown (Allocation group size), in number of blocks |
| 16 | 4 | | Free space counts B+ tree root block number |
| 20 | 4 | | Free space sizes B+ tree root block number |
| 24 | 4 | | Unknown (reserved) |
| 28 | 4 | | Free space counts B+ tree height/depth |
| 32 | 4 | | Free space sizes B+ tree height/depth |
| 36 | 4 | | Unknown (reserved) |
| 40 | 4 | | Index of the first "free list" block |
| 44 | 4 | | Index of the last "free list" block |
| 48 | 4 | | "Free list" size, in number of blocks |
| 52 | 4 | | Number of free blocks in the allocation group |
| 56 | 4 | | Longest contiguous free space in the allocation group, in number of blocks |
| <td colspan="4">*Only used if the XFS_SB_VERSION2_LAZYSBCOUNTBIT feature flag is set*</td> |
| 60 | 4 | | Number of blocks used for the free space B+ trees |
| <td colspan="4">*If superblock format version >= 5 (XFS_SB_VERSION_5)*</td> |
| 64 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 80 | 4 | | Unknown (Size of the reverse mapping B+ tree in blocks) |
| 84 | 4 | | Unknown (Size of the reference count B+ tree in blocks) |
| 88 | 4 | | Reverse mapping B+ tree root block number, which contains a block number relative to the start of the allocation group |
| 92 | 4 | | Reference count B+ tree root block number, which contains a block number relative to the start of the allocation group |
| 96 | 14 x 8 | | Unknown (reserved) |
| 208 | 8 | | Log sequence number |
| 216 | 4 | | Unknown (Checksum of the free sector) |
| 220 | 4 | | Unknown (reserved) |

<!-- rumdl-enable MD033 MD056 -->

## Free list

A free list consists of:

* As of version 5, free list header
* Array of free block numbers

### Free list header

The free list header is 36 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "AGFL" | Signature |
| 4 | 4 | | Sequence number, which contains the allocation group number of the corresponding sector |
| 8 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 24 | 8 | | Log sequence number |
| 32 | 4 | | Checksum |

TODO: describe sb_uuid or sb_meta_uuid

## Inode information

The inode information (xfs_agi_t) is (at least) 512 bytes of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "XAGI" | Signature |
| 4 | 4 | 1 | Version |
| 8 | 4 | | Sequence number, which contains the allocation group number of the corresponding sector |
| 12 | 4 | | Unknown (Allocation group size), in number of blocks |
| 16 | 4 | | Number of inodes in the allocation group |
| 20 | 4 | | Inode B+ tree root block number, which contains a block number relative to the start of the allocation group |
| 24 | 4 | | Inode B+ tree height/depth |
| 28 | 4 | | Number of unused (free) inodes in the allocation group |
| 32 | 4 | | First inode number of the last allocated inode chunk, which contains an inode number relative to the allocation group |
| 36 | 4 | -1 (0xffffffff) | Unknown |
| 40 | 64 x 4 | | Hash table of 32-bit unlinked (deleted) inode numbers that are still being referenced, which contains -1 (0xffffffff) if not set |
| <td colspan="4">*If superblock format version >= 5 (XFS_SB_VERSION_5)*</td> |
| 296 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 312 | 4 | | Checksum |
| 316 | 4 | | Unknown (padding) |
| 320 | 8 | | Log sequence number |
| 328 | 4 | | Free inode B+ tree root block number, which contains a block number relative to the start of the allocation group |
| 332 | 4 | | Free inode B+ tree height/depth |
| 336 | 4 | | Unknown |
| 340 | 4 | | Unknown |
| 344 | 168 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

## B+ tree

XFS uses B+ trees to store various types of information. There are 2 different types of B+ trees,
namely:

* Free space block B+ tree
* Inode B+ tree
* Reference count B+ tree

### B+ tree block {#btree_block}

A B+ tree block consists of:

* B+ tree block header
* Array of branch or leaf block records

### B+ tree block header {#btree_block_header}

#### B+ tree block header 32-bit {#btree_block_header_32bit}

The B+ tree block header 32-bit (xfs_btree_sblock_t or xfs_btree_iblock_t) is 16 or 56 bytes of
size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Signature |
| 4 | 2 | | Level (or depth/height), which contains 0 for a leaf block |
| 6 | 2 | | Number of records |
| 8 | 4 | | Previous B+ tree block number, which is relative to the start of the allocation group or contains -1 (0xffffffff) if not set |
| 12 | 4 | | Next B+ tree block number, which is relative to the start of the allocation group or contains -1 (0xffffffff) if not set |
| <td colspan="4">*If superblock format version >= 5 (XFS_SB_VERSION_5)*</td> |
| 16 | 8 | | Block number |
| 24 | 8 | | Log sequence number |
| 32 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 48 | 4 | | Owner allocation group, which contains the allocation group the block is part of |
| 52 | 4 | | Checksum |

<!-- rumdl-enable MD033 MD056 -->

#### B+ tree block header 64-bit {#btree_block_header_64bit}

The B+ tree block header 64-bit (xfs_btree_lblock_t) is 24 or 68 bytes of size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Signature |
| 4 | 2 | | Level (or depth/height), where 0 represents a leaf block |
| 6 | 2 | | Number of records |
| 8 | 8 | | Previous B+ tree block number, which is relative to the start of the allocation group or contains -1 (0xffffffffffffffff) if not set |
| 16 | 8 | | Next B+ tree block number, which is relative to the start of the allocation group or contains -1 (0xffffffffffffffff) if not set |
| <td colspan="4">*If superblock format version >= 5 (XFS_SB_VERSION_5)*</td> |
| 24 | 8 | | Block number |
| 32 | 8 | | Log sequence number |
| 40 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 56 | 8 | | Owner allocation group, which contains the allocation group the block is part of |
| 64 | 4 | | Checksum |
| 64 | 4 | 0 | Unknown (padding) |

<!-- rumdl-enable MD033 MD056 -->

### B+ tree block extended header

TODO: complete section

TODO: determine where this is defined, it seems to be represented in the examples.

#### B+ tree block header signatures

| Signature | Description |
| --- | --- |
| "AB3B" | Free space block offset B+ tree (file system version 5) |
| "AB3C" | Free space block count B+ tree (file system version 5) |
| "ABTB" | Free space block offset B+ tree |
| "ABTC" | Free space block count B+ tree |
| "FIB3" | Free inode B+tree (file system version 5) |
| "FIBT" | Free inode B+tree |
| "IAB3" | (Allocated) inode B+tree (file system version 5) |
| "IABT" | (Allocated) inode B+tree |
| "R3FC" | Reference count B+ tree (file system version 5) |

### Free space B+ tree

TODO: complete section

#### Free space B+ tree branch node record

The free space B+ tree branch node record (xfs_alloc_ptr_t) is 4 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown |

#### Free space B+ tree leaf node record

The free space B+ tree leaf node record (xfs_alloc_key_t) is 8 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown (ar_startblock) |
| 4 | 4 | | Unknown (ar_blockcount) |

### Inode B+ tree

The inode B+ tree uses [the B+ tree block header 32-bit](#btree_block_header_32bit).

#### Inode B+ tree branch node

The inode B+ tree branch node consists of:

* node header
* array of inode B+ tree branch node entry keys
* array of inode B+ tree branch node entry values

The number of key-value pairs is calculated as following:

```python
number_of_key_value_pairs = node_records_data_size / 8
```

##### Inode B+ tree branch node key

The inode B+ tree branch node key (xfs_inobt_key_t) is 4 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of the first inode in the branch, which contains an inode number relative to the allocation group |

> Note that the inode number of the last key can be 0.

##### Inode B+ tree branch node value

The inode B+ tree branch node key is 4 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Block number of the inode B+ tree sub node, which contains a block number relative to the start of the allocation group |

#### Inode B+ tree leaf node record

The inode B+ tree leaf node record (xfs_inobt_rec_t) is 16 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of the first inode of the inode chunk, which contains an inode number relative to the allocation group |
| 4 | 4 | | Number of unused (free) inodes of the inode chunk |
| 8 | 8 | | Inode chunk allocation bitmap, which tracks which inodes of the inode chunk are unused (free) |

The inode chunk is a group of 64 inodes. The file offset of the inode chunk is calculated as
following:

```python
file_offset = allocation_group_file_offset + (inode_number * inode_size)
```

## Inode

The inode can be followed by:

* data fork (descriptor)
  * device identifier (fork type is XFS_DINODE_FMT_DEV)
  * inline data fork (fork type is XFS_DINODE_FMT_LOCAL)
  * extent list data fork (fork type is XFS_DINODE_FMT_EXTENTS)
  * extent B+ tree data fork (fork type is XFS_DINODE_FMT_BTREE)
* optional (extended) attributes data fork (descriptor)
  * inline attributes fork (fork type is XFS_DINODE_FMT_LOCAL)
  * extent list attributes fork (fork type is XFS_DINODE_FMT_EXTENTS)
  * extent B+ tree attributes fork (fork type is XFS_DINODE_FMT_BTREE)

### Inode version 1

The inode version 1 (xfs_dinode_core_t) is 100 bytes of size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "IN" | Signature |
| 2 | 2 | | [File mode](#file_mode), which contains file type and permissions |
| 4 | 1 | 1 | Format version |
| 5 | 1 | | (Data) [fork type](#fork_type) |
| 6 | 2 | | Number of links |
| 8 | 4 | | Owner (or user) identifier (UID) |
| 12 | 4 | | Group identifier (GID) |
| 16 | 14 | 0 | Unknown (contains data in XFS_SB_VERSION_1) |
| 30 | 2 | | Flush counter, which contains a value that is incremented on flush |
| 32 | 4 | | (last) access time, which contains a POSIX timestamp in seconds |
| 36 | 4 | | (last) access time fraction of second, which contains number of nanoseconds |
| 40 | 4 | | (last) modification time, which contains a POSIX timestamp in seconds |
| 44 | 4 | | (last) modification time fraction of second, which contains number of nanoseconds |
| 48 | 4 | | (last) inode change time, which contains a POSIX timestamp in seconds |
| 52 | 4 | | (last) inode change time fraction of second, which contains number of nanoseconds |
| 56 | 8 | | (Data) size |
| 64 | 8 | | Number of (data) blocks |
| 72 | 4 | | Extent size |
| 76 | 4 | | Number of data extents |
| 80 | 2 | | Number of (extended) attributes extents, which can contain 0 if an attributes fork of type XFS_DINODE_FMT_EXTENTS is empty |
| 82 | 1 | | (Extended) attributes fork descriptor offset, which contains an offset (value x 8) relative to the end of the inode |
| 83 | 1 | | (Extended) attributes fork type |
| 84 | 4 | | Unknown (DMAPI event mask) |
| 88 | 2 | | Unknown (DMAPI state) |
| 90 | 2 | | Inode flags |
| 92 | 4 | | Generation number |
| <td colspan="4">*Non-inode core field*</td> |
| 96 | 4 | | Unknown (next unlinked inode), which contains -1 (0xffffffff) if not set |

<!-- rumdl-enable MD033 MD056 -->

### Inode version 2

The inode version 2 (xfs_dinode_core_t) is 100 bytes of size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "IN" | Signature |
| 2 | 2 | | [File mode](#file_mode), which contains file type and permissions |
| 4 | 1 | 2 | Format version |
| 5 | 1 | | (Data) [fork type](#fork_type) |
| 6 | 2 | | Unknown |
| 8 | 4 | | Owner (or user) identifier (UID) |
| 12 | 4 | | Group identifier (GID) |
| 16 | 4 | | Number of links |
| 20 | 2 | | Project identifier |
| 22 | 8 | 0 | Unknown (padding) |
| 30 | 2 | | Flush counter, which contains a value that is incremented on flush |
| 32 | 4 | | (last) access time, which contains a POSIX timestamp in seconds |
| 36 | 4 | | (last) access time fraction of second, which contains number of nanoseconds |
| 40 | 4 | | (last) modification time, which contains a POSIX timestamp in seconds |
| 44 | 4 | | (last) modification time fraction of second, which contains number of nanoseconds |
| 48 | 4 | | (last) inode change time, which contains a POSIX timestamp in seconds |
| 52 | 4 | | (last) inode change time fraction of second, which contains number of nanoseconds |
| 56 | 8 | | (Data) size |
| 64 | 8 | | Number of (data) blocks |
| 72 | 4 | | Extent size |
| 76 | 4 | | Number of data extents |
| 80 | 2 | | Number of (extended) attributes extents, which can contain 0 if an attributes fork of type XFS_DINODE_FMT_EXTENTS is empty |
| 82 | 1 | | (Extended) attributes fork descriptor offset, which contains an offset (value x 8) relative to the end of the inode |
| 83 | 1 | | (Extended) attributes fork type |
| 84 | 4 | | Unknown (DMAPI event mask) |
| 88 | 2 | | Unknown (DMAPI state) |
| 90 | 2 | | Inode flags |
| 92 | 4 | | Generation number |
| <td colspan="4">*Non-inode core field*</td> |
| 96 | 4 | | Unknown (next unlinked inode), which contains -1 (0xffffffff) if not set |

<!-- rumdl-enable MD033 MD056 -->

### Inode version 3

The inode version 3 (xfs_dinode_core_t) is 176 bytes of size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "IN" | Signature |
| 2 | 2 | | [File mode](#file_mode), which contains file type and permissions |
| 4 | 1 | 3 | Format version |
| 5 | 1 | | (Data) [fork type](#fork_type) |
| 6 | 2 | | Unknown |
| 8 | 4 | | Owner (or user) identifier (UID) |
| 12 | 4 | | Group identifier (GID) |
| 16 | 4 | | Number of links |
| 20 | 2 | | Project identifier lower 16-bit |
| 22 | 2 | | Project identifier upper 16-bit |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_NREXT64 not is set*</td> |
| 24 | 8 | 0 | Unknown (padding) |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_NREXT64 is set*</td> |
| 24 | 8 | | Number of data extents (64-bit) |
| <td colspan="4">*Common*</td> |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_BIGTIME not is set*</td> |
| 32 | 4 | | (last) access time, which contains a POSIX timestamp in seconds |
| 36 | 4 | | (last) access time fraction of second, which contains number of nanoseconds |
| 40 | 4 | | (last) modification time, which contains a POSIX timestamp in seconds |
| 44 | 4 | | (last) modification time fraction of second, which contains number of nanoseconds |
| 48 | 4 | | (last) inode change time, which contains a POSIX timestamp in seconds |
| 52 | 4 | | (last) inode change time fraction of second, which contains number of nanoseconds |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_BIGTIME is set*</td> |
| 32 | 8 | | (last) access time, which contains a bigtime timestamp |
| 40 | 8 | | (last) modification time, which contains a bigtime timestamp |
| 48 | 8 | | (last) inode change time, which contains a bigtime timestamp |
| <td colspan="4">*Common*</td> |
| 56 | 8 | | (Data) size |
| 64 | 8 | | Number of (data) blocks |
| 72 | 4 | | Extent size |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_NREXT64 not is set*</td> |
| 76 | 4 | | Number of data extents |
| 80 | 2 | | Number of (extended) attributes extents, which can contain 0 if an attributes fork of type XFS_DINODE_FMT_EXTENTS is empty |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_NREXT64 is set*</td> |
| 76 | 4 | | Number of (extended) attributes extents (32-bit), which can contain 0 if an attributes fork of type XFS_DINODE_FMT_EXTENTS is empty |
| 80 | 2 | | Unknown (padding) |
| <td colspan="4">*Common*</td> |
| 82 | 1 | | (Extended) attributes fork descriptor offset, which contains an offset (value x 8) relative to the end of the inode |
| 83 | 1 | | (Extended) attributes fork type |
| 84 | 4 | | Unknown (DMAPI event mask) |
| 88 | 2 | | Unknown (DMAPI state) |
| 90 | 2 | | [Inode flags](#inode_flags) |
| 92 | 4 | | Generation number |
| <td colspan="4">*Pre version 3 non-inode core field*</td> |
| 96 | 4 | | Unknown (next unlinked inode), which contains -1 (0xffffffff) if not set |
| <td colspan="4">*Introduced in version 3*</td> |
| 100 | 4 | | Checksum |
| 104 | 8 | | Change count, which contains the number of changes made to the inode |
| 112 | 8 | | Log sequence number |
| 120 | 8 | | Extended inode flags |
| 128 | 4 | | Copy-on-write (COW) extent size |
| 132 | 12 | | Unknown (padding) |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_BIGTIME not is set*</td> |
| 144 | 4 | | Creation time, which contains a POSIX timestamp in seconds |
| 148 | 4 | | Creation time fraction of second, which contains number of nanoseconds |
| <td colspan="4">*If incompatible feature flag XFS_SB_FEAT_INCOMPAT_BIGTIME is set*</td> |
| 144 | 8 | | Creation time, which contains a bigtime timestamp |
| <td colspan="4">*Common*</td> |
| 152 | 8 | | Inode number, which contains an absolute inode number |
| 160 | 16 | | Inode type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |

<!-- rumdl-enable MD033 MD056 -->

### File mode {#file_mode}

<!-- rumdl-disable MD033 MD056 -->

| Value | Identifier | Description |
| --- | --- | --- |
| <td colspan="3">*Access other, bitmask: 0x0007 (S_IRWXO)*</td> |
| 0x0001 | S_IXOTH | X-access for other |
| 0x0002 | S_IWOTH | W-access for other |
| 0x0004 | S_IROTH | R-access for other |
| <td colspan="3">*Access group, bitmask: 0x0038 (S_IRWXG)*</td> |
| 0x0008 | S_IXGRP | X-access for group |
| 0x0010 | S_IWGRP | W-access for group |
| 0x0020 | S_IRGRP | R-access for group |
| <td colspan="3">*Access owner (or user), bitmask: 0x01c0 (S_IRWXU)*</td> |
| 0x0040 | S_IXUSR | X-access for owner (or user) |
| 0x0080 | S_IWUSR | W-access for owner (or user) |
| 0x0100 | S_IRUSR | R-access for owner (or user) |
| <td colspan="3">*Other*</td> |
| 0x0200 | S_ISTXT | Sticky bit |
| 0x0400 | S_ISGID | Set group identifer (GID) on execution |
| 0x0800 | S_ISUID | Set owner (or user) identifer (UID) on execution |
| <td colspan="3">*Type of file, bitmask: 0xf000 (S_IFMT)*</td> |
| 0x1000 | S_IFIFO | Named pipe (FIFO) |
| 0x2000 | S_IFCHR | Character device |
| 0x4000 | S_IFDIR | Directory |
| 0x6000 | S_IFBLK | Block device |
| 0x8000 | S_IFREG | Regular file |
| 0xa000 | S_IFLNK | Symbolic link |
| 0xc000 | S_IFSOCK | Socket |

<!-- rumdl-enable MD033 MD056 -->

### Fork type {#fork_type}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | XFS_DINODE_FMT_DEV | Device identifier is stored inline (in the inode) |
| 1 | XFS_DINODE_FMT_LOCAL | Data is stored inline (in the inode) |
| 2 | XFS_DINODE_FMT_EXTENTS | Data is referrenced by extents stored in [an extent list](#extent_list) |
| 3 | XFS_DINODE_FMT_BTREE | Data is referrence by extents stored in [an extent B+ tree](#extent_btree) |
| 4 | XFS_DINODE_FMT_UUID | Unknown (currently not used) |
| 5 | XFS_DINODE_FMT_RMAP | Data is referrence by a reverse mapping |

### Inode flags {#inode_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | XFS_DIFLAG_REALTIME | The data is located on the real-time device |
| 0x0002 | XFS_DIFLAG_PREALLOC | The extents have been preallocated |
| 0x0004 | XFS_DIFLAG_NEWRTBM | Uses the new real-time bitmap format |
| 0x0008 | XFS_DIFLAG_IMMUTABLE | Immutable (cannot be modified) |
| 0x0010 | XFS_DIFLAG_APPEND | Append only |
| 0x0020 | XFS_DIFLAG_SYNC | Use synchronous write |
| 0x0040 | XFS_DIFLAG_NOATIME | Do not update access time (atime) |
| 0x0080 | XFS_DIFLAG_NODUMP | Do not "dump", which indicates that xfsdump should ignore the file |
| 0x0100 | XFS_DIFLAG_RTINHERIT | Sub directories inherit XFS_DIFLAG_REALTIME |
| 0x0200 | XFS_DIFLAG_PROJINHERIT | Sub directories inherit the project identifier |
| 0x0400 | XFS_DIFLAG_NOSYMLINKS | No symbolic links can be created for sub directories |
| 0x0800 | XFS_DIFLAG_EXTSIZE | Has extent size |
| 0x1000 | XFS_DIFLAG_EXTSZINHERIT | Sub directories inherit extent size |
| 0x2000 | XFS_DIFLAG_NODEFRAG | Do not defragment |
| 0x4000 | XFS_DIFLAG_FILESTREAM | Unknown (Use filestream allocator) |

### Extent list {#extent_list}

The extent list consists of:

* one or more [packed extents](#packed_extent)

#### Packed extent {#packed_extent}

The packed extent (xfs_bmbt_rec_t) is 128 bits of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 21 bits | | Number of blocks |
| 2.4 | 52 bits | | Physical block number, which contains a [file system block number](#file_system_block_number) |
| 9.1 | 54 bits | | Logical block number |
| 15.7 | 1 bit | | Uninitialized (unwritten) extent |

### Extent B+ tree {#extent_btree}

#### Extent B+ tree root node

The root node of the extent B+ tree is stored in the inode and equivalent to
[an extent B+ tree branch node](#extent_btree_branch_node).

The number of key-value pairs is calculated as following:

```python
number_of_key_value_pairs = (node_data_size - 4) / 16
```

Where "node data size" is `(attributes_fork_descriptor_offset * 8)` if the value is not 0, or
otherwise the remaining inode block size.

#### Extent B+ tree sub node block

An extent B+ tree sub nodes is stored in [a B+ tree block](#btree_block).

The inode B+ tree uses [the B+ tree block header 64-bit](#btree_block_header_64bit).

##### Extent B+ tree sub node block header

The sub node block header (xfs_bmbt_block_t) is equivalent to
[B+ tree block header](#btree_block_header).

##### Extent B+ tree sub node block header signatures

| Signature | Description |
| --- | --- |
| "BMA3" | File system version 5 extent B+ tree sub node block |
| "BMAP" | Extent B+ tree sub node block |

#### Extent B+ tree branch node {#extent_btree_branch_node}

The extent B+ tree branch node record consists of:

* node header
* array of extent B+ tree branch node entry keys
* array of extent B+ tree branch node entry values

TODO: number of key-value pairs

The number of key-value pairs is calculated as following:

```python
number_of_key_value_pairs = node_records_data_size / 16
```

##### Extent B+ tree branch node header

The branch node header (xfs_bmdr_block_t) is 4 byte of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Node level |
| 2 | 2 | | Number of used key-value pairs in the node |

##### Extent B+ tree branch node entry key

The branch node entry key (xfs_bmbt_key_t) is 8 byte of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Data offset |

##### Extent B+ tree branch node entry value

The branch node entry value (xfs_bmbt_ptr_t or xfs_bmdr_ptr_t) is 8 byte of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Block number of the extent B+ tree sub node, which contains a [file system block number](#file_system_block_number) |

#### Extent B+ tree leaf block node

The extent B+ tree leaf block node consists of:

* one or more [packed extents](#packed_extent)

## Directory entries

Directories entries are stored in the data fork of a directory inode. The directory entries can be
stored in multiple ways:

* as a short-form directory table
* as an extent-based block directory (or leaf directory)
* as an extent-based directory B+ tree (or node directory)

### Short-form directory table

The short-form directory table (xfs_dir_sf_t or xfs_dir2_sf_t) is stored in the inode (as inline
data), where fork type is XFS_DINODE_FMT_LOCAL. The short-form directory table consist of:

* Short-form directory table header
* Short-form directory table entries

The XFS_SB_VERSION_DIRV2BIT flag in the superblock indicates if version 2 is used.

#### Short-form directory table header version 1

The short-form directory table header version 1 (xfs_dir_sf_hdr_t) is 9 bytes of size and consists
of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Parent inode number, which contains an absolute inode number |
| 8 | 1 | | Number of entries |

#### Short-form directory table header version 2

The short-form directory table header version 2 (xfs_dir2_sf_hdr_t) is 6 or 10 bytes of size and
consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Number of 32-bit entries |
| 1 | 1 | | Number of 64-bit entries |
| 2 | 4 or 8 | | Parent inode number, which contains an absolute inode number |

> Note that if the inode numbers are stored as 32-bit values then number of 32-bit entries is set
> and number of 64-bit entries must be 0. If the inode numbers are stored as 64-bit values then
> number of 64-bit entries is set and number of 32-bit entries must be 0.

#### Short-form directory table entry version 1

The short-form directory table entry version 1 (xfs_dir_sf_entry_t) is variable of size and
consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Inode number, which contains an absolute inode number |
| 9 | 1 | | Name size, which does not include the end-of-string character |
| 10 | ... | | Name |

#### Short-form directory table entry version 2

The short-form directory table entry version 2 (xfs_dir2_sf_entry_t) is variable of size and
consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Name size, which does not include the end-of-string character |
| 1 | 2 | | Unknown (offset, tag) |
| 3 | ... | | Name |
| <td colspan="4">*Only present if XFS_SB_VERSION2_FTYPE is set*</td> |
| ... | 1 | | File type |
| <td colspan="4">*Common*</td> |
| ... | 4 or 8 | | Inode number, which contains an absolute inode number |

<!-- rumdl-enable MD033 MD056 -->

> Note that file type seems to be present on format version even if XFS_SB_VERSION2_FTYPE is not
> set.

### Directory (leaf) block {#directory_leaf_block}

A directory (leaf) block (xfs_dir_leafblock_t) consist of:

* a block header
* array of block entries
* array of block values
* a block footer

If more than one block is needed to store the directory entries
[a directory B+ tree](#directory_btree) is used.

> Note that if the XFS_SB_VERSION_DIRV2BIT flag in the superblock is set the directory is stored
> using [a block directory](#block_directory) instead.

#### Directory (leaf) block header version 1

A directory (leaf) block header version 1 (xfs_dir_leaf_hdr_t) is 16 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 12 | | [File system block header version 1](#file_system_block_header_version_1) |
| 12 | 2 | | Number of entries |
| 14 | 2 | | Used (block) data size, in number of bytes |
| 16 | 2 | | Used data offset |
| 18 | 1 | | Flag to indicate block compaction is needed |
| 19 | 1 | | Unknown (padding) |
| 20 | 4 x 3 | | Array of [free regions](#block_free_region_v2) in the block |

> Note that a directory (leaf) block header version 2 (xfs_dir2_leaf_hdr_t) is equivalent to
> version 1.

#### Directory (leaf) block entry version 1

The directory (leaf) block entry (xfs_dir_leaf_entry_t) is 8 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Name hash |
| 4 | 2 | | Values offset, which contains an offset relative to the start of the directory block |
| 6 | 1 | | Name size, which does not include the end-of-string character |
| 7 | 1 | | Unknown (padding) |

> Note that a directory (leaf) block entry version 2 (xfs_dir2_leaf_entry_t) is equivalent to
> version 1.

#### Directory (leaf) block values

The directory (leaf) block value (xfs_dir_leaf_name_t) is variable of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Inode number, which contains an absolute inode number |
| 8 | ... | | Name |

#### Directory (leaf) block footer version 1

A directory (leaf) block footer version 1 (xfs_dir_leaf_tail_t) is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown (bestcount) |

> Note that a directory (leaf) block footer version 2 (xfs_dir2_leaf_tail_t) is equivalent to
> version 1.

### Block directory {#block_directory}

A block directory (xfs_dir2_block_t) consist of one or more blocks that consist of:

* a block directory header
* array of used and unused directory entries
* hash values of the entries
* a block directory footer

> Note that if the XFS_SB_VERSION_DIRV2BIT flag in the superblock is not set the directory is
> stored using [a directory (leaf) block](#directory_leaf_block) instead.

#### Block directory header

##### Block directory header version 2

The block directory header version 2 (xfs_dir2_data_hdr_t) is 16 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "XD2B" or "XD2D" | Signature |
| 4 | 4 x 3 | | Array of [free regions](#block_free_region_v2) in the block |

##### Block directory header version 3

The block directory header version 3 (xfs_dir3_data_hdr_t) is 64 bytes of size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header (xfs_dir3_blk_hdr_t)*</td> |
| 0 | 4 | "XDB3" or "XDD3" | Signature |
| 4 | 4 | | Checksum |
| 8 | 8 | | Block number |
| 16 | 8 | | Log sequence number |
| 24 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 40 | 8 | | Owner inode number, which contains the absolute inode number the block is part of |
| <td colspan="4">&nbsp;</td> |
| 48 | 4 x 3 | | Array of [free regions](#block_free_region_v2) in the block |
| 60 | 4 | | Unknown (padding) |

<!-- rumdl-enable MD033 MD056 -->

##### Block directory header version signatures

| Signature | Description |
| --- | --- |
| "XD2B" | Version 2 directory entries B+ tree (single block) |
| "XD2D" | Version 2 directory entries B+ tree (multi block) |
| "XD2F" | Version 2 directory free space B+ tree |
| "XDB3" | Version 3 directory entries B+ tree (single block) |
| "XDD3" | Version 3 directory entries B+ tree (multi block) |
| "XDF3" | Version 3 directory free space B+ tree |

##### Block free region version 2 {#block_free_region_v2}

The block free region version 2 (xfs_dir2_data_free_t) is 4 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Free region offset, which is relative to the start of the directory block |
| 2 | 2 | | Free region size |

#### Block directory entries

##### Block directory entry version 2

The block directory entry version 2 (xfs_dir2_data_entry_t) is variable of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Inode number, which contains an absolute inode number |
| 8 | 1 | | Name size, which does not include the end-of-string character |
| 9 | ... | | Name |
| <td colspan="4">*Only present if XFS_SB_VERSION2_FTYPE is set*</td> |
| ... | 1 | | Unknown (ftype) |
| <td colspan="4">*Common*</td> |
| ... | ... | | Unknown (8-byte alignment padding?) |
| ... | 2 | | Unknown (offset, tag) |

<!-- rumdl-enable MD033 MD056 -->

##### Unused block directory entry version 2

The unused block directory entry version 2 (xfs_dir2_data_unused_t) is variable of size
and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0xffff | Signature (free tag) |
| 2 | 2 | | Entry size, which contains the size of the unused entry including the size of the signature and entry size |
| 4 | 2 | | Unknown (padding) |
| ... | 2 | | Unknown (offset, tag) |

#### Block directory hash value

The block directory hash value (xfs_dir_leaf_entry_t or xfs_dir2_leaf_entry_t) is 8 bytes of size
and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Hash value of the name of the directory entry |
| 4 | 4 | | Entry offset, which is relative to the start of the block |

#### Block directory footer

##### Block directory footer version 2

The block directory footer version 2 (xfs_dir2_block_tail_t) is 8 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of used entries |
| 4 | 4 | | Number of unused entries |

### Directory or attributes B+ tree {#directory_btree}

The first block in the extents is the B+ tree root block.

#### Directory or attributes B+ tree branch node block {#directory_btree_branch_node_block}

A directory or attributes B+ tree branch node block consist of:

* [a directory or attributes branch node block header](#directory_branch_node_block_header)
* array of directory or attributes branch node entries

##### Directory or attributes branch node block header {#directory_branch_node_block_header}

A directory or attributes branch node block header is 16 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 12 | | [File system block header version 1](#file_system_block_header_version_1) |
| 12 | 2 | | Number of entries |
| 14 | 2 | | Node level |

##### Directory or attributes B+ tree branch node block entry

A directory or attributes B+ tree branch node block entry is 8 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Name hash |
| 4 | 4 | | Sub block number, which contains a block number relative to the start of the attributes extents |

## Device identifier

Character and block devices identifiers are stored as inline data with fork type is
XFS_DINODE_FMT_DEV.

The device identifier (xfs_dev_t) is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 18 bits | | Minor device number |
| 2.2 | 14 bits | | Major device number |

## File content

XFS supports multiple ways to store file content:

* inline data (fork type is XFS_DINODE_FMT_LOCAL)
* extents defined by either an extent list (fork type is XFS_DINODE_FMT_EXTENTS) or an extent
  B+ tree (fork type is XFS_DINODE_FMT_BTREE)

### Inline data

The file content data is stored in the inode data fork.

### Extents

The file content data is stored in the block defined by the extents.

If the logical block numbers of successive extents are non-contiguous this means the file content
data has an implicit sparse extent (or hole).

TODO: determine if the hole can be at the start or end of the file content data.

## File system block B+ tree

The file system block B+ tree is a structure used to store the directory and attributes B+ trees

### File system block header {#file_system_block_header}

#### File system block header version 1 {#file_system_block_header_version_1}

If the superblock format version <= 4 the file system block header version 1 is used. The file
system block header version 1 (xfs_da_blkinfo_t) is 12 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | (Logical) block number of the next B+ tree block at the same level |
| 4 | 4 | | (Logical) block number of the previous B+ tree block at the same level |
| 8 | 2 | 0xfbee | Signature (XFS_ATTR_LEAF_MAGIC) |
| 10 | 2 | | Unknown (padding) |

> Note that a file system block header version 2 is equivalent to version 1.

#### File system block header version 3 {#file_system_block_header_version_3}

If the superblock format version >= 5 the file system block header version 3 is used. The file
system block header version 3 (xfs_da3_blkinfo_t) is 56 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | (Logical) block number of the next B+ tree block at the same level |
| 4 | 4 | | (Logical) block number of the previous B+ tree block at the same level |
| 8 | 2 | 0x3bee | Signature (XFS_ATTR3_LEAF_MAGIC) |
| 10 | 2 | | Unknown (padding) |
| 12 | 4 | | Checksum |
| 16 | 8 | | Block number |
| 24 | 8 | | Log sequence number |
| 32 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 48 | 8 | | Owner inode number, which contains the absolute inode number the block is part of |

#### File system block header signatures

| Signature | Identifier | Description |
| --- | --- | --- |
| 0x3bee | XFS_ATTR_LEAF_MAGIC | File system version 5 attributes B+ tree leaf block |
| 0x3ebe | XFS_DA3_NODE_MAGIC | File system version 5 directory or attributes B+ tree branch block |
| 0xd2f1 | XFS_DIR2_LEAF1_MAGIC | |
| 0xd2ff | XFS_DIR2_LEAFN_MAGIC | |
| 0xfbee | XFS_ATTR_LEAF_MAGIC | Attributes B+ tree leaf block |
| 0xfebe | XFS_DA_NODE_MAGIC | Directory or attributes [B+ tree branch block](#directory_btree_branch_node_block) |
| 0xfeeb | XFS_DIR_LEAF_MAGIC | Directory B+ tree leaf block |

## Extended attributes

Extended attributes are stored in the attributes fork of an inode. The extended attributes can be
stored in multiple ways:

* as a short-form attributes table
* as an extent-based [attributes block](#attributes_block) (or leaf attributes)
* as an extent-based [attributes B+ tree](#attributes_btree) (or node attributes)

The start of the attributes fork can be determined using the attributes fork descriptor offset.

### Short-form attributes table

If the inode attributes fork type is XFS_DINODE_FMT_LOCAL the extended attributes are stored in a
short-form attributes table (xfs_attr_shortform) inline in the attribtes fork. The short-form
attributes table consist of:

* a short-form attribute table header
* one or more short-form attribute table entries

#### The short-form attribute table header

The short-form attribute table header (xfs_attr_sf_hdr) is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Data size, which contains the size of the short form attributes table data |
| 2 | 1 | | Number of entries |
| 3 | 1 | | Unknown (padding?) |

> Note that the size of the short-form attribute header deviates from `[SGI18]` based on analysis
> of test data.

#### The short-form attribute entry

The short-form attribute table entry (xfs_attr_sf_entry) is variable of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Name size |
| 1 | 1 | | Value data size |
| 2 | 1 | | [Attribute flags](#attribute_flags) |
| 3 | ... | | Name string, which contains an ASCII string without end-of-string character |
| ... | ... | | Value data |

### Attributes (leaf) block {#attributes_block}

If the inode attributes fork type is XFS_DINODE_FMT_EXTENTS the extended attributes are stored in
an attributes (leaf) block. The attributes fork contains [an extent list](#extent_list).

An attributes (leaf) block (xfs_attr_leafblock_t or xfs_attr3_leafblock_t) consist of:

* [an attributes (leaf) block header](#attributes_leaf_block_header)
* array of block entries
* array of local or remote attribute block values

If more than one block is needed to store the extended attributes
[an attributes B+ tree](#attributes_btree) is used.

> Note that since extended attributes were introduced in superblock format version 2 there are no
> version 1 structures.

#### Attributes (leaf) block header {#attributes_leaf_block_header}

##### Attributes (leaf) block header version 2

If the superblock format version <= 4 the attributes (leaf) block header version 2 is used. The
attributes (leaf) block header version 2 (xfs_attr_leaf_hdr_t) is 32 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 12 | | [File system block header version 1](#file_system_block_header_version_1) |
| 12 | 2 | | Number of entries |
| 14 | 2 | | Used (block) data size, in number of bytes |
| 16 | 2 | | Used data offset |
| 18 | 1 | | Flag to indicate block compaction is needed |
| 19 | 1 | | Unknown (padding) |
| 20 | 4 x 3 | | Array of [free regions](#block_free_region_v2) in the block |

##### Attributes (leaf) block header version 3

If the superblock format version >= 5 the attributes (leaf) block header version 3 is used. The
attributes (leaf) block header version 3 (xfs_attr3_leaf_hdr_t) is 80 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 56 | | [File system block header version 3](#file_system_block_header_version_3) |
| 56 | 2 | | Number of entries |
| 58 | 2 | | Used (block) data size, in number of bytes |
| 60 | 2 | | Used data offset |
| 62 | 1 | | Flag to indicate block compaction is needed |
| 63 | 1 | | Unknown (padding) |
| 64 | 4 x 3 | | Array of [free regions](#block_free_region_v2) in the block |
| 76 | 4 | | Unknown (padding) |

#### Attributes (leaf) block entry

The attributes (leaf) block entry (xfs_attr_leaf_entry_t) is 8 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Name hash |
| 4 | 2 | | Values offset, which is relative to the start of the attributes block |
| 6 | 1 | | [Attribute flags](#attribute_flags) |
| 7 | 1 | | Unknown (padding) |

#### Attribute (leaf) block values

If the attributes (leaf) block entry flag XFS_ATTR_LOCAL is set the attribute values are stored as
local attribute block values otherwise as remote attribute block values. The value data of remote
attribute values are stored in
[a remote attribute value data block](#remote_attribute_value_data_block).

##### Local attribute block values

The local attributes values (xfs_attr_leaf_name_local_t) is variable of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Value data size |
| 2 | 1 | | Name size |
| 3 | ... | | Name string, which contains an ASCII string without end-of-string character |

##### Remote attribute block values

The remote attributes values (xfs_attr_leaf_name_remote_t) is variable of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Value data block number, which is relative to the start of the attributes extents |
| 4 | 4 | | Value data size |
| 8 | 1 | | Name size |
| 9 | ... | | Name string, which contains an ASCII string without end-of-string character |

### Attributes B+ tree {#attributes_btree}

If the inode attributes fork type is XFS_DINODE_FMT_BTREE the extended attributes are stored in an
attributes B+ tree. The attributes fork contains [an extent B+ tree](#extent_btree).

The first block in the extents is the B+ tree root block.

### Attributes B+ tree branch node block

An attributes B+ tree branch node block consist of:

* [an attributes branch node block header](#attributes_branch_node_block_header)
* array of attribute branch node entry

#### Attributes branch node block header {#attributes_branch_node_block_header}

##### Attributes branch node block header version 2

If the superblock format version <= 4 the attributes branch node block header version 2 is used.
The attributes branch node block header version 2 (xfs_da_blkinfo_t) is 16 bytes of size and
consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 12 | | [File system block header version 1](#file_system_block_header_version_1) |
| 12 | 2 | | Number of entries |
| 14 | 2 | | Node level |

##### Attributes branch node block header version 3

If the superblock format version >= 5 the attributes branch node block header version 3 is used.
The attributes branch node block header version 3 (xfs_da3_blkinfo_t) is 64 bytes of size and
consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 56 | | [File system block header version 3](#file_system_block_header_version_3) |
| 56 | 2 | | Number of entries |
| 58 | 2 | | Node level |
| 60 | 4 | | Unknown (padding) |

##### Attributes branch node block entry

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Name hash |
| 4 | 4 | | Sub block number, which is relative to the start of the attributes extents |

### Attributes B+ tree leaf node

The extended attributes B+ tree leaf node is equivalent to an [attributes block](#attributes_block).

### The attribute flags {#attribute_flags}

The attribute flags indicate the prefix (or namespace) of the attribute name.

| Value | Identifier | Name prefix | Description |
| --- | --- | --- | --- |
| 0x00 | | "user." | The attribute is part of the user namespace |
| 0x01 | XFS_ATTR_LOCAL | | The attribute value is contained within the current block |
| 0x02 | XFS_ATTR_ROOT | "trusted." | The attribute is part of the trusted namespace |
| 0x04 | XFS_ATTR_SECURE | "secure." | The attribute is part of the secure namespace |
| 0x08 | XFS_ATTR_PARENT | | The attribute contains a parent directory reference, where the attribute name contains the file tentry name and the attribute value data [a reference to the parent directory](#parent_directory_attribute_value_data) |
| | | | |
| 0x80 | XFS_ATTR_INCOMPLETE | | The attribute is being modified |

#### Remote attribute value data block {#remote_attribute_value_data_block}

If the superblock format version <= 4 the attribute value data is stored directly in remote
attribute value date blocks.

If the superblock format version >= 5 each individual remote attribute value data block will start
with a remote attribute value data block header version 3 followed by attribute value data.

The attributes extents contain the physical location of the individual remote attribute value data
blocks.

##### Remote attribute value data block header version 3

The remote attribute value data block header (xfs_attr3_rmt_hdr) is 52 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "XARM" | Signature |
| 4 | 4 | | Value data offset |
| 8 | 4 | | Value data size |
| 12 | 4 | | Checksum of the remote attribute value data block |
| 16 | 16 | | Block type identifier, which contains an UUID that should correspond to sb_uuid or sb_meta_uuid |
| 32 | 8 | | Owner inode number, which contains the absolute inode number the block is part of |
| 40 | 8 | | Block number |
| 48 | 8 | | Log sequence number |

##### Parent directory attribute value data {#parent_directory_attribute_value_data}

The parent directory attribute value data (xfs_parent_rec) is 12 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Parent directory inode number |
| 8 | 4 | | Parent directory generation number |

## References

* [XFS Filesystem Structure - 3rd Edition](https://mirrors.edge.kernel.org/pub/linux/utils/fs/xfs/docs/xfs_filesystem_structure.pdf)
* [XFS Filesystem Structure](https://kernel.googlesource.com/pub/scm/fs/xfs/xfs-documentation/+/master/design/XFS_Filesystem_Structure)
