# Extended File System (ext) format

The Extended File System (ext) is one of the more common file system used in
Linux.

There are multiple version of ext.

| Version | Remarks
| --- | ---
| 1 | Introduced in April 1992
| 2 | Introduced in January 1993
| 3 | Introduced in November 2001, which featured journaling, dynamic growth and large directory indexing (HTree)
| 4 | Introduces in October 2006 as unstable and becmae stable in October 2008, which featured extents and improved timestamps

## Overview

An Extended File System (ext) consists of:

* one or more block groups

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian, with the exception of UUID values that are stored in big-endian.
| Date and time values | number of seconds since January 1, 1970 00:00:00 (POSIX epoch), disregarding leap seconds. Or number of nanoseconds, when extra precision is enabled. Date and time values are stored in UTC.
| Character strings | UTF-8 or a narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a system defined codepage.

## Block group

A block group consists of:

* optional 1024 bytes of boot code or zero bytes (at offset: 0)
* optional superblock
* optional group descriptor table
* block bitmap
* inode bitmap
* allocated and unallocated blocks

The primary superblock is stored at offset 1024 relative from the start of the
volume. Backup superblocks are stored at offset 1024 relative from the start of
the block group if block size <= 1024 or otherwise at offset 0 from the start
of the block group.

The group descriptor table is stored in the block after the superblock.

The ext2 revision 0 stores a copy at the start of every block group, along with
backups of the group descriptor table. Later revisions can reduce the number of
backup copies by only putting backups in specific groups (sparse superblock
feature EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER).

> Note that not all values in a backup superblock and backup group descriptor
> tables always match those of the primary superblock and group descriptor
> table.

### Flex block groups

Flex (or flexible) block groups are a set of block groups that treated as
a single logical block group. Metadata such as the superblock, group
descriptors, data block bitmaps spans the entire logical block group and
not the individual block groups part of the set.

### Meta block groups

Meta block groups (META_BG) are a set (or cluster) of block groups, for which
its group descriptor structures can be stored in a single block.

The first meta block group value in the superblock indicates what the first

meta block group value is 256, and the number of group descriptors that can be
stored in a single block 64, then the group descriptors for the block groups
\[0, 16383\] are stored in the group descriptor table after the primary
superblock and corresponding locations of backups.

Successive group descriptor tables, for example \[16384, 16447\], are stored in
the first block group of a meta block group and backups in the second and last
block groups of the meta block group.

### Blocks

The volume is devided in blocks:

```
block offset = block number x block size
```

The block size is defined in the superblock.

## The superblock

### The ext2 superblock

The ext2 superblock is 208 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Number of inodes
| 4 | 4 | | Number of blocks
| 8 | 4 | | Number of reserved blocks. Reserved blocks are used to prevent the file system from filling up.
| 12 | 4 | | Number of unallocated blocks
| 16 | 4 | | Number of unallocated inodes
| 20 | 4 | | First data block number. The block number is relative from the start of the volume
| 24 | 4 | | Block size, which contains the number of bits to shift 1024 to the MSB (left)
| 28 | 4 | | Fragment size, which contains the number of bits to shift 1024 to the MSB (left)
| 32 | 4 | | Number of blocks per block group
| 36 | 4 | | Number of fragments per block group
| 40 | 4 | | Number of inodes per block group
| 44 | 4 | | Last mount time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 48 | 4 | | Last written time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 52 | 2 | | The (current) mount count
| 54 | 2 | | Maximum mount count
| 56 | 2 | "\x53\xef" | Signature
| 58 | 2 | | [File system state flags](#file_system_state_flags)
| 60 | 2 | | [Error-handling status](#error_handling_status)
| 62 | 2 | | Minor format revision
| 64 | 4 | | Last consistency check time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 68 | 4 | | Consistency check interval, which which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 72 | 4 | | [Creator operating system](#creator_operating_system)
| 76 | 4 | | [Format revision](#format_revisision)
| 80 | 2 | | Reserved block owner (or user) identifier (UID)
| 82 | 2 | | Reserved block group identifier (GID)
| <td colspan="4"> *Dynamic inode information, if major version is EXT2_DYNAMIC_REV*
| 84 | 4 | | First non-reserved inode
| 88 | 2 | | Inode size. Note that the inode size must be a power of 2 larger or equal to 128, the maximum supported by mke2fs is 1024
| 90 | 2 | | Block group, which contains a block group number
| 92 | 4 | | [Compatible features flags](#compatible_features_flags)
| 96 | 4 | | [Incompatible features flags](#incompatible_features_flags)
| 100 | 4 | | [Read-only compatible features flags](#read_only_compatible_features_flags)
| 104 | 16 | | File system identifier, which contains a big-endian UUID
| 120 | 16 | | Volume label, which contains a narrow character string without end-of-string character
| 136 | 64 | | Last mount path, which contains a narrow character string without end-of-string character
| 200 | 4 | | Algorithm usage bitmap
| <td colspan="4"> *Performance hints, if EXT2_COMPAT_PREALLOC is set*
| 204 | 1 | | Number of pre-allocated blocks per file
| 205 | 1 | | Number of pre-allocated blocks per directory
| 206 | 2 | | Unknown (padding)

### The ext3 superblock

The ext3 superblock is 336 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Number of inodes
| 4 | 4 | | Number of blocks
| 8 | 4 | | Number of reserved blocks. Reserved blocks are used to prevent the file system from filling up.
| 12 | 4 | | Number of unallocated blocks
| 16 | 4 | | Number of unallocated inodes
| 20 | 4 | | First data block number. The block number is relative from the start of the volume
| 24 | 4 | | Block size, which contains the number of bits to shift 1024 to the MSB (left)
| 28 | 4 | | Fragment size, which contains the number of bits to shift 1024 to the MSB (left)
| 32 | 4 | | Number of blocks per block group
| 36 | 4 | | Number of fragments per block group
| 40 | 4 | | Number of inodes per block group
| 44 | 4 | | Last mount time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 48 | 4 | | Last written time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 52 | 2 | | The (current) mount count
| 54 | 2 | | Maximum mount count
| 56 | 2 | "\x53\xef" | Signature
| 58 | 2 | | [File system state flags](#file_system_state_flags)
| 60 | 2 | | [Error-handling status](#error_handling_status)
| 62 | 2 | | Minor format revision
| 64 | 4 | | Last consistency check time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 68 | 4 | | Consistency check interval, which which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 72 | 4 | | [Creator operating system](#creator_operating_system)
| 76 | 4 | | [Format revision](#format_revisision)
| 80 | 2 | | Reserved block owner (or user) identifier (UID)
| 82 | 2 | | Reserved block group identifier (GID)
| <td colspan="4"> *Dynamic inode information, if major version is EXT2_DYNAMIC_REV*
| 84 | 4 | | First non-reserved inode
| 88 | 2 | | Inode size. Note that the inode size must be a power of 2 larger or equal to 128, the maximum supported by mke2fs is 1024
| 90 | 2 | | Block group, which contains a block group number
| 92 | 4 | | [Compatible features flags](#compatible_features_flags)
| 96 | 4 | | [Incompatible features flags](#incompatible_features_flags)
| 100 | 4 | | [Read-only compatible features flags](#read_only_compatible_features_flags)
| 104 | 16 | | File system identifier, which contains a big-endian UUID
| 120 | 16 | | Volume label, which contains a narrow character string without end-of-string character
| 136 | 64 | | Last mount path, which contains a narrow character string without end-of-string character
| 200 | 4 | | Algorithm usage bitmap
| <td colspan="4"> *Performance hints, if EXT2_COMPAT_PREALLOC is set*
| 204 | 1 | | Number of pre-allocated blocks per file
| 205 | 1 | | Number of pre-allocated blocks per directory
| 206 | 2 | | Unknown (padding)
| <td colspan="4"> *Journalling support, if EXT3_FEATURE_COMPAT_HAS_JOURNAL is set*
| 208 | 16 | | Journal identifier, which contains a big-endian UUID
| 224 | 4 | | Journal inode
| 228 | 4 | | Unknown (Journal device)
| 232 | 4 | | Unknown (Head of orphan inode list). The orphan inode list is a list of inodes to delete.
| 236 | 4 x 4 | | hash-tree seed
| 252 | 1 | | Default hash version
| 253 | 1 | | Journal backup type
| 254 | 2 | | Group descriptor size
| 256 | 4 | | Default mount options
| 260 | 4 | | First metadata block group (or metablock)
| 264 | 4 | | File system creation time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 268 | 17 x 4 | | Backup journal inodes

### The ext4 superblock

The superblock is 1024 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Number of inodes
| 4 | 4 | | Number of blocks, which contains the lower 32-bit of the value
| 8 | 4 | | Number of reserved blocks, which contains the lower 32-bit of the value. Reserved blocks are used to prevent the file system from filling up.
| 12 | 4 | | Number of unallocated blocks, which contains the lower 32-bit of the value
| 16 | 4 | | Number of unallocated inodes, which contains the lower 32-bit of the value
| 20 | 4 | | Root group block number. The block number is relative from the start of the volume
| 24 | 4 | | Block size, which contains the number of bits to shift 1024 to the most-significant-bit (MSB)
| 28 | 4 | | Fragment size, which contains the number of bits to shift 1024 to the most-significant-bit (MSB)
| 32 | 4 | | Number of blocks per block group
| 36 | 4 | | Number of fragments per block group
| 40 | 4 | | Number of inodes per block group
| 44 | 4 | | Last mount time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 48 | 4 | | Last written time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 52 | 2 | | The (current) mount count
| 54 | 2 | | Maximum mount count
| 56 | 2 | "\x53\xef" | Signature
| 58 | 2 | | [File system state flags](#file_system_state_flags)
| 60 | 2 | | [Error-handling status](#error_handling_status)
| 62 | 2 | | Minor format revision
| 64 | 4 | | Last consistency check time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 68 | 4 | | Consistency check interval, which which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 72 | 4 | | [Creator operating system](#creator_operating_system)
| 76 | 4 | | [Format revision](#format_revisision)
| 80 | 2 | | Reserved block owner (or user) identifier (UID)
| 82 | 2 | | Reserved block group identifier (GID)
| <td colspan="4"> *Dynamic inode information, if major version is EXT2_DYNAMIC_REV*
| 84 | 4 | | First non-reserved inode
| 88 | 2 | | Inode size. Note that the inode size must be a power of 2 larger or equal to 128, the maximum supported by mke2fs is 1024
| 90 | 2 | | Block group
| 92 | 4 | | [Compatible features flags](#compatible_features_flags)
| 96 | 4 | | [Incompatible features flags](#incompatible_features_flags)
| 100 | 4 | | [Read-only compatible features flags](#read_only_compatible_features_flags)
| 104 | 16 | | File system identifier, which contains a big-endian UUID
| 120 | 16 | | Volume label, which contains a narrow character string without end-of-string character
| 136 | 64 | | Last mount path, which contains a narrow character string without end-of-string character
| 200 | 4 | | Algorithm usage bitmap
| <td colspan="4"> *Performance hints, if EXT2_COMPAT_PREALLOC is set*
| 204 | 1 | | Number of pre-allocated blocks per file
| 205 | 1 | | Number of pre-allocated blocks per directory
| 206 | 2 | | Unknown (padding)
| <td colspan="4"> *Journalling support, if EXT3_FEATURE_COMPAT_HAS_JOURNAL is set*
| 208 | 16 | | Journal identifier, which contains a big-endian UUID
| 224 | 4 | | Journal inode
| 228 | 4 | | Unknown (Journal device)
| 232 | 4 | | Unknown (Head of orphan inode list). The orphan inode list is a list of inodes to delete.
| 236 | 4 x 4 | | hash-tree seed
| 252 | 1 | | Default hash version
| 253 | 1 | | Journal backup type
| 254 | 2 | | Group descriptor size
| 256 | 4 | | Default mount options
| 260 | 4 | | First metadata block group (or metablock)
| 264 | 4 | | File system creation time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 268 | 17 x 4 | | Backup journal inodes
| <td colspan="4"> *If 64-bit support (EXT4_FEATURE_INCOMPAT_64BIT) is enabled*
| 336 | 4 | | Number of blocks, which contains the upper 32-bit of the value
| 340 | 4 | | Number of reserved blocks, which contains the upper 32-bit of the value
| 344 | 4 | | Number of unallocated blocks, which contains the upper 32-bit of the value
| 348 | 2 | | Minimum inode size
| 350 | 2 | | Reserved inode size
| 352 | 4 | | Miscellaneous flags
| 356 | 2 | | RAID stride
| 358 | 2 | | Multiple mount protection (MMP) update interval in seconds
| 360 | 8 | | Block for multi-mount protection
| 368 | 4 | | Unknown (blocks on all data disks (N\*stride))
| 372 | 1 | | Flex block group size, where the size is stored as: 2 ^ value
| 373 | 1 | | [Checksum type](#checksum_type)
| 374 | 1 | | Unknown (encryption level)
| 375 | 1 | | Unknown (padding)
| 376 | 8 | | Unknown (s_kbytes_written)
| 384 | 4 | | Inode number of active snapshot
| 388 | 4 | | Identifier of active snapshot
| 392 | 8 | | Unknown (reserved s_snapshot_r_blocks_count)
| 400 | 4 | | Inode number of snapshot list head
| 404 | 4 | | Unknown (s_error_count)
| 408 | 4 | | Unknown (s_first_error_time)
| 412 | 4 | | Unknown (s_first_error_ino)
| 416 | 8 | | Unknown (s_first_error_block)
| 424 | 32 | | Unknown (s_first_error_func)
| 456 | 4 | | Unknown (s_first_error_line)
| 460 | 4 | | Unknown (s_last_error_time)
| 464 | 4 | | Unknown (s_last_error_ino)
| 468 | 4 | | Unknown (s_last_error_line)
| 472 | 8 | | Unknown (s_last_error_block)
| 480 | 32 | | Unknown (s_last_error_func)
| 512 | 64 | | Unknown (s_mount_opts)
| 576 | 4 | | Unknown (s_usr_quota_inum)
| 580 | 4 | | Unknown (s_grp_quota_inum)
| 584 | 4 | | Unknown (s_overhead_clusters)
| 588 | 2 x 4 | | Unknown (s_backup_bgs)
| 596 | 4 | | Unknown (s_encrypt_algos)
| 600 | 16 | | Unknown (s_encrypt_pw_salt)
| 616 | 4 | | Unknown (s_lpf_ino)
| 620 | 4 | | Unknown (s_prj_quota_inum)
| 624 | 4 | | Unknown (s_checksum_seed)
| 628 | 1 | | Unknown (s_wtime_hi)
| 629 | 1 | | Unknown (s_mtime_hi)
| 630 | 1 | | Unknown (s_mkfs_time_hi)
| 631 | 1 | | Unknown (s_lastcheck_hi)
| 632 | 1 | | Unknown (s_first_error_time_hi)
| 633 | 1 | | Unknown (s_last_error_time_hi)
| 634 | 1 | | Unknown (s_first_error_errcode)
| 635 | 1 | | Unknown (s_last_error_errcode)
| 636 | 2 | | Unknown (s_encoding)
| 638 | 2 | | Unknown (s_encoding_flags)
| 640 | 4 | | Unknown (s_orphan_file_inum)
| 644 | 94 x 4 = 376 | | Unknown (reserved)
| 1020 | 4 | | Checksum

If checksum type is CRC-32C, the checksum is stored as 0xffffffff - CRC-32C.

> Note that some versions of mkfs.ext set the file system creation time even for
> ext2 and when EXT3_FEATURE_COMPAT_HAS_JOURNAL is not set.

TODO: Is the only way to determine the file system version the compatibility and equivalent flags?

### Checksum calculation

If checksum type is CRC-32C, the CRC32-C algorithm with the Castagnoli
polynomial (0x1edc6f41) and initial value of 0 is used to calculate the
checksum.

The checksum is calculated over the 1020 bytes of data of the suberblock.

### <a name="file_system_state_flags"></a>File system state flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | | Is clean
| 0x0002 | | Has errors
| 0x0004 | | Recovering orphan inodes

### <a name="error_handling_status"></a>Error-handling status

| Value | Identifier | Description
| --- | --- | ---
| 1 | | Continue
| 2 | | Remount as read-only
| 3 | | Panic

### <a name="creator_operating_system"></a>Creator operating system

| Value | Identifier | Description
| --- | --- | ---
| 0 | | Linux
| 1 | | GNU Hurd
| 2 | | Masix
| 3 | | FreeBSD
| 4 | | Lites

### <a name="format_revisision"></a>Format revision

| Value | Identifier | Description
| --- | --- | ---
| 0 | EXT2_GOOD_OLD_REV | Original version with a fixed inode size of 128 bytes
| 1 | EXT2_DYNAMIC_REV | Version with dynamic inode size support

### <a name="compatible_features_flags"></a>Compatible features flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | EXT2_COMPAT_PREALLOC | Pre-allocate directory blocks, which is intended to reduce fragmentation
| 0x00000002 | EXT2_FEATURE_COMPAT_IMAGIC_INODES | Has AFS server inodes.
| 0x00000004 | EXT3_FEATURE_COMPAT_HAS_JOURNAL | Has a journal.
| 0x00000008 | EXT2_FEATURE_COMPAT_EXT_ATTR | Has extended inode attributes.
| 0x00000010 | EXT2_FEATURE_COMPAT_RESIZE_INO, EXT2_FEATURE_COMPAT_RESIZE_INODE | Has reserved GDT blocks for file system expansion, which requires RO_COMPAT_SPARSE_SUPER
| 0x00000020 | EXT2_FEATURE_COMPAT_DIR_INDEX | Has hash-indexed directories.
| 0x00000040 | COMPAT_LAZY_BG | Unknown (Lazy block group)
| 0x00000080 | COMPAT_EXCLUDE_INODE | Unknown (Exclude inode), which is not yet implemented and intended for a future file system snapshot feature
| 0x00000100 | COMPAT_EXCLUDE_BITMAP | Unknown (Exclude bitmap), which is not yet implemented and intended for a future file system snapshot feature
| 0x00000200 | EXT4_FEATURE_COMPAT_SPARSE_SUPER2 | Has a version 2 sparse superblock.
| 0x00000400 | EXT4_FEATURE_COMPAT_FAST_COMMIT | Unknown (fast commit)
| 0x00000800 | EXT4_FEATURE_COMPAT_STABLE_INODES | Unknown (stable inodes)
| 0x00001000 | EXT4_FEATURE_COMPAT_ORPHAN_FILE | Has orphan file.

> Note that EXT2_FEATURE_COMPAT_, EXT3_FEATURE_COMPAT_, EXT4_FEATURE_COMPAT_ and
> COMPAT_ can be used interchangeably.

### <a name="incompatible_features_flags"></a>Incompatible features flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | EXT2_FEATURE_INCOMPAT_COMPRESSION | Has compression, which is not yet implemented
| 0x00000002 | EXT2_FEATURE_INCOMPAT_FILETYPE | Has directory type
| 0x00000004 | EXT3_FEATURE_INCOMPAT_RECOVER | Needs recovery
| 0x00000008 | EXT3_FEATURE_INCOMPAT_JOURNAL_DEV | Has journal device
| 0x00000010 | EXT2_FEATURE_INCOMPAT_META_BG | Has meta (or metadata) block groups
| | |
| 0x00000040 | EXT4_FEATURE_INCOMPAT_EXTENTS | Has extents
| 0x00000080 | EXT4_FEATURE_INCOMPAT_64BIT | Has 64-bit support, which supports more than 2^32 blocks
| 0x00000100 | EXT4_FEATURE_INCOMPAT_MMP | Multiple mount protection
| 0x00000200 | EXT4_FEATURE_INCOMPAT_FLEX_BG | Has flex (or flexible) block groups
| 0x00000400 | EXT4_FEATURE_INCOMPAT_EA_INODE | Inodes can be used to store large extended attribute values
| | |
| 0x00001000 | EXT4_FEATURE_INCOMPAT_DIRDATA | Data in directory entry, which is not yet implemented
| 0x00002000 | EXT4_FEATURE_INCOMPAT_CSUM_SEED, EXT4_FEATURE_INCOMPAT_BG_USE_META_CSUM | Initial metadata checksum value (or seed) is stored in the superblock
| 0x00004000 | EXT4_FEATURE_INCOMPAT_LARGEDIR | Large directory >2GB or 3-level hash tree (HTree).
| 0x00008000 | EXT4_FEATURE_INCOMPAT_INLINE_DATA | Has data stored in inode.
| 0x00010000 | EXT4_FEATURE_INCOMPAT_ENCRYPT | Has encrypted inodes.
| 0x00020000 | EXT4_FEATURE_INCOMPAT_CASEFOLD | Hash case folding

> Note that EXT2_FEATURE_INCOMPAT_, EXT3_FEATURE_INCOMPAT_,
> EXT4_FEATURE_INCOMPAT_ and INCOMPAT_ can be used interchangeably.

### <a name="read_only_compatible_features_flags"></a>Read-only compatible features flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER | Has sparse superblocks and group descriptor tables. If set a superblock is stored in block groups 0, 1 and those that are powers of 3, 5 and 7. If not set a superblock is stored in every block group.
| 0x00000002 | EXT2_FEATURE_RO_COMPAT_LARGE_FILE | Contains large files.
| 0x00000004 | EXT2_FEATURE_RO_COMPAT_BTREE_DIR | Intended for hash-tree directory (or directory B-tree), which is not yet implemented
| 0x00000008 | EXT4_FEATURE_RO_COMPAT_HUGE_FILE | Has huge file support.
| 0x00000010 | EXT4_FEATURE_RO_COMPAT_GDT_CSUM | Has group descriptors with checksums.
| 0x00000020 | EXT4_FEATURE_RO_COMPAT_DIR_NLINK | The ext3 32000 subdirectory limit does not apply. A directory's number of links will be set to 1 if it is incremented past 64999.
| 0x00000040 | EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE | Has large inodes. The size of an inode can be larger than 128 bytes.
| 0x00000080 | EXT4_FEATURE_RO_COMPAT_HAS_SNAPSHOT | Has snapshots, which is not yet implemented and intended for a future file system snapshot feature
| 0x00000100 | EXT4_FEATURE_RO_COMPAT_QUOTA | Quota is handled transactionally with the journal.
| 0x00000200 | EXT4_FEATURE_RO_COMPAT_BIGALLOC | Has big block allocation bitmaps. Block allocation bitmaps are tracked in units of clusters (of blocks) instead of blocks.
| 0x00000400 | EXT4_FEATURE_RO_COMPAT_METADATA_CSUM | File system metadata has checksums.
| 0x00000800 | EXT4_FEATURE_RO_COMPAT_REPLICA | Supports replicas.
| 0x00001000 | EXT4_FEATURE_RO_COMPAT_READONLY | Read-only file system image.
| 0x00002000 | EXT4_FEATURE_RO_COMPAT_PROJECT | File system tracks project quotas.
| 0x00004000 | EXT4_FEATURE_RO_COMPAT_SHARED_BLOCKS | File system has (read-only) shared blocks.
| 0x00008000 | EXT4_FEATURE_RO_COMPAT_VERITY | Unknown (Verity inodes may be present on the filesystem)
| 0x00010000 | EXT4_FEATURE_RO_COMPAT_ORPHAN_PRESENT | Orphan file may be non-empty.

> Note that EXT2_FEATURE_RO_COMPAT_, EXT3_FEATURE_RO_COMPAT_,
> EXT4_FEATURE_RO_COMPAT_ and RO_COMPAT_ can be used interchangeably.

> Note that in some ext file systems used by ChromeOS it has been observed that
> the upper 8-bits of the read-only compatible features flags are set as in
> 0xff000003. debugfs identifies these as FEATURE_R24 - FEATURE_R31.

### <a name="checksum_types"></a>Checksum types

| Value | Identifier | Description
| --- | --- | ---
| 1 | EXT4_CRC32C_CHKSUM | CRC-32C (or CRC32-C), which uses the Castagnoli polynomial (0x1edc6f41)

## The group descriptor table

The group descriptor table is stored in the block following the superblock.

The group descriptor table consist of:

* one or more group descriptors

### The ext2 and ext3 group descriptor

The ext2 and ext3 group descriptor is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Block bitmap block number. The block number is relative from the start of the volume
| 4 | 4 | | Inode bitmap block number. The block number is relative from the start of the volume
| 8 | 4 | | Inode table block number. The block number is relative from the start of the volume
| 12 | 2 | | Number of unallocated blocks
| 14 | 2 | | Number of unallocated inodes
| 16 | 2 | | Number of directories
| 18 | 2 | | Unknown (padding)
| 20 | 3 x 4 | | Unknown (reserved)

> Note that it has been observed that implementations that support ext4 can set
> a value in the padding. It is currently assumed that this value contains
> [block group flags](#block_group_flags).

### The ext4 group descriptor

The ext4 group descriptor is 68 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Block bitmap block number, which contains the lower 32-bit of the value. The block number is relative from the start of the volume
| 4 | 4 | | Inode bitmap block number, which contains the lower 32-bit of the value. The block number is relative from the start of the volume
| 8 | 4 | | Inode table block number, which contains the lower 32-bit of the value. The block number is relative from the start of the volume
| 12 | 2 | | Number of unallocated blocks, which contains the lower 16-bit of the value
| 14 | 2 | | Number of unallocated inodes, which contains the lower 16-bit of the value
| 16 | 2 | | Number of directories, which contains the lower 16-bit of the value
| 18 | 2 | | [Block group flags](#block_group_flags)
| 20 | 4 | | Exclude bitmap block number, which contains the lower 32-bit of the value. The block number is relative from the start of the volume
| 24 | 2 | | Block bitmap checksum, which contains the lower 16-bit of the value
| 26 | 2 | | Inode bitmap checksum, which contains the lower 16-bit of the value
| 28 | 2 | | Number of unused inodes, which contains the lower 16-bit of the value
| 30 | 2 | | Checksum
| <td colspan="4"> *If 64-bit support (EXT4_FEATURE_INCOMPAT_64BIT) is enabled and group descriptor size > 32*
| 32 | 4 | | Block bitmap block number, which contains the upper 32-bit of the value. The block number is relative from the start of the volume
| 36 | 4 | | Inode bitmap block number, which contains the upper 32-bit of the value. The block number is relative from the start of the volume
| 40 | 4 | | Inode table block number, which contains the upper 32-bit of the value. The block number is relative from the start of the volume
| 44 | 2 | | Number of unallocated blocks, which contains the upper 16-bit of the value
| 46 | 2 | | Number of unallocated inodes, which contains the upper 16-bit of the value
| 48 | 2 | | Number of directories, which contains the upper 16-bit of the value
| 50 | 2 | | Number of unused inodes, which contains the upper 16-bit of the value
| 52 | 4 | | Exclude bitmap block number, which contains the upper 32-bit of the value. The block number is relative from the start of the volume
| 56 | 2 | | Block bitmap checksum, which contains the upper 16-bit of the value
| 60 | 2 | | Inode bitmap checksum, which contains the upper 16-bit of the value
| 64 | 4 | | Unknown (padding)

If checksum type is CRC-32C, the checksum is stored as the lower 16-bits of
0xffffffff - CRC-32C, otherwise the checksum is stored as a CRC-16.

### Checksum calculation

If checksum type is CRC-32C, the CRC32-C algorithm with the Castagnoli
polynomial (0x1edc6f41) and initial value of 0 is used to calculate the
checksum.

The checksum is calculated over:

* the 16 byte file system identifier in the superblock
* the group number as a 32-bit little-endian integer
* the data of the group descriptor with the checksum set to 0-byte values

TODO: describe the block bitmap checksum calculation: crc32c(s_uuid+grp_num+bbitmap)

TODO: describe the inode bitmap checksum calculation: crc32c(s_uuid+grp_num+ibitmap)

### <a name="block_group_flags"></a>Block group flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | EXT4_BG_INODE_UNINIT | The inode table and bitmap are not initialized
| 0x0002 | EXT4_BG_BLOCK_UNINIT | The block bitmap is not initialized
| 0x0004 | EXT4_BG_INODE_ZEROED | The inode table is filled with 0

## Direct and indirect blocks

Direct blocks are blocks that part of the data stream of a file entry.

A direct block number is 0 that is part of the data stream represents a sparse
data block.

Indirect blocks are blocks that refer to blocks containing direct or indirect
block numbers. There are multiple levels of indirect block:

* indirect blocks (level 1), that refer to direct blocks
* double indirect blocks (level 2), that refer to indirect blocks
* triple indirect blocks (level 3), that refer to double indirect blocks

An indirect block number is 0 that is part of the data stream represents sparse
data blocks.

## Extents

Extents were introduced in ext4 and are controlled by
EXT4_FEATURE_INCOMPAT_EXTENTS.

Extents form an extent B-Tree, where:

* [extent indexes](#ext4_extent_index) are stored in the branch nodes and
* [extent descriptors](#ext4_extent_descriptor) are stored in the leaf nodes.

An extents B-tree node consists of:

* extents header
* extents entries
* extents footer

> Note that inodes can have an implicit last sparse extent if the the inode
> data size is greater than the total data size defined by the extent
> descriptors.

### <a name="ext4_extents_header"></a>The ext4 extents header

The ext4 extents header (ext4_extent_header) is 12 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | "\x0a\xf3" | Signature
| 2 | 2 | | Number of entries
| 4 | 2 | | Maximum number of entries
| 6 | 2 | | Depth, where 0 reprensents a leaf node and 1 to 5 different levels of branch nodes.
| 8 | 4 | | Generation, which is used by Lustre, but not by standard ext4.

### <a name="ext4_extent_descriptor"></a>The ext4 extent descriptor

The ext4 extent descriptor (ext4_extent) is 12 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Logical block number
| 4 | 2 | | Number of blocks
| 6 | 2 | | Upper 16-bits of physical block number
| 8 | 4 | | Lower 32-bits of physical block number

If number of blocks > 32768 the extent is considered "uninitialized" which is
(as far as currently known) comparable to extent being sparse. The number of
blocks of the sparse extent can be determined as following:

```
sparse_number_of_blocks = number_of_blocks - 32768
```

> Note that sparse extents can exist between the extent descriptors. In such a
> case the logical block number will not align with the information from the
> previous extent descriptors.

> Note that the native Linux ext implementation expects the extents to be stored
> in order of logical block number.

### <a name="ext4_extent_index"></a>The ext4 extents index

The ext4 extent index (ext4_extent_idx) is 12 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Logical block number, which contains the first logical block number of next depth extents block
| 4 | 4 | | Lower 32-bits of physical block number, which contains the block number of the next depth extents block
| 8 | 2 | | Upper 16-bits of physical block number, which contains the block number of the next depth extents block
| 10 | 2 | | Unknown (unused)

### The ext4 extents footer

The ext4 extents footer (ext4_extent_tail) is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum of an extents block, which contains a CRC32

## The inode

> Note that the size of the inode is defined in the superblock when dynamic
> inode information is present.

> Note that an ext4 inode can be used on ext2 formatted file system. Seen in
> combination with format revision 1 and inode size > 128 created by mkfs.ext2.

### The ext2 inode

The ext2 inode is 128 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | [File mode](#file_mode), which contains file type and permissions
| 2 | 2 | | Lower 16-bits of owner (or user) identifier (UID)
| 4 | 4 | | Data size
| 8 | 4 | | (last) access time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 12 | 4 | | (last) inode change time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 16 | 4 | | (last) modification time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 20 | 4 | | Deletion time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 24 | 2 | | Lower 16-bits of group identifier (GID)
| 26 | 2 | | Number of (hard) links
| 28 | 4 | | Numer of blocks
| 32 | 4 | | [Flags](#inode_flags)
| 36 | 4 | | Unknown (reserved)
| 40 | 12 x 4 | | Array of direct block numbers. A block number is relative from the start of the volume
| 88 | 4 | | Indirect block number. A block number is relative from the start of the volume
| 92 | 4 | | Double indirect block number. A block number is relative from the start of the volume
| 96 | 4 | | Triple indirect block number. A block number is relative from the start of the volume
| 100 | 4 | | NFS generation number
| 104 | 4 | | File ACL (or extended attributes) block number
| 108 | 4 | | Unknown (Directory ACL)
| 112 | 4 | | Fragment block address
| 116 | 1 | | Fragment block index
| 117 | 1 | | Fragment size
| 118 | 2 | | Unknown (padding)
| 120 | 2 | | Upper 16-bits of owner (or user) identifier (UID)
| 122 | 2 | | Upper 16-bits of group identifier (GID)
| 124 | 4 | | Unknown (reserved)

> Note that for a character and block device the first 2 bytes of the array of
> direct block numbers contain the minor and major device number respectively.

### The ext3 inode

The ext3 inode is 132 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | [File mode](#file_mode), which contains file type and permissions
| 2 | 2 | | Lower 16-bits of owner (or user) identifier (UID)
| 4 | 4 | | Data size
| 8 | 4 | | (last) access time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 12 | 4 | | (last) inode change time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 16 | 4 | | (last) modification time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 20 | 4 | | Deletion time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 24 | 2 | | Lower 16-bits of group identifier (GID)
| 26 | 2 | | Number of (hard) links
| 28 | 4 | | Numer of blocks
| 32 | 4 | | [Flags](#inode_flags)
| 36 | 4 | | Unknown (reserved)
| 40 | 12 x 4 | | Array of direct block numbers. A block number is relative from the start of the volume
| 88 | 4 | | Indirect block number. A block number is relative from the start of the volume
| 92 | 4 | | Double indirect block number. A block number is relative from the start of the volume
| 96 | 4 | | Triple indirect block number. A block number is relative from the start of the volume
| 100 | 4 | | NFS generation number
| 104 | 4 | | File ACL (or extended attributes) block number
| 108 | 4 | | Unknown (Directory ACL)
| 112 | 4 | | Fragment block address
| 116 | 1 | | Fragment block index
| 117 | 1 | | Fragment size
| 118 | 2 | | Unknown (padding)
| 120 | 2 | | Upper 16-bits of owner (or user) identifier (UID)
| 122 | 2 | | Upper 16-bits of group identifier (GID)
| 124 | 4 | | Unknown (reserved)
| <td colspan="4"> *If inode size > 128*
| 128 | 2 | | Extended inode size
| 130 | 2 | | Unknown (padding)

> Note that for a character and block device the first 2 bytes of the array of
> direct block numbers contain the minor and major device number respectively.

### The ext4 inode

The ext4 inode is 160 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | [File mode](#file_mode), which contains file type and permissions
| 2 | 2 | | Lower 16-bits of owner (or user) identifier (UID)
| 4 | 4 | | Lower 32-bits of data size
| <td colspan="4"> *If EXT4_EA_INODE_FL is not set*
| 8 | 4 | | (last) access time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 12 | 4 | | (last) inode change time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 16 | 4 | | (last) modification time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| <td colspan="4"> *If EXT4_EA_INODE_FL is set*
| 8 | 4 | | Unknown (extended attribute value data checksum)
| 12 | 4 | | Unknown (lower 32-bits of extended attribute reference count)
| 16 | 4 | | Unknown (inode number that owns the extended attribute)
| <td colspan="4"> *Common*
| 20 | 4 | | Deletion time, which contains the number of seconds since January 1, 1970 00:00:00 UTC (POSIX epoch)
| 24 | 2 | | Lower 16-bits of group identifier (GID)
| 26 | 2 | | Number of (hard) links
| 28 | 4 | | Lower 16-bits of number of blocks
| 32 | 4 | | [Flags](#inode_flags)
| <td colspan="4"> *If EXT4_EA_INODE_FL is not set*
| 36 | 4 | | Unknown (lower 32-bits of version)
| <td colspan="4"> *If EXT4_EA_INODE_FL is set*
| 36 | 4 | | Unknown (upper 32-bits of extended attribute reference count)
| <td colspan="4"> *If EXT4_EXTENTS_FL and EXT4_INLINE_DATA_FL are not set*
| 40 | 12 x 4 | | Array of direct block numbers. A block number is relative from the start of the volume
| 88 | 4 | | Indirect block number. A block number is relative from the start of the volume
| 92 | 4 | | Double indirect block number. A block number is relative from the start of the volume
| 96 | 4 | | Triple indirect block number. A block number is relative from the start of the volume
| <td colspan="4"> *If EXT4_EXTENTS_FL is set*
| 40 | 12 | | [Extents header](#xt4_extents_header)
| 52 | 4 x 12 | | [extent descriptors](#ext4_extent_descriptor) or [extents indexes](#ext4_extent_index)
| <td colspan="4"> *If EXT4_INLINE_DATA_FL is set*
| 40 | 60 | | File content data
| <td colspan="4"> *Common*
| 100 | 4 | | NFS generation number
| 104 | 4 | | Lower 32-bits of file ACL (or extended attributes) block number
| 108 | 4 | | Upper 32-bits of data size
| 112 | 4 | | Fragment block address
| 116 | 2 | | Upper 16-bits of number of blocks
| 118 | 2 | | Upper 16-bits of file ACL (or extended attributes) block number
| 120 | 2 | | Upper 16-bits of owner (or user) identifier (UID)
| 122 | 2 | | Upper 16-bits of group identifier (GID)
| 124 | 2 | | Lower 16-bits of checksum
| 126 | 2 | | Unknown (reserved)
| <td colspan="4"> *If inode size > 128*
| 128 | 2 | | Extended inode size
| 130 | 2 | | Upper 16-bits of checksum
| 132 | 4 | | (last) inode change time extra precision
| 136 | 4 | | (last) modification time extra precision
| 140 | 4 | | (last) access time extra precision
| 144 | 4 | | Creation time
| 148 | 4 | | Creation time extra precision
| 152 | 4 | | Unknown (upper 32-bits of version)
| 156 | 4 | | Unknown (i_projid)

If checksum type is CRC-32C, the checksum is stored as 0xffffffff - CRC-32C.

> Note that for a character and block device the first 2 bytes of the array of
> direct block numbers contain the minor and major device number respectively.

#### Checksum calculation

If checksum type is CRC-32C, the CRC32-C algorithm with the Castagnoli
polynomial (0x1edc6f41) and initial value of 0 is used to calculate the
checksum.

The checksum is calculated from:

* the 16 byte file system identifier in the superblock
* the inode number as a 32-bit little-endian integer
* the NFS generation number in the inode as a 32-bit little-endian integer
* the data of the inode with the lower and upper part of the checksum set to 0-byte values.

#### Extra precision

The ext4 extra precision is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 2 bits | | Extra epoch value
| 0.2 | 30 bits | | Fraction of second in nanoseconds

The 34 bits extra precision timestamp (in number of seconds) can be calculated as
following:

```
extra_precision_timestamp = ( extra_epoch_value x 0x100000000 ) + timestamp
```

#### Notes

It has been observed that when EXT4_EA_INODE_FL is set the (last) modification
time can contain a valid timestamp.

According to [The Linux Kernel documentation](https://docs.kernel.org/filesystems/ext4/overview.html#large-extended-attribute-values)

> For backward compatibility with older versions of this feature, the
> i_mtime/i_generation may store a back-reference to the inode number and
> i_generation of the one owning inode (in cases where the EA inode is not
> referenced by multiple inodes) to verify that the EA inode is the correct
> one being accessed.

### <a name="file_mode"></a>File mode

| Value | Identifier | Description
| --- | --- | ---
| <td colspan="3"> *Access other, Bitmask: 0x0007 (S_IRWXO)*
| 0x0001 | S_IXOTH | X-access for other
| 0x0002 | S_IWOTH | W-access for other
| 0x0004 | S_IROTH | R-access for other
| <td colspan="3"> *Access group, Bitmask: 0x0038 (S_IRWXG)*
| 0x0008 | S_IXGRP | X-access for group
| 0x0010 | S_IWGRP | W-access for group
| 0x0020 | S_IRGRP | R-access for group
| <td colspan="3"> *Access owner (or user), Bitmask: 0x01c0 (S_IRWXU)*
| 0x0040 | S_IXUSR | X-access for owner (or user)
| 0x0080 | S_IWUSR | W-access for owner (or user)
| 0x0100 | S_IRUSR | R-access for owner (or user)
| <td colspan="3"> *Other*
| 0x0200 | S_ISTXT | Sticky bit
| 0x0400 | S_ISGID | Set group identifer (GID) on execution
| 0x0800 | S_ISUID | Set owner (or user) identifer (UID) on execution
| <td colspan="3"> *Type of file, Bitmask: 0xf000 (S_IFMT)*
| 0x1000 | S_IFIFO | Named pipe (FIFO)
| 0x2000 | S_IFCHR | Character device
| 0x4000 | S_IFDIR | Directory
| 0x6000 | S_IFBLK | Block device
| 0x8000 | S_IFREG | Regular file
| 0xa000 | S_IFLNK | Symbolic link
| 0xc000 | S_IFSOCK | Socket

### <a name="inode_flags"></a>Inode flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | EXT2_SECRM_FL, EXT3_SECRM_FL, EXT4_SECRM_FL, EXT4_INODE_SECRM | Secure deletion
| 0x00000002 | EXT2_UNRM_FL, EXT3_UNRM_FL, EXT4_UNRM_FL, EXT4_INODE_UNRM | Undelete
| 0x00000004 | EXT2_COMPR_FL, EXT3_COMPR_FL, EXT4_COMPR_FL, EXT4_INODE_COMPR | Compressed file, which is not yet implemented
| 0x00000008 | EXT2_SYNC_FL, EXT3_SYNC_FL, EXT4_SYNC_FL, EXT4_INODE_SYNC | Synchronous updates
| 0x00000010 | EXT2_IMMUTABLE_FL, EXT3_IMMUTABLE_FL, EXT4_IMMUTABLE_FL, EXT4_INODE_IMMUTABLE | Immutable file
| 0x00000020 | EXT2_APPEND_FL, EXT3_APPEND_FL, EXT4_APPEND_FL, EXT4_INODE_APPEND | Writes to file may only append
| 0x00000040 | EXT2_NODUMP_FL, EXT3_NODUMP_FL, EXT4_NODUMP_FL, EXT4_INODE_NODUMP | Do not remove (or dump) file
| 0x00000080 | EXT2_NOATIME_FL, EXT3_NOATIME_FL, EXT4_NOATIME_FL, EXT4_INODE_NOATIME | Do not update access time (atime)
| 0x00000100 | EXT2_DIRTY_FL, EXT3_DIRTY_FL, EXT4_DIRTY_FL, EXT4_INODE_DIRTY | Dirty compressed file, which is not yet implemented
| 0x00000200 | EXT2_COMPRBLK_FL, EXT3_COMPRBLK_FL, EXT4_COMPRBLK_FL, EXT4_INODE_COMPRBLK | One or more compressed clusters, which is not yet implemented
| 0x00000400 | EXT2_NOCOMP_FL, EXT3_NOCOMP_FL, EXT4_NOCOMPR_FL, EXT4_INODE_NOCOMPR | Do not compress, which is not yet implemented
| <td colspan="3"> *ext2 and ext3*
| 0x00000800 | EXT2_ECOMPR_FL, EXT3_ECOMPR_FL | Encrypted Compression error
| <td colspan="3"> *ext4*
| 0x00000800 | EXT4_ENCRYPT_FL, EXT4_INODE_ENCRYPT | Encrypted file
| <td colspan="3"> *Common*
| 0x00001000 | EXT2_BTREE_FL, EXT2_INDEX_FL, EXT3_INDEX_FL, EXT4_INDEX_FL, EXT4_INODE_INDEX | Hash-indexed directory (previously referred to as B-tree format).
| 0x00002000 | EXT2_IMAGIC_FL, EXT3_IMAGIC_FL, EXT4_IMAGIC_FL, EXT4_INODE_IMAGIC | AFS directory
| 0x00004000 | EXT2_JOURNAL_DATA_FL, EXT3_JOURNAL_DATA_FL, EXT4_JOURNAL_DATA_FL, EXT4_INODE_JOURNAL_DATA | File data must be written using the journal.
| 0x00008000 | EXT2_NOTAIL_FL, EXT3_NOTAIL_FL, EXT4_NOTAIL_FL, EXT4_INODE_NOTAIL | File tail should not be merged, which is not used by ext4
| 0x00010000 | EXT2_DIRSYNC_FL, EXT3_DIRSYNC_FL, EXT4_DIRSYNC_FL, EXT4_INODE_DIRSYNC | Directory entries should be written synchronously (dirsync)
| 0x00020000 | EXT2_TOPDIR_FL, EXT3_TOPDIR_FL, EXT4_TOPDIR_FL, EXT4_INODE_TOPDIR | Top of directory hierarchy
| <td colspan="3"> *ext4*
| 0x00040000 | EXT4_HUGE_FILE_FL, EXT4_INODE_HUGE_FILE | Is a [huge file](#huge_files)
| 0x00080000 | EXT4_EXTENTS_FL, EXT4_INODE_EXTENTS | Inode uses extents
| 0x00100000 | EXT4_INODE_VERITY | Verity protected inode
| 0x00200000 | EXT4_EA_INODE_FL, EXT4_INODE_EA_INODE | Inode used for large extended attribute
| 0x00400000 | EXT4_EOFBLOCKS_FL, EXT4_INODE_EOFBLOCKS | Blocks allocated beyond EOF
| | |
| 0x01000000 | EXT4_SNAPFILE_FL | Inode is a snapshot
| 0x02000000 | EXT4_INODE_DAX | Inode is direct-access (DAX)
| 0x04000000 | EXT4_SNAPFILE_DELETED_FL | Snapshot is being deleted
| 0x08000000 | EXT4_SNAPFILE_SHRUNK_FL | Snapshot shrink has completed
| 0x10000000 | EXT4_INLINE_DATA_FL, EXT4_INODE_INLINE_DATA | Inode has inline data
| 0x20000000 | EXT4_PROJINHERIT_FL, EXT4_INODE_PROJINHERIT | Create sub file entries with the same project identifier
| 0x40000000 | EXT4_INODE_CASEFOLD | Casefolded directory
| 0x80000000 | EXT4_INODE_RESERVED | Unknown (reserved)

### Reserved inode numbers

| Value | Identifier | Description
| --- | --- | ---
| 1 | EXT2_BAD_INO, EXT3_BAD_INO, EXT4_BAD_INO | Bad blocks inode
| 2 | EXT2_ROOT_INO, EXT3_ROOT_INO, EXT4_ROOT_INO | Root inode
| 3 | EXT4_USR_QUOTA_INO | Owner (or user) quota inode
| 4 | EXT4_GRP_QUOTA_INO | Group quota inode
| 5 | EXT2_BOOT_LOADER_INO, EXT3_BOOT_LOADER_INO, EXT4_BOOT_LOADER_INO | Boot loader inode
| 6 | EXT2_UNDEL_DIR_INO, EXT3_UNDEL_DIR_INO, EXT4_UNDEL_DIR_INO | Undelete directory inode
| 7 | EXT3_RESIZE_INO, EXT4_RESIZE_INO | Reserved group descriptors inode
| 8 | EXT3_JOURNAL_INO, EXT4_JOURNAL_INO | Journal inode

## Inline data

ext4 supports storing file entry data inline when the inode flag
EXT4_INLINE_DATA_FL is set.

> Note that inodes can have an implicit last sparse extent if the the inode
> data size is greater than 60 bytes.

## <a name="huge_files"></a>Huge files

TODO: complete section

## Directory entries

Directories entries are stored in the data blocks of a directory inode. The
directory entries can be stored in multiple ways:

* as linear directory entries
* as inline data directory entries
* as hash-tree directory entries

### Linear directory entries

Linear directories entries are stored in a series of allocation blocks.

Linear directory entries contain:

* directory entry for "." (self)
* directory entry for ".." (parent)
* directory entry for other file system entries

#### <a name="directory_entry"></a>The directory entry

The directory entry is of variable size, at most 263 bytes, and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Inode number
| 4 | 2 | | Directory entry size, which must be a multitude of 4.
| 6 | 1 | | Name size, which contains the size of the name without the end-of-string character and has a maximum of 255
| 7 | 1 | | [File type](#file_types)
| 8 | ... | | Name, which contains a narrow character string without end-of-string character

Older directory entry structures considered the name size a 16-bit value, but
the upper byte was never used.

The name can contain any character value except the path segment separator ('/')
and the NUL-character ('\0').

#### <a name="file_types"></a>File types

| Value | Identifier | Description
| --- | --- | ---
| 0 | EXT2_FT_UNKNOWN | Unknown
| 1 | EXT2_FT_REG_FILE | Regular file
| 2 | EXT2_FT_DIR | Directory
| 3 | EXT2_FT_CHRDEV | Character device
| 4 | EXT2_FT_BLKDEV | Block device
| 5 | EXT2_FT_FIFO | FIFO queue
| 6 | EXT2_FT_SOCK | Socket
| 7 | EXT2_FT_SYMLINK | Symbolic link

### Inline data directory entries

ext4 supports storing the directory entries as inline data when the inode flag
EXT4_INLINE_DATA_FL is set.

The inline data directory entries is of variable size, at most 60 bytes, and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Parent inode number
| 4 | ... | | Array of [directory entries](#directory_entry)

### Hash tree directory entries

The data of the hash tree (HTree) is stored in the data blocs or extent defined
by the directory inode. The hash-indexed directory entries are read-compatible
with the [linear directory entry](#directory_entry).

#### Hash tree root

The hash tree root consists of:

* dx_root
  * directory entry for "." (self)
  * directory entry for ".." (parent)
  * dx_root_info
  * Array of dx_entry
* directory entry for other file system entries

#### dx_root_info

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | 0 | Unknown (reserved)
| 4 | 1 | | Hash method (or version)
| 5 | 1 | 8 | Root information size
| 6 | 1 | | Number of indirect levels in the hash tree
| 7 | 1 | | Unknown (unused flags)

#### dx_entry

TODO: complete section

```
struct dx_entry
{
        __le32 hash;
        __le32 block;
};
```

## Symbolic links

If the target path of a symbolic link is less than 60 characters long, it is
stored in the 60 bytes in the inode that are normally used for the 12 direct
and 3 indirect block numbers. If the target path is longer than 60 characters,
a block is allocated, and the block contains the target path. The inode data
size contains the length of the target path.

## <a name="extended_attributes"></a>Extended attributes

Extended attributes can be stored:

* in the inode block after the inode data
* in the block referenced by the file ACL (or extended attributes) block number, if not 0

> Note that both should be read to get the all the extended attributes.

Extended attributes consists of:

* An extended attributes header
* Extended attributes entries with a terminator

### The extended attributes inode header

The extended attributes inode header (ext2_xattr_ibody_header,
ext3_xattr_ibody_header, ext4_xattr_ibody_header) is 4 bytes in size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "\x00\x00\x02\xea" | Signature

### The extended attributes block header

#### The ext2 and ext3 extended attributes block header

The ext2 and ext3 extended attributes block header (ext2_xattr_header,
ext3_xattr_header) is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "\x00\x00\x02\xea" | Signature
| 4 | 4 | | Unknown (reference count)
| 8 | 4 | | Number of blocks
| 12 | 4 | | Attributes hash
| 16 | 4 x 4 | | Unknown (reserved)

#### The ext4 extended attributes block header

The ext4 extended attributes block header (ext4_xattr_header) is 32 bytes of
size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "\x00\x00\x02\xea" | Signature
| 4 | 4 | | Unknown (Reference count)
| 8 | 4 | | Number of blocks
| 12 | 4 | | Attributes hash
| 16 | 4 | | Checksum
| 20 | 3 x 4 | | Unknown (reserved)

### The extended attributes entry

The extended attributes entry (ext2_xattr_entry, ext3_xattr_entry,
ext4_xattr_entry) is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | | Name size, which contains the size of the name without the end-of-string character
| 1 | 1 | | Name index
| 2 | 2 | | Value data offset, which contains the offset of the value data relative from the start of the extended attributes block or after the extended attributes signature in the inode block data
| 4 | 4 | | Value data inode number, which contains the inode number that contains the value data or 0 to indicate the current block
| 8 | 4 | | Value data size
| 12 | 4 | | Unknown (Attribute hash)
| 16 | ... | | Name string, which contains an ASCII string without end-of-string character
| ... | ... | | 32-bit alignment padding

The last extended attributes entry has the first 4 values set to 0 (8 bytes)
and is used as a terminator.

> Note that the name can be empty, for example in combination with a prefix or
> with an encrypted file.

### The extended attribute name index

The name index indicates the prefix of the extended attribute name.

| Name index | Name prefix | Description
| --- | --- | ---
| 0 | "" | No prefix
| 1 | "user." |
| 2 | "system.posix_acl_access" |
| 3 | "system.posix_acl_default" |
| 4 | "trusted." |
| | |
| 6 | "security." |
| 7 | "system." |
| 8 | "system.richacl" |

## Journal

The journal was introduced in ext3.

TODO: complete section

## Exclude bitmap

TODO: complete section

> Note that the excluded bitmap is used for snapshots.

## References

* [ext4 Data Structures and Algorithms](https://docs.kernel.org/filesystems/ext4), by the Linux kernel documentation
