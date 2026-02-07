# Hierarchical File System (HFS) format

The Hierarchical File System (HFS) was the default file system for Mac OS after
[Macintosh File System (MFS)](mfs.md) and before [Apple File System (APFS)](apfs.md).

> Note that this document uses Mac OS to refer to the Macintosh Operating System in general,
> instead of specific versions like Mac OS X or macOS. Mac OS X is used to refer to version of Mac
> OS 10.0 or later.

There are multiple known variants or derivatives of HFS, such as:

* HFS
* HFS+ 8.10, used by Mac OS 8.1 to 9.2.2
* HFS+ 10.0, introduced in Mac OS 10.0
* HFSX, introduced in Mac OS 10.3

> Note that HFS can be referred to as "HFS Standard" and HFS+ or HFSX as "HFS Extended".

HFSX (or HFS/X) is an extension to HFS+ to allow additional features that are incompatible with
HFS+. One such feature is case-sensitive file names. A HFSX volume may be either case-sensitive or
case-insensitive. Case sensitivity (or lack thereof) applies to all file and directory names on the
volume.

## Overview

| Feature | HFS | HFS+ and HFSX |
| --- | --- | --- |
| Maximum file size | 231 (2 GiB) | 263 (8 EiB) |
| Maximum file name size | 31 characters | 255 characters |
| Maximum number of blocks | 216 (65535 bytes) | 232 (4294967296 bytes) |
| Character set | narrow character with codepage | Unicode UTF-16 big-endian |
| Time stamps | In local time | In UTC |
| Catalog B-tree file node size | 512 bytes | 4096 bytes |
| File attributes | none | Basic and extended |

### HFS

A HFS file system consists of:

* optional [MFS boot block](mfs.md#boot_block)
* [master directory block (MDB)](#hfs_master_directory_block)
* [volume bitmap](#hfs_volume_bitmap)
* extents overflow file
* [catalog file](#catalog_file)
* optional backup (or alternate) [master directory block (MDB)](#hfs_master_directory_block)

The backup master directory block (MDB), is stored in the last 2 sectors of the volume.

#### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | [HFS timestamp](#hfs_timestamp) in local time |
| Character strings | Narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a system defined codepage |

### HFS+ and HFSX

A HFS+ or HFSX file system consists of:

* reserved (or unused) blocks
* [volume header](#hfs_plus_volume_header)
* allocation file
* extents overflow file
* [catalog file](#catalog_file)
* optional attributes file
* optional startup file
* optional backup (or alternate) [volume header](#hfs_plus_volume_header)

The backup volume header, is stored in the last 1024 bytes of the volume.

#### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | [HFS timestamp](#hfs_timestamp) in UTC |
| Character strings | UTF-16 big-endian |

### Terminology

| Term | Description |
| --- | --- |
| Clump size | Size of the group of (allocation) blocks (or clump), in bytes, to avoid fragmentation |

### Unicode strings

Unicode strings are stored as UTF-16 big-endian in Normalization Form Canonical Decomposition (NFD)
based on Unicode 3.2, with exclusions. Unicode values in the ranges U+2000 - U+2FFF, U+F900 - U+FAFF
and U+2F800 - U+2FAFF are not decomposed.

On Mac OS 8.1 through 10.2.x decomposition was based on Unicode 2.1.

TODO: determine what the impact of the different Unicode versions is.

> Note that based on observations on Mac OS 10.15.7 on HFS+ the range U+1D000 - U+1D1FF is excluded
> from decomposition and U+2400 is replaced by U+0.

### HFS timestamp {#hfs_timestamp}

Date and time values are stored as an unsigned 32-bit integer containing the number of seconds
since January 1, 1904 at 00:00:00 (midnight), where:

* MFS and HFS use local time;
* HFS+ and HFSX use Coordinated Universal Time (UTC).

This document will refer to both forms as HFS timestamp.

The maximum representable date is February 6, 2040 at 06:28:15 UTC.

The HFS timestamp does not account for leap seconds. It includes a leap day in every year that is
evenly divisible by 4. This is sufficient given that the range of representable dates does not
contain 1900 or 2100, neither of which have leap days.

### File names

TN1150 states that HFS file names are compared in case-insensitive assuming a MacRoman encoding.

| Upper case | Lower case |
| --- | --- |
| 0x41 - 0x5a (A - Z) | 0x61 - 0x7a (a - z) |
| 0x80 (Ä) | 0x8a (ä) |
| 0x81 (Å) | 0x8c (å) |
| 0x82 (Ç) | 0x8d (ç) |
| 0x83 (É) | 0x8e (é) |
| 0x84 (Ñ) | 0x96 (ñ) |
| 0x85 (Ö) | 0x9a (ö) |
| 0x86 (Ü) | 0x9f (ü) |
| 0xae (Æ) | 0xbe (æ) |
| 0xaf (Ø) | 0xbf (ø) |
| 0xcb (À) | 0x88 (à) |
| 0xcc (Ã) | 0x8b (ã) |
| 0xcd (Õ) | 0x9b (õ) |
| 0xce (Œ) | 0xcf (œ) |
| 0xd9 (Ÿ) | 0xd8 (ÿ) |
| 0xe5 (Â) | 0x89 (â) |
| 0xe6 (Ê) | 0x90 (ê) |
| 0xe7 (Á) | 0x87 (á) |
| 0xe8 (Ë) | 0x91 (ë) |
| 0xe9 (È) | 0x8f (è) |
| 0xea (Í) | 0x92 (í) |
| 0xeb (Î) | 0x94 (î) |
| 0xec (Ï) | 0x95 (ï) |
| 0xed (Ì) | 0x93 (ì) |
| 0xee (Ó) | 0x97 (ó) |
| 0xef (Ô) | 0x99 (ô) |
| 0xf1 (Ò) | 0x98 (ò) |
| 0xf2 (Ú) | 0x9c (ú) |
| 0xf3 (Û) | 0x9e (û) |
| 0xf4 (Ù) | 0x9d (ù) |

HFS+ allows for the "/" character in file names. On Mac OS, Finder this will be represented as a
"/" but in Terminal it is replaced by ":" since the same character is used as path segment
separator. A file name with a ":" created in Terminal will be shown as "/" in Finder. Finder does
not allow the creation of a file containing ":" in the name. A symbolic link created in Terminal to
a file with a ":" in name will not convert the ":" character in the link target data. The Linux
HFS+ implementation appears to apply a similar conversion logic as Terminal.

## B-tree files {#btree_files}

HFS, HFS+ and HFSX use multiple B-trees files.

A B-tree file consists of fixed sized nodes:

* header node
* map nodes
* index (root and branch) nodes
* leaf nodes

> Note that only the data fork of a B-tree file is used. The resource fork should be unused.

The size of a B-tree file can be calculated in the following manner:

```python
size = number_of_nodes * node_size
```

### Node size

The node size is determined when the B-tree file is created.

| Feature | HFS | HFS+ and HFSX |
| --- | --- | --- |
| Node size | 512 bytes | where the value must be a power of 2 in the range 512 - 32768 |

In a HFS+ the B-tree node size is stored in the header node.

Default node sizes:

| Feature | HFS | HFS+ and HFSX |
| --- | --- | --- |
| catalog file | 512 | 4 KiB (8 KiB in Mac OS X) |
| extents overflow file | 512 | 1 KiB (4 KiB in Mac OS X) |
| attributes file | N/A | 4 KiB |

### B-tree (file) node

A B-tree file node consists of:

* node descriptor
* node records
* node record offsets

The first node in the file is referenced by node number 0.

The node offset relative to the start of the file and can be calculated in the following manner:

```python
node_offset = node_number * node_size
```

#### B-tree node descriptor

The B-tree node descriptor (BTNodeDescriptor) is 14 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Next tree node number (forward link), which contains 0 if empty |
| 4 | 4 | | Previous tree node number (backward link), which contains 0 if empty |
| 8 | 1 | | [Node type](#btree_node_types), which consists of a signed 8-bit integer |
| 9 | 1 | | Node level, which consists of a signed 8-bit integer |
| 10 | 2 | | Number of records |
| 12 | 2 | 0 | Unknown (Reserved), should contain 0 |

The root node level is 0, with a maximum depth of 8.

##### B-tree node types {#btree_node_types}

| Value | Identifier | Description |
| --- | --- | --- |
| -1 | kBTLeafNode | leaf node |
| 0 | kBTIndexNode | index node |
| 1 | kBTHeaderNode | header node |
| 2 | kBTMapNode | map node |

#### B-tree node record

The B-tree node record contains (leaf) data or a reference to an index node and consists of:

* a key
* value data

#### B-tree record offsets

The B-tree record offsets are an array of 16-bit integers relative from the start of the B-tree
node descriptor. The first record offset is found at node size - 2, e.g. 512 - 2 = 510, the
second 2 bytes before that, e.g. 508, etc.

An additional record offset is added at the end to signify the start of the free space.

> Note that the record offsets are not necessarily stored in linear order.

### B-tree header node

The B-tree header node is stored in the first node of the B-tree file and contains 3 records:

* the B-tree header record;
* the user data record, which consist of 128 bytes (reserved within HFS);
* the B-tree map record.

> Note that the records in the B-tree header node do not have keys.

#### B-tree header record {#btree_header_record}

The B-tree header record (BTHeaderRec) is 106 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Depth of the tree |
| 2 | 4 | | Root node number |
| 6 | 4 | | Number of data records contained in leaf nodes |
| 10 | 4 | | First leaf node number |
| 14 | 4 | | Last leaf node number |
| 18 | 2 | | Node size, in bytes, where the value must be a power of 2 in the range 512 - 32768 |
| 20 | 2 | | Maximum key size, in bytes |
| 22 | 4 | | Number of nodes |
| 26 | 4 | | Number of unused nodes |
| <td colspan="4">*HFS*</td> |
| 30 | 76 | | Unknown (Reserved) |
| <td colspan="4">*HFS+/HFSX*</td> |
| 30 | 2 | | Unknown (Reserved) |
| 32 | 4 | | Clump size, in bytes |
| 36 | 1 | | [B-tree file type](#btree_header_record_file_types) |
| 37 | 1 | | [Key comparision method](#btree_header_record_key_comparion_method) |
| 38 | 4 | | [Flags](#btree_header_record_flags) (or attributes) |
| 42 | 16 x 4 = 64 | | Unknown (Reserved) |

<!-- rumdl-enable MD033 MD056 -->

TODO: does the number of data records equal the number of leaf nodes?

##### File type {#btree_header_record_file_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00 | | Control file |
| 0x80 | | First user B-tree type |
| 0xff | | Reserved B-tree type |

##### Key comparision methodtype {#btree_header_record_key_comparion_method}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00 | | Unknown (not set), observed on HFS standard, HFS+ and an empty HFSX file system |
| 0xbc | | Binary compare (case-sensitive) |
| 0xcf | | Unicode case folding (case-insensitive) |

##### Flags {#btree_header_record_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | kBTBadCloseMask | Bad close, which indicates that the B-tree was not closed properly and should be checked for consistency (Not used by HFS+ and HFSX) |
| 0x00000002 | kBTBigKeysMask | Big keys, which indicates the key data size value of the keys in index and leaf nodes is 16-bit integer, otherwise, it is an 8-bit integer (Must be set for HFS+ and HFSX) |
| 0x00000004 | kBTVariableIndexKeysMask | Variable-size (index) keys, which indicates that the keys in index nodes occupy the number of bytes indicated by their key size; otherwise, the keys in index nodes always occupy maximum key size (must be set for the HFS+ and HFSX Catalog B-tree, and cleared for the HFS+ and HFSX Extents overflow B-tree) |

#### B-tree map record

The B-tree map record contains of a bitmap that indicates which nodes in the B-tree file are used
and which are not. If a bit is set, then the corresponding node in the B-tree file is in use.

The bitmap is 256 bytes in size and can represent a maximum of 2048 nodes. If more nodes are needed
a [map node](#btree_map_node) is used to store additional mappings.

### The map node {#btree_map_node}

If a B-tree file contains more than 2048 nodes, which are enough for about 8000 files, a map node
is used to store additional node-mapping information.

The next tree node value in the B-tree node descriptor of the header node is used to refer to the
first map node.

A map node consists of a B-tree node descriptor and one B-tree map record. The map record is 494
bytes in size 512 - (14 + 2) and can therefore contain mapping information for 3952 nodes.

If a B-tree contains more than 6000 nodes (enough for about 25000 files) a second map node is
needed. The next tree node value in the B-tree node descriptor of the first map node is used to
refer to the second.

If more map nodes are required, each additional map node is similarly linked to the previous one.

### The root node

The root node is the start of the B-tree structure; usually the root node is an index node, but it
might be a leaf node if there are no index nodes.

The root node number is stored in the [B-tree header record](#btree_header_record) and is 0 if the
B-tree is empty.

### The index node

The records stored in an index node are called pointer records. A pointer record consists of a key
followed by the node number of the corresponding node. The size of the key varies according to the
type of B-tree file.

* In a catalog file, the search key is a combination of the file or directory name and the parent
  identifier of that file or directory.
* In an extents overflow file, the search key is a combination of that file's type, its file
  identifier and the index of the first block in the extent.

The immediate descendants of an index node are called the children of the index node. An index node
can have from 1 to 15 children, depending on the size of the pointer records that the index node
contains.

### The leaf node

The leaf nodes contain data records. The structure of the leaf node data records varies according
to the type of B-tree.

* In an extents overflow file, the leaf node data records consist of a key and an extent record.
* In a catalog file, the leaf node data records can be any one of four kinds of records.

## HFS Master Directory Block (MDB) {#hfs_master_directory_block}

The primary Master Directory Block (MDB) (or volume information block (VIB)) is located at offset
1024 of the volume.

The MDB is 162 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "BD" (or "\x42\x44") | Volume signature |
| 2 | 4 | | Creation time, which contains a HFS timestamp in local time |
| 6 | 4 | | (last) modification time, which contains a HFS timestamp in local time |
| 10 | 2 | | [Volume attribute flags](#volume_attribute_flags) |
| 12 | 2 | | Number of files in the root directory |
| 14 | 2 | | Volume bitmap block number, contains a block number relative from the start of the volume, where 0 is the first block number, typically 3 |
| 16 | 2 | | [Next allocation search](#next_allocation_search) block number |
| 18 | 2 | | Number of blocks, where a volume can contain at most 65535 blocks |
| 20 | 4 | | Block size, in bytes, must be a multitude of 512 |
| 24 | 4 | | Clump size, in bytes |
| 28 | 2 | | Data area block number, contains a block number relative from the start of the volume, where 0 is the first block number |
| 30 | 4 | | Next available catalog node identifier (CNID), which can be a directory or file record identifier |
| 34 | 2 | | Number of unused blocks |
| 36 | 1 | | Volume label size, with a maximum of 27 |
| 37 | 27 | | Volume label |
| 64 | 4 | | (last) backup time, which contains a HFS timestamp in local time |
| 68 | 2 | | Backup sequence number |
| 70 | 4 | | Volume write count, which contains the number of times the volume has been written to |
| 74 | 4 | | Extents overflow file clump size, in bytes |
| 78 | 4 | | Catalog file clump size, in bytes |
| 82 | 2 | | Number of sub directories in the root directory |
| 84 | 4 | | Total number of files, which does not include file system metadata files |
| 88 | 4 | | Total number of directories (folders), which does not include the root folder |
| 92 | 32 | | [Finder information](#finder_information) |
| 124 | 2 | | Embedded volume signature (drVCSize) |
| 126 | 4 | | Embedded volume [extent descriptor](#hfs_extents_descriptor) (drVBMCSize and drCtlCSize) |
| 130 | 4 | | Extents overflow file size |
| 134 | 12 | | Extents overflow file [extents record](#hfs_extents_record) |
| 146 | 4 | | Catalog file size |
| 150 | 12 | | Catalog file [extents record](#hfs_extents_record) |

> Note that the volume modification time is not necessarily the data and time when the volume was
> last flushed.

### Notes

TODO: check

* drVCSize => Volume cache block size (16-bit)
* drVBMCSize => Volume bitmap cache block size (16-bit)
* drCtlCSize => Common volume cache block size (16-bit)

## HFS Volume Bitmap {#hfs_volume_bitmap}

The volume bitmap is used to keep track of block allocation. The bitmap contains one bit for each
block in the volume.

* If a bit is set, the corresponding block is currently in use by some file.
* If a bit is clear, the corresponding block is not currently in use by any file and is available.

The volume bitmap does not indicate which files occupy which blocks. The actual file-mapping
information in maintained in two locations:

* in the corresponding catalog entry;
* in the corresponding extents overflow file entry.

The size of the volume bitmap depends on the number of blocks in the volume.

A 800 KiB floppy disk with a block size of 512 bytes has a volume bitmap size of:

```python
((800 * 1024) / (512 * 8)) = 1600 bits (200 bytes).
```

A 32 MiB volume containing 32 MiB with a block size of 512 bytes has a volume bitmap size of:

```python
((32 * 1024 * 1024) / (512 * 8)) = 65536 bits (8192 bytes).
```

The number of blocks in the volume in the MDB consists of a 16-bit integer, so no more than 65535
blocks can be addressed. The volume bitmap is never larger than 8192 bytes (or 16 physical blocks).
For volumes containing more than 32 MiB of space, the block size must be increased.

A volume containing 40 MiB of space must have an block size that is at least 2 x 512 bytes.

A volume containing 80 MiB of space must have an block size that is at least 3 x 512 bytes.

## HFS+ and HFSX Volume Header {#hfs_plus_volume_header}

The volume header (HFSPlusVolumeHeader) replaces the master directory block (MDB). The volume
header starts at offset 1024 of the volume.

The block containing the first 1536 bytes (reserved space plus volume header) are marked as used in
the allocation file.

The volume header is 512 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "H+" (or "\x48\x2b") or "HX" (or "\x48\x58") | Volume signature, where "H+" (kHFSPlusSigWord) is used for HFS+ and "HX" (kHFSXSigWord) for HFSX |
| 2 | 2 | | Format version, where 4 (kHFSPlusVersion) is used for HFS+ and 5 (kHFSXVersion) for HFSX |
| 4 | 4 | | [Volume attribute flags](#volume_attribute_flags) |
| 8 | 4 | | [Last mounted version](#last_mounted_version) |
| 12 | 4 | | [Journal information block](#hfs_plus_journal_information_block) number, contains a block number relative from the start of the volume |
| 16 | 4 | | Creation time, which contains a HFS timestamp in UTC |
| 20 | 4 | | (last) content modification time, which contains a HFS timestamp in UTC |
| 24 | 4 | | (last) backup time, which contains a HFS timestamp in UTC |
| 28 | 4 | | Checked time, which contains a HFS timestamp in UTC |
| 32 | 4 | | Total number of files, which does not include file system metadata files |
| 36 | 4 | | Total number of directories (folders), which does not include the root folder |
| 40 | 4 | | Block size, in bytes |
| 44 | 4 | | Total number of blocks |
| 48 | 4 | | Number of unused blocks |
| 52 | 4 | | [Next allocation search](#next_allocation_search) block number (nextAllocation) |
| 56 | 4 | | Clump size, in bytes, of a resource fork |
| 60 | 4 | | Clump size, in bytes, of a data fork |
| 64 | 4 | | Next available catalog node identifier (CNID), which can be a directory or file record identifier |
| 68 | 4 | | Volume write count, which contains the number of times the volume has been written to |
| 72 | 8 | | Encodings bitmap |
| 80 | 32 | | [Finder information](#finder_information) |
| 112 | 80 | | Allocation file [fork descriptor](#hfs_plus_fork_descriptor_structure) |
| 192 | 80 | | Extents overflow file [fork descriptor](#hfs_plus_fork_descriptor_structure) |
| 272 | 80 | | Catalog file [fork descriptor](#hfs_plus_fork_descriptor_structure) |
| 352 | 80 | | Attributes file [fork descriptor](#hfs_plus_fork_descriptor_structure) |
| 432 | 80 | | Startup file [fork descriptor](#hfs_plus_fork_descriptor_structure) |

### Total number of blocks

For a disk whose size is an even multiple of the block size, all areas on the disk are included in
an block, including the volume header and backup volume header. For a disk whose size is not an
even multiple of the block size, only the blocks that will fit entirely on the disk are counted
here. The remaining space at the end of the disk is not used by the volume format (except for
storing the backup volume header, as described above).

### Volume attribute flags {#volume_attribute_flags}

The volume attributes flags are specified as following.

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000080 | kHFSVolumeHardwareLockBit | Volume hardware lock, set if the volume is write-protected due to a hardware setting |
| 0x00000100 | kHFSVolumeUnmountedBit | Volume unmounted, set if the volume was correctly flushed before being unmounted or ejected |
| 0x00000200 | kHFSVolumeSparedBlocksBit | Volume spared blocks, set if there are any records in the extents overflow file for bad blocks |
| 0x00000400 | kHFSVolumeNoCacheRequiredBit | Volume no cache required, set if the blocks from this volume should not be cached |
| 0x00000800 | kHFSBootVolumeInconsistentBit | Boot volume inconsistent, set if the volume was mounted for writing |
| 0x00001000 | kHFSCatalogNodeIDsReusedBit | Catalog node identifiers reused, set when the next catalog identifier value overflows 32 bits, forcing smaller catalog node identifiers to be reused |
| 0x00002000 | kHFSVolumeJournaledBit | Journaled, set if the file system uses a journal |
| 0x00004000 | kHFSVolumeInconsistentBit | Unknown (Reserved) |
| 0x00008000 | kHFSVolumeSoftwareLockBit | Volume software lock, set if the volume is write-protected due to a software setting |
| | | |
| 0x40000000 | kHFSContentProtectionBit | Unknown (Reserved) |
| 0x80000000 | kHFSUnusedNodeFixBit | Unknown (Reserved) |

### Last mounted version {#last_mounted_version}

| Value | Identifier | Description |
| --- | --- | --- |
| "8.10" | | used by Mac OS 8.1 to 9.2.2 |
| "10.0" | kHFSPlusMountVersion | used by Mac OS X |
| "FSK!" or "fsck" | | used by fsck_hfs on Mac OS X |
| "HFSJ" | kHFSJMountVersion | used by journaled HFS+ or HFSX |

### Links

TODO: add text about HFS standard

HFS+ supports both hard links and symbolic links.

Hard links to directories are not supported (allowed).

#### Hard Links

Hard links in HFS+/HFSX are represented by multiple different types of file records:

* one indirect node file record, named "iNode#", where # is the link reference. This file contains
  the content of the file shared by the hard links.
* one or more hard link file records, that reference the indirect node file record.

Indirect node files are stored in a file system metadata directory referred to as the metadata
directory with the name "/\u{2400}\u{2400}\u{2400}\u{2400}HFS+ Private Data".

The link reference corresponds to the catalog node identifier (CNID) of the indirect node file,
where 0 is not a valid link reference.

> Note that TN1150 states that a new link reference randomly chosen from the range 100
> to 1073741923. However link references that fall outside of this range have been observed such
> as "iNode20".

The special permission data of the hard link file records contains the link reference if:

* the catalog file record flag kHFSHasLinkChainMask is set;
* and the first 8 bytes of the file information contains "hlnkhfs+"

| Value | Identifier | Description |
| --- | --- | --- |
| "hlnk" | kHardLinkFileType | Hard link file type |
| "hfs+" | kHFSPlusCreator | Hard link file creator |

The hard link file's creation date should be set to the creation date of the metadata directory,
but the creation date may also be set to the creation date of the volume's root directory though
this is deprecated.

#### Device identifier

The [Special permission data](#hfs_plus_file_special_permission_data) contains the device
identifier. The device identifier can be stored in different formats, such as: "native", "386bsd",
"4bsd", "bsdos", "freebsd", "hpux", "isc", "linux", "netbsd", "osf1", "sco", "solaris", "sunos",
"svr3", "svr4" and "ultrix".

The "native" and "hpux" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Major device number |
| 1 | 2 | 0 | Unknown |
| 3 | 1 | | Minor device number |

The "386bsd", "4bsd", "freebsd", "isc", "linux", "netbsd", "sco", "sunos", "svr3" and "ultrix"
device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0 | Unknown |
| 2 | 1 | | Major device number |
| 3 | 1 | | Minor device number |

The "solaris" and "svr4" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 18 bits | | Minor device number |
| 2.2 | 14 bits | | Major device number |

The "bsdos" and "osf1" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 20 bits | | Minor device number |
| 2.4 | 12 bits | | Major device number |

The "bsdos" alternative device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 8 bits | | Sub unit number |
| 1.0 | 12 bits | | Unit number |
| 2.4 | 12 bits | | Major device number |

#### Symbolic Links

The data fork of a symbolic link contains the path of the directory or file it refers to.

On HFS+/HFSX the symbolic link target contains a POSIX pathname, as used by the Mac OS BSD and Cocoa
programming interfaces; not a traditional Mac OS or Carbon, path.

The path is stored as an UTF-8 encoded string without an end-of-string character. The length of the
path should be 1024 bytes or less. The path may be full or partial, with or without a leading
forward slash.

The first 8 bytes of the file information should contain "slnkrhap".

| Value | Identifier | Description |
| --- | --- | --- |
| "slnk" | kSymLinkFileType | Symbolic link file type |
| "rhap" | kSymLinkCreator | Symbolic link file creator |

The resource fork of a symbolic link is reserved and should be 0 bytes in size.

## The catalog file {#catalog_file}

The catalog file is a B-tree file used to maintain information about the hierarchy of files and
directories of a volume.

The block number of the first file extent of the catalog file (the header node) is stored in the
master directory block (HFS) or the volume header (HFS+). The B-tree structure is described in
section: [B-tree files](#btree_files).

Each node in the catalog file is assigned a unique catalog node identifier (CNID). The CNID is used
for both directory and file identifiers. For any given file or directory the parent identifier is
the CNID of the parent directory. The first 16 CNIDs are reserved for use by Apple and include the
following standard assignments:

| CNID | Identifier | Assignment |
| --- | --- | --- |
| 0 | | Unknown (Reserved) |
| 1 | kHFSRootParentID | Parent identifier of the root directory (folder) |
| 2 | kHFSRootFolderID | Directory identifier of the root directory (folder) |
| 3 | kHFSExtentsFileID | Extents overflow file |
| 4 | kHFSCatalogFileID | Catalog file |
| 5 | kHFSBadBlockFileID | Bad allocation block file |
| 6 | kHFSAllocationFileID | Allocation file (HFS+) |
| 7 | kHFSStartupFileID | Startup file (HFS+) |
| 8 | kHFSAttributesFileID | Attributes file (HFS+) |
| | | |
| 14 | kHFSRepairCatalogFileID | Used temporarily by fsck_hfs when rebuilding the catalog file |
| 15 | kHFSBogusExtentFileID | Bogus extent file, which is used temporarily during exchange files operations |
| 16 | kHFSFirstUserCatalogNodeID | First available CNID for user's files and folders |

### Catalog file keys

In a catalog file a key consists of:

* parent directory identifier
* (optional) file or directory name

The volume reference number is not included in the search key.

### Text encoding hint {#text_encoding_hint}

| Encoding type | Value | Encodings bitmap number |
| --- | --- | --- |
| MacRoman | 0 | 0 |
| MacJapanese | 1 | 1 |
| MacChineseTrad | 2 | 2 |
| MacKorean | 3 | 3 |
| MacArabic | 4 | 4 |
| MacHebrew | 5 | 5 |
| MacGreek | 6 | 6 |
| MacCyrillic | 7 | 7 |
| | | |
| MacDevanagari | 9 | 9 |
| MacGurmukhi | 10 | 10 |
| MacGujarati | 11 | 11 |
| MacOriya | 12 | 12 |
| MacBengali | 13 | 13 |
| MacTamil | 14 | 14 |
| MacTelugu | 15 | 15 |
| MacKannada | 16 | 16 |
| MacMalayalam | 17 | 17 |
| MacSinhalese | 18 | 18 |
| MacBurmese | 19 | 19 |
| MacKhmer | 20 | 20 |
| MacThai | 21 | 21 |
| MacLaotian | 22 | 22 |
| MacGeorgian | 23 | 23 |
| MacArmenian | 24 | 24 |
| MacChineseSimp | 25 | 25 |
| MacTibetan | 26 | 26 |
| MacMongolian | 27 | 27 |
| MacEthiopic | 28 | 28 |
| MacCentralEurRoman | 29 | 29 |
| MacVietnamese | 30 | 30 |
| MacExtArabic | 31 | 31 |
| | | |
| MacSymbol | 33 | 33 |
| MacDingbats | 34 | 34 |
| MacTurkish | 35 | 35 |
| MacCroatian | 36 | 36 |
| MacIcelandic | 37 | 37 |
| MacRomanian | 38 | 38 |
| | | |
| MacFarsi | 140 | 49 |
| | | |
| MacUkrainian | 152 | 48 |

#### HFS catalog key

The HFS catalog key is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Key data size, in bytes, which consists of a signed 8-bit integer |
| <td colspan="4">*If key data size >= 6*</td> |
| 1 | 1 | | Unknown (Reserved) |
| 2 | 4 | | Parent identifier (CNID) |
| 6 | 1 | | Name size without the end-of-string character |
| 7 | ... | | Name string, which contains a narrow character string without end-of-string character |
| ... | ... | | Unknown (Alignment padding) |

<!-- rumdl-enable MD033 MD056 -->

> Note that a key data size of 0 indicates a records that is no longer in use.

The catalog node name always is stored as 32 bytes and therefore the maximum key size within an
index node should be 37. In a leaf node the catalog node name varies in size.

Keys in a leaf node must be stored 16-bit aligned within the node data. The size of the alignment
padding is not included in the key data size.

#### HFS+ and HFSX catalog key

The HFS+ and HFSX catalog key is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Key data size, in bytes |
| <td colspan="4">*If key data size >= 4*</td> |
| 2 | 4 | | Parent identifier, which conatains a CNID |
| <td colspan="4">*If key data size >= 6*</td> |
| 6 | 2 | | Number of characters in the name string |
| 8 | ... | | Name string, which contains an UTF-16 big-endian string without end-of-string character |

<!-- rumdl-enable MD033 MD056 -->

> Note that the characters ':' and U+2400 are stored as '/' and U+0 respectively
> and must be converted before comparision.

### The catalog data

A catalog leaf node can contain four different types of records:

* a folder record, which contains information about a single directory.
* a file record, which contains information about a single file.
* a folder thread record, which provides a link between a directory and its parent directory.
* a file thread record, which provides a link between a file and its parent directory.

The thread records are used to find the name and directory identifier of the parent of a given file
or directory.

Each catalog data record consists of:

* the catalog data record header;
* the catalog data record data.

#### The catalog data record header

##### HFS catalog data record header

The HFS catalog data record header is 2 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | [Record type](#catalog_file_data_record_types), which consists of a signed 8-bit integer |
| 1 | 1 | 0x00 | Unknown (Reserved), which consists of a signed 8-bit integer |

> Note that to distinguish between HFS and HFS+ record types, record type should be treated as a
> 16-bit big-endian value.

##### HFS+ and HFSX catalog data record header

The HFS+ and HFSX catalog data record header is 2 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | [Record type](#catalog_file_data_record_types) |

##### The catalog data record types {#catalog_file_data_record_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | kHFSPlusFolderRecord | HFS+/HFSX Folder record |
| 0x0002 | kHFSPlusFileRecord | HFS+/HFSX File record |
| 0x0003 | kHFSPlusFolderThreadRecord | HFS+/HFSX Folder thread record |
| 0x0004 | kHFSPlusFileThreadRecord | HFS+/HFSX File thread record |
| | | |
| 0x0100 | kHFSFolderRecord (or cdrDirRec) | HFS Folder record |
| 0x0200 | kHFSFileRecord (or cdrFilRec) | HFS File record |
| 0x0300 | kHFSFolderThreadRecord (or cdrThdRec) | HFS Folder thread record |
| 0x0400 | kHFSFileThreadRecord (or cdrFThdRec) | HFS File thread record |

#### The catalog folder record

##### HFS catalog folder record

The HFS catalog folder record (cdrDirRec, kHFSFolderRecord) is 70 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0100 | [Record type](#catalog_file_data_record_types) |
| 2 | 2 | | [Folder flags](#hfs_catalog_folder_record_flags) |
| 4 | 2 | | Number of directory entries (valence) |
| 6 | 4 | | Identifier (CNID) |
| 10 | 4 | | Creation time, which contains a HFS timestamp in local time |
| 14 | 4 | | (last) content modification time, which contains a HFS timestamp in local time |
| 18 | 4 | | (last) backup time, which contains a HFS timestamp in local time |
| 22 | 16 | | [Folder information](#hfs_folder_information) |
| 38 | 16 | | [Extended folder information](#hfs_extended_folder_information) |
| 54 | 4 x 4 = 16 | | Unknown (Reserved), which consists of an array of 32-bit integer values |

###### HFS catalog folder record flags {#hfs_catalog_folder_record_flags}

Not defined. The HFS catalog folder record appears to always have a corresponding folder thread
record.

##### HFS+ and HFSX catalog folder record

The HFS+ and HFSX catalog folder record (HFSPlusCatalogFolder) is 88 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0001 | [Record type](#catalog_file_data_record_types) |
| 2 | 2 | | [Flags](#hfs_plus_catalog_file_record_flags) |
| 4 | 4 | | Number of directory entries (valence) |
| 8 | 4 | | Identifier (CNID) |
| 12 | 4 | | Creation time, which contains a HFS timestamp in UTC |
| 16 | 4 | | (last) content modification time, which contains a HFS timestamp in UTC |
| 20 | 4 | | (last) record (or attribute) modification (or change) time, which contains a HFS timestamp in UTC |
| 24 | 4 | | (last) access time, which contains a HFS timestamp in UTC |
| 28 | 4 | | (last) backup time, which contains a HFS timestamp in UTC |
| <td colspan="4">*Permissions*</td> |
| 32 | 4 | | Owner identifier |
| 36 | 4 | | Group identifier |
| 40 | 1 | | [Administration flags](#administration_flags) |
| 41 | 1 | | [Owner flags](#owner_flags) |
| 42 | 2 | | [File mode](#file_mode) |
| 44 | 4 | | [Special permission data](#hfs_plus_file_special_permission_data) |
| <td colspan="4">*Folder information*</td> |
| 48 | 16 | | [Folder information](#hfs_plus_folder_information) |
| <td colspan="4">*Extended folder information*</td> |
| 64 | 16 | | [Extended folder information](#hfs_plus_extended_folder_information) |
| <td colspan="4">&nbsp;</td> |
| 80 | 4 | | [Text encoding hint](#text_encoding_hint) |
| 84 | 4 | 0x00 | Unknown (Reserved) |

<!-- rumdl-enable MD033 MD056 -->

#### The catalog file record {#catalog_file_record}

##### HFS catalog file record

The HFS catalog file record (cdrFilRec, kHFSFileRecord) is 102 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0200 | [Record type](#catalog_file_data_record_types) |
| 2 | 1 | | [Flags](#hfs_catalog_file_record_flags), which consists of a signed 8-bit integer |
| 3 | 1 | 0x00 | File type, which consists of a signed 8-bit integer and should contain 0 |
| 4 | 16 | | [File information](#hfs_file_information) |
| 20 | 4 | | Identifier (CNID) |
| 24 | 2 | | Data fork block number |
| 26 | 4 | | Data fork size |
| 30 | 4 | | Data fork allocated size |
| 34 | 2 | | Resource fork block number |
| 36 | 4 | | Resource fork size |
| 40 | 4 | | Resource fork allocated size |
| 44 | 4 | | Creation time, which contains a HFS timestamp in local time |
| 48 | 4 | | (last) content modification time, which contains a HFS timestamp in local time |
| 52 | 4 | | (last) backup time, which contains a HFS timestamp in local time |
| 56 | 16 | | [Extended file information](#hfs_extended_file_information) |
| 72 | 2 | | Clump size |
| 74 | 12 | | Data fork [extents record](#hfs_extents_record) |
| 86 | 12 | | Resource fork [extents record](#hfs_extents_record) |
| 98 | 4 | 0x00 | Unknown (Reserved) |

TODO: determine if the data and resource fork block number values are used

###### HFS catalog file record flags {#hfs_catalog_file_record_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | | File is locked and cannot be written to |
| 0x0002 | | Has thread record |
| | | |
| 0x0080 | kHFSHasDateAddedMask | Had added time |

##### HFS+ and HFSX catalog file record

The HFS+ and HFSX catalog file record (kHFSPlusFileRecord) is 248 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0002 | [Record type](#catalog_file_data_record_types) |
| 2 | 2 | | [Flags](#hfs_plus_catalog_file_record_flags) |
| 4 | 4 | 0x00 | Unknown (Reserved) |
| 8 | 4 | | Identifier (CNID) |
| 12 | 4 | | Creation time, which contains a HFS timestamp in UTC |
| 16 | 4 | | (last) content  modification time, which contains a HFS timestamp in UTC |
| 20 | 4 | | (last) record (or attribute) modification time, which contains a HFS timestamp in UTC |
| 24 | 4 | | (last) access time, which contains a HFS timestamp in UTC |
| 28 | 4 | | (last) backup time, which contains a HFS timestamp in UTC |
| <td colspan="4">*Permissions*</td> |
| 32 | 4 | | Owner identifier |
| 36 | 4 | | Group identifier |
| 40 | 1 | | [Administration flags](#administration_flags) |
| 41 | 1 | | [Owner flags](#owner_flags) |
| 42 | 2 | | [File mode](#file_mode) |
| 44 | 4 | | [Special permission data](#hfs_plus_file_special_permission_data) |
| <td colspan="4">*File information*</td> |
| 48 | 16 | | [File information (or user information)](#hfs_plus_file_information) |
| <td colspan="4">*Extended file information*</td> |
| 64 | 16 | | [Extended file information (or finder information)](#hfs_plus_extended_file_information) |
| <td colspan="4">&nbsp;</td> |
| 80 | 4 | | [Text encoding hint](#text_encoding_hint) |
| 84 | 4 | 0x00 | Unknown (Reserved) |
| 88 | 80 | | Data [fork descriptor](#hfs_plus_fork_descriptor_structure) |
| 168 | 80 | | Resource [fork descriptor](#hfs_plus_fork_descriptor_structure) |

<!-- rumdl-enable MD033 MD056 -->

###### HFS+ catalog file record flags {#hfs_plus_catalog_file_record_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | kHFSFileLockedMask | File is locked and cannot be written to |
| 0x0002 | kHFSThreadExistsMask | Has thread record, which should be always set for a file record on HFS+/HSFX |
| 0x0004 | kHFSHasAttributesMask | Has extended attributes |
| 0x0008 | kHFSHasSecurityMask | Has ACLs |
| 0x0010 | kHFSHasFolderCountMask | Has number of sub-folder |
| 0x0020 | kHFSHasLinkChainMask | Has a hard link target (link chain), where the CNID of the hard link target is stored in the special permission data |
| 0x0040 | kHFSHasChildLinkMask | Has a child that is a directory link |
| 0x0080 | kHFSHasDateAddedMask | Had added time, where the extended folder of file information contains the time the folder or file was added (date_added) |
| 0x0100 | kHFSFastDevPinnedMask | Unknown |
| 0x0200 | kHFSDoNotFastDevPinMask | Unknown |
| 0x0400 | kHFSFastDevCandidateMask | Unknown |
| 0x0800 | kHFSAutoCandidateMask | Unknown |

#### The catalog thread record

The file thread record is similar to the folder thread record except that it refers to a file,
instead of a directory.

##### HFS catalog file thread record

The HFS catalog thread record (kHFSFolderThreadRecord (or cdrThdRec), kHFSFileThreadRecord (or
cdrFThdRec)) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0300 or 0x0400 | [Record type](#catalog_file_data_record_types) |
| 2 | 2 x 4 = 8 | 0x00 | Unknown (Reserved), which consists of an array of 32-bit integer values |
| 10 | 4 | | Parent identifier (CNID) |
| 14 | 1 | | Number of characters in the name string, with a maximum of 31 |
| 15 | ... | | Name string, which contains a narrow character string without end-of-string character |

##### HFS+ and HFSX catalog file thread record

The HFS+ and HFSX catalog thread record (kHFSPlusFolderThreadRecord, kHFSPlusFileThreadRecord) is of
variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0x0003 or 0x0004 | [Record type](#catalog_file_data_record_types) |
| 2 | 2 | 0x00 | Unknown (Reserved), which consists of a unsigned 16-bit integer |
| 4 | 4 | | Parent identifier (CNID) |
| 8 | 2 | | Number of characters in the name string, with a maximum of 255 |
| 10 | ... | | Name string, which contains an UTF-16 big-endian string without end-of-string character |

### Permissions

For each file and folder HFS+ maintains basic access permissions record for each file and folder.
These are similar to basic Unix file permissions.

TODO: add note about permissions on HFS

#### Owner and group identifier

The Mac OS X user ID of the owner of the file or folder. Mac OS X versions prior to 10.3 treats
user ID 99 as if it was the user ID of the user currently logged in to the console. If no user is
logged in to the console, user ID 99 is treated as user ID 0 (root). Mac OS X version 10.3 treats
user ID 99 as if it was the user ID of the process making the call (in effect, making it owned by
everyone simultaneously). These substitutions happen at run-time. The actual user ID on disk is not
changed.

The Mac OS X group ID of the group associated with the file or folder. Mac OS X typically maps
group ID 99 to the group named "unknown." There is no run-time substitution of group IDs in Mac OS
X.

#### Administration flags {#administration_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | SF_ARCHIVED | File has been archived |
| 0x02 | SF_IMMUTABLE | File is immutable and may not be changed |
| 0x04 | SF_APPEND | Writes to file may only append |

#### Owner flags {#owner_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | UF_NODUMP | Do not backup (dump) this file |
| 0x02 | UF_IMMUTABLE | File is immutable and may not be changed |
| 0x04 | UF_APPEND | Writes to file may only append |
| 0x08 | UF_OPAQUE | Directory is opaque |

#### File mode {#file_mode}

| Value | Identifier | Description |
| --- | --- | --- |
| 0xf000 (0170000) | S_IFMT | File type bitmask |
| 0x1000 (0010000) | S_IFIFO | Named pipe |
| 0x2000 (0020000) | S_IFCHR | Character-special file (Character device) |
| 0x4000 (0040000) | S_IFDIR | Directory |
| 0x6000 (0060000) | S_IFBLK | Block-special file (Block device) |
| 0x8000 (0100000) | S_IFREG | Regular file |
| 0xa000 (0120000) | S_IFLNK | Symbolic link |
| 0xc000 (0140000) | S_IFSOCK | Socket |
| 0xe000 (0160000) | S_IFWHT | Whiteout, which is a file entry that covers up all entries of a particular name from lower branches |

HFS+ uses the BSD file type and mode bits. Note that the constants from the header shown below are
in octal (base eight), not hexadecimal.

| Octal value | Identifier | Description |
| --- | --- | --- |
| 0004000 | S_ISUID | Set user identifier on execution |
| 0002000 | S_ISGID | Set group identifier on execution |
| 0001000 | S_ISTXT | Sticky bit |
| | | |
| 0000700 | S_IRWXU | Read, write and execute access for owner |
| 0000400 | S_IRUSR | Read access for owner |
| 0000200 | S_IWUSR | Write access for owner |
| 0000100 | S_IXUSR | Execute access for owner |
| | | |
| 0000070 | S_IRWXG | Read, write and execute access for group |
| 0000040 | S_IRGRP | Read access for group |
| 0000020 | S_IWGRP | Write access for group |
| 0000010 | S_IXGRP | Execute access for group |
| | | |
| 0000007 | S_IRWXO | Read, write and execute access for other |
| 0000004 | S_IROTH | Read access for other |
| 0000002 | S_IWOTH | Write access for other |
| 0000001 | S_IXOTH | Execute access for other |

> Note that if the sticky bit is set for a directory, then Mac OS restricts movement, deletion, and
> renaming of files in that directory. Files may be removed or renamed only if the user has write
> access to the directory; and is the owner of the file or the directory, or is the super-user.

#### HFS+ file special permission data {#hfs_plus_file_special_permission_data}

The special permission data is used to store the following information:

* hard link reference (iNodeNum)
* number of (hard) links (linkCount) in indirect node files
* device numbers of block (S_IFBLK) and character (S_IFCHR) devices files

### File system hierarchy

File and folder records have a search key with a non-empty name string. In thread records the name
string in the search key is empty. E.g. to list the file entries in a directory:

* find all the file or folder records given the parent CNID

Finding a file or directory by its CNID is a two-step process:

1. use the CNID to look up the thread record for the file or directory
1. use the thread record to look up the file or folder record

### File forks

Forks in HFS and HFS+ can be compared to data streams in NTFS. In HFS+ the fork
values are grouped in a separate fork descriptor structure. HFS+ also defines
extended attributes (named forks). These are not stored in the catalog file but
in the attributes file.

#### HFS+ fork descriptor structure {#hfs_plus_fork_descriptor_structure}

HFS+ maintains information about file contents using the HFS+ fork descriptor structure
(HFSPlusForkData).

The fork descriptor structure is 80 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Size, in bytes |
| 8 | 4 | | Clump size, in bytes |
| 12 | 4 | | Number of blocks |
| 16 | 64 | | Data [extents record](#hfs_plus_extents_record) |

## The extents overflow file

In HFS and HFS+ extents (contiguous ranges of blocks) are used to track which blocks belong to a
file. The first three (HFS) and eight (HFS+) are stored in the catalog file. Additional extents are
stored in the extents overflow file.

The structure of an extents overflow file is relatively simple compared to that of a catalog
file. The function of the extents overflow file is to store those file extents that are not
contained in the master directory block (MDB) or volume header and the catalog file

> Note that the file system B-tree files can have additional extents in the extents overflow
> file. This has been observed with the attributes file. It is currently unknown if the extents
> (overflow) file itself can have overflow extents.

### The extents overflow key (record)

Disks initialized using the enhanced Disk Initialization Manager introduced in system software
version might contain extent records for some blocks that do not belong to any actual file in the
file system. These extent records have been marked as a bad block (CNID 5). See the chapter "Disk
Initialization Manager" in this book for details on bad block sparing.

The key has been selected so that the extent records for a particular fork are grouped together in
the B-tree, right next to all the extent records for the other fork of the file. The fork offset of
the preceding extent record is needed to determine the key of the next extent record

In an extents overflow file the search key consists of:

* fork type
* file identifier
* first block in the extent

#### HFS extents overflow key (record)

The HFS extents overflow key (record) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | 7 | Key data size, in bytes, which consists of a signed 8-bit integer |
| 1 | 1 | | [Fork type](#hfs_fork_types), which consists of a signed 8-bit integer |
| 2 | 4 | | File identifier (CNID) |
| 6 | 2 | | Logical block number |

The first 8 extents in a fork are held in its catalog file record. So the number of extent records
for a fork is:

```python
(number_of_extents - 3 + 2) / 4
```

#### HFS+ and HFSX extents overflow key (record)

The HFS+ and HFSX extents overflow key (record) is 12 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 10 | Key data size, in bytes, which consists of an unsigned 16-bit integer |
| 2 | 1 | | [Fork type](#hfs_fork_types), which consists of a signed 8-bit integer |
| 3 | 1 | 0x00 | Unknown (Padding) |
| 4 | 4 | | File identifier (CNID) |
| 8 | 4 | | Logical block number |

The first 8 extents in a fork are held in its catalog file record. So the number of extent records
for a fork is:

```python
(number_of_extents - 8 + 7) / 8
```

#### HFS fork types {#hfs_fork_types}

| Value | Identifier | Description |
| --- | --- | --- |
| -1 (0xff) | | Resource fork |
| 0 (0x00) | | Data fork |

### The extent (data) record

An extent is a contiguous range of blocks that have been allocated to an individual file. An extent
is represented by an extent descriptor.

#### HFS extents record {#hfs_extents_record}

The HFS extents record (HFSExtentRecord) is 12 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 3 x 4 = 12 | | Array of [HFS extent descriptors](#hfs_extents_descriptor) |

#### HFS extent descriptor {#hfs_extents_descriptor}

The HFS extents descriptor (HFSExtentDescriptor) is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Physical block number, which contains a block number relative from the start of the data area |
| 2 | 2 | | Number of blocks |

```python
extent_offset = (data_area_block_number + extent_block_number) * block_size
```

An unused extent descriptor should have both the block number and number of blocks set to 0.

#### HFS+ and HFSX extents record {#hfs_plus_extents_record}

The HFS+ and HFSX extents record (HFSPlusExtentRecord) is 64 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 x 8 = 64 | | Array of [HFS+ extent descriptors](#hfs_plus_extents_descriptor) |

#### HFS+ and HFSX extent descriptor {#hfs_plus_extents_descriptor}

The HFS+ and HFSX extents descriptor (HFSPlusExtentDescriptor) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Physical block number, which contains a block number relative from the start of the volume |
| 4 | 4 | | Number of blocks |

```python
extent_offset = extent_block_number * block_size
```

An unused extent descriptor should have both the block number and number of blocks set to 0.

### Bad Block File

The extents overflow file is also used to hold information about the bad blocks; refered to as the
bad block file. The bad block file is used to mark areas on the disk as bad, unable to be used for
storing data; typically to map out bad sectors on the storage medium.

Typically, blocks are larger than sectors. If a single sector is found to be bad, the entire block
is unusable. The bad block file is sometimes used to mark blocks as unusable when they are not bad,
e.g. in the [HFS wrapper](#hfs_wrapper).

Bad block extent records are always assumed to reference the data fork (fork type of 0).

## Allocation (bitmap) file

The allocation file is uzed to keep track of whether each block in a volume is currently allocated
to some file system structure or not. The contents of the allocation file is a bitmap. The bitmap
contains one bit for each block in the volume.

* If a bit is set, the corresponding block is currently in use by some file system structure.
* If a bit is clear, the corresponding block is not currently in use, and is available for
  allocation.

The size of the allocation file depends on the number of blocks in the volume, which in turn
depends both on the size of the disk and on the size of the volume's blocks. For example, a volume
on a 1 GB disk and having an block size of 4 KB needs an allocation file size of 256 Kbits (32 KiB,
or 8 blocks). Since the allocation file itself is allocated using blocks, it always occupies an
integral number of blocks (its size may be rounded up).

The allocation file may be larger than the minimum number of bits required for the given volume
size. Any unused bits in the bitmap must be set to 0.

Each byte in the allocation file holds the state of eight blocks. The byte at offset X into the
file contains the allocation state of allocations blocks (N x 8) through (N x 8 + 7). Within each
byte, the most significant bit holds information about the block with the lowest number, the least
significant bit holds information about the block with the highest number. Listing 1 shows how you
would test whether an block is in use, assuming that you've read the entire allocation file into
memory.

```text
Determining whether a block is in use.

static Boolean IsAllocationBlockUsed(UInt32 thisAllocationBlock,
                                     UInt8 *allocationFileContents)
{
    UInt8 thisByte;

    thisByte = allocationFileContents[thisAllocationBlock / 8];
    return (thisByte & (1 << (7 - (thisAllocationBlock % 8)))) != 0;
}
```

## Attributes file {#hfs_plus_attributes_file}

The attributes file is a B-tree file used to store extended attributes.

The location of the attributes file can be found in the HFS+ and HFSX volume header.

### Attributes file keys

An attributes file key is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Key data size, in bytes |
| <td colspan="4">*If key data size >= 12*</td> |
| 2 | 2 | | Unknown |
| 4 | 4 | | Identifier (CNID) |
| 8 | 4 | | Unknown |
| 12 | 2 | | Number of characters in the name string |
| 14 | ... | | Name string, which contains an UTF-16 big-endian string without end-of-string character |

<!-- rumdl-enable MD033 MD056 -->

> Note that the name of an extended attribute appears to be case senstive even on a case insensitive
> file system.

### The attributes file data

The attributes file defines two types of attributes:

1. Fork data attributes, which are used for attributes whose data is large. The attribute's data is
   stored in extents on the volume and the attribute merely contains a reference to those extents.
1. Extension attributes, which are used to augment fork descriptor structure, allowing a forks to
   have more than eight extents.

#### Attributes file data record header

Each attributes file data record starts with a type value, which describes the type of attribute
data record.

The attributes file data record header is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Record type](#attributes_file_data_record_types) |

##### The attributes data record types {#attributes_file_data_record_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000010 | kHFSPlusAttrInlineData | Attribute record with inline data |
| 0x00000020 | kHFSPlusAttrForkData | Attribute record with fork descriptor |
| 0x00000030 | kHFSPlusAttrExtents | Attribute record with extents overflow |

> Note that at the moment it is unclear when an attribute record of type
> kHFSPlusAttrExtents is created and how it should be handled.

#### The inline data attribute record

The inline data attribute record is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | 0x00000010 | [Record type](#attributes_file_data_record_types) |
| 4 | 4 | 0 | Unknown (Reserved) |
| 8 | 4 | | Unknown |
| 12 | 4 | | Attribute data size |
| 16 | ... | | Attribute data |

#### The fork descriptor attribute record

The fork descriptor attribute record is 88 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | 0x00000020 | [Record type](#attributes_file_data_record_types) |
| 4 | 4 | 0 | Unknown (Reserved) |
| 8 | 80 | | Attribute [fork descriptor](#hfs_plus_fork_descriptor_structure) |

#### The extents attribute record

The extents attribute record is 72 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | 0x00000030 | [Record type](#attributes_file_data_record_types) |
| 4 | 4 | 0 | Unknown (Reserved) |
| 8 | 64 | | Attribute [extents record](#hfs_plus_extents_record) |

### Compressed data extended attribute

The compressed extended attribute is named "com.apple.decmpfs" and consists of:

* compressed data header
* optional compressed data

#### Compressed data header {#compressed_data_header}

The compressed data header is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "fpmc" | Signature |
| 4 | 4 | | [Compression method](#compression_methods) |
| 8 | 8 | | Uncompressed data size |

> Note that the signature is likely stored in little-endian and represents "cmpf".

#### Compression methods {#compression_methods}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | CMP_Type1 | Unknown (uncompressed extended attribute data) |
| | | |
| 3 | | ZLIB (DEFLATE) compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 4 | | 64k chunked ZLIB (DEFLATE) compressed resource fork, where the compressed data is stored in the resource fork |
| 5 | | Unknown (sparse compressed extended attribute data), where the uncompressed data contains 0-byte values |
| 6 | | Unknown (unused) |
| 7 | | LZVN compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 8 | | 64k chunked LZVN compressed resource fork, where the compressed data is stored in the resource fork |
| 9 | | Unknown (uncompressed extended attribute data, different than CMP_Type1) |
| 10 | | Unknown (64k chunked uncompressed data resource fork), where the compressed data is stored in the resource fork |
| 11 | | LZFSE compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 12 | | 64k chunked LZFSE compressed resource fork, where the compressed data is stored in the resource fork |
| | | |
| 0x80000001 | | Unknown (faulting file) |

<!-- rumdl-disable MD028 -->

> Note that if the ZLIB (DEFLATE) compressed data starts with 0xff the data is stored uncompressed
> after the first compressed data byte.

> Note that if the LZVN compressed data starts with 0x06 (end of stream oppcode) the data is stored
> uncompressed after the first compressed data byte.

<!-- rumdl-enable MD028 -->

## Startup file

The startup file is a file system metadata file intended to hold information needed when
booting a system that does not have built-in (ROM) support for HFS+ (or HFSX). A boot loader can
find the startup file without full knowledge of the format using the first eight extents of the
startup file located in the volume header.

Format wise it is valid for the startup file to contain more than eight extents, but in doing so
the purpose of the startup file is defeated.

## Next allocation search {#next_allocation_search}

The next block number is used by Mac OS as a hint for where to start searching for available blocks
when allocating space for a file.

## Metadata zone and hot files

In Mac OS X 10.3 a metadata zone was instroduced to store certain file system metadata, such as
allocation bitmap file, extents overflow file, and the catalog file, the journal file and
frequently used small files (also referred to as "hot files") near each other to reduces seek time
for typical accesses.

### Hot File B-tree

The hot file B-tree is a file named ".hotfiles.btree" stored the root directory.

## Journal

A HFS+ (or HFSX) volume may have an optional journal to speed recovery when mounting a volume that
was not unmounted safely. The purpose of the journal is to ensure that when a group of related
changes are being made, that either all of those changes are actually made, or none of them are
made. The journal makes it quick and easy to restore the volume structures to a consistent state,
without having to scan all of the structures. The journal is used only for the volume structures
and metadata; it does not protect the contents of a fork.

The volume header specifies if journalling is activated.

The journal data stuctures consist of:

* a journal information block, contains the location and size of the journal header and journal
  buffer;
* a journal header, describes which part of the journal buffer is active and contains transactions
  waiting to be committed;
* a journal buffer, a cyclic buffer to hold the file system meta data transactions.

On HFS+ volumes, the journal information block is stored as a file. The name of that file is
".journal_info_block" and it is stored in the volume's root directory.

The journal header and journal buffer are stored together in a different file named ".journal",
also in the volume's root directory. Each of these files are contiguous on disk, they occupy
exactly one extent.

The volume header contains the extent of the journal information block file. The journal
information block contains the location of the journal file.

### Journal information block {#hfs_plus_journal_information_block}

The journal information block describes where the journal header and journal buffer are stored. The
journal information block is stored at the start of the block referred to by the volume header.

The journal information block is 44 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Journal flags](#hfs_plus_journal_flags) |
| 4 | 8 x 4 = 32 | | Device signature |
| 36 | 8 | | Journal header offset |
| 44 | 8 | | Journal size, in bytes, which includes the size of the journal header and the journal buffer, but not the journal information block |
| 52 | 32 x 4 = 128 | 0x00 | Unknown (Reserved) |

#### Journal flags {#hfs_plus_journal_flags}

The journal flags consist of the following values:

| Value(s) | Description |
| --- | --- |
| 0x00000001 | On volume, where the journal header offset is relative to the start of the volume |
| 0x00000002 | On other device, where the device signature identifies the device containing the journal and the journal header offset is relative to the start of the device |
| 0x00000004 | Needs initialization, to indicate that there are no valid transactions in the journal and needs to be initialized |

> Note that according to TN1150 journals stored on a separate device are not supported.

### The journal header

The journal header is 44 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "\x4a\x4e\x4c\x78" | Signature |
| 4 | 4 | "\x12\x34\x56\x78" | Byte order (or endian) signature |
| 8 | 8 | | First transaction start offset |
| 16 | 8 | | Next transaction start offset |
| 24 | 8 | | Journal size, in bytes, which includes the size of the journal header and buffer |
| 32 | 4 | | Journal block header size, in bytes, typically ranges from 4096 to 16384 |
| 36 | 4 | | [checksum](#journal_checksum) |
| 40 | 4 | | Journal header size, in bytes, typically the size of one sector |

#### First and next transaction offset

The first transaction offset contains the offset in bytes from the start of the journal header to
the start of the first (oldest) transaction.

The next transaction offset contains the offset in bytes from the start of the journal header to
the end of the last (newest) transaction. Note that this field may be less than the start field,
indicating that the transactions wrap around the end of the journal's circular buffer. If end
equals start, then the journal is empty, and there are no transactions that need to be replayed.

### Journal transactions

A single transaction is stored in the journal as several blocks. These blocks include both the data
to be written and the location where that data is to be written. This is represented on storage
medium by a block list header, which describes the number and sizes of the blocks, immediately
followed by the contents of those blocks.

Since block list headers are of limited size, a single transaction may consist of several block
list headers and their associated block contents. If the next value in the first block information
structure is non-zero, then the next block list header is a continuation of the same transaction.

The journal buffer is treated as a circular buffer. When reading or writing the journal buffer, the
I/O operation must stop at the end of the journal buffer and resume (wrap around) immediately
following the journal header. Block list headers or the contents of blocks may wrap around in this
way. Only a portion of the journal buffer is active at any given time; this portion is indicated by
the start and end fields of the journal header. The part of the journal buffer that is not active
contains no meaningful data, and must be ignored.

To prevent ambiguity when start equals end, the journal is never allowed to be perfectly full (all
of the journal buffer used by block lists and blocks). If the journal was perfectly full, and start
was not equal to jhdr_size, then end would be equal to start. You would then be unable to
differentiate between an empty and full journal.

When the journal is not empty (contains transactions), it must be replayed to be sure the volume is
consistent. That is, the data from each of the transactions must be written to the correct blocks
on disk.

### The journal block list header

The block list header describes a list of blocks included in a transaction. A transaction may
include several block lists if it modifies more blocks than can be represented in a single block
list.

The journal block list header is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Maximum number of journal blocks |
| 2 | 2 | | Number of journal blocks following the journal block header, typically 1 |
| 4 | 4 | | Block list size, in bytes, which includess the size of the header and blocks |
| 8 | 4 | | [Checksum](#journal_checksum) |
| 12 | 4 | 0x00 | Unknown (Alignment padding) |
| 16 | ... | | Journal block information array |

> Note that the number of journal blocks includes the first journal block, The first journal block
> is reserved to be used when multiple blocks need to be chained, therefore the number of journal
> blocks actually containing data is minus one (-1).

### Journal block information

The journal block information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Block sector number |
| 8 | 4 | | Block size, in bytes |
| 12 | 4 | | Next journal block |

### Journal checksum {#journal_checksum}

The journal header and block list header both contain checksum values. The checksums are verified
as part of a basic consistency check of these journal data structures. To verify the checksum,
temporarily set the checksum field to 0 and then call the hfs_plus_calculate_checksum routine as
specified below.

```text
uint32_t hfs_plus_calculate_checksum(
          uint8_t *buffer,
          size_t buffer_size )
{
    size_t buffer_offset = 0;
    uint32_t checksum    = 0;

    for( buffer_offset = 0;
         buffer_offset < buffer_size;
         buffer_offset++)
    {
        checksum = ( checksum << 8 ) ^ ( checksum + buffer[ buffer_offset ] );
    }
    return( ~checksum );
}
```

## Application specific data structures

HFS, HFS+ and HFSX contain application specific data structures.

### Finder information {#finder_information}

The finder information in the master directory block (MDB) and volume header consists of an array
of 32-bit values. This array contains information used by the Mac OS Finder and the system software
boot process.

| Array entry | Description |
| --- | --- |
| 0 | Bootable system directory identifier (CNID), i.e. "System Folder" in Mac OS 8 or 9, or "/System/Library/CoreServices" in Mac OS X. Typically 3 or 5, is 0 if the volume is not bootable |
| 1 | Startup application parent identifier (CNID), i.e. "Finder". Is 0 if the volume is not bootable |
| 2 | Directory identifier (CNID) to display in Finder on mount, or 0 if none |
| 3 | Directory identifier (CNID) of a bootable Mac OS 8 or 9 System Folder, or 0 if none |
| 4 | Unknown (Reserved) |
| 5 | Directory identifier (CNID) of a bootable Mac OS X system, the "/System/Library/CoreServices" directory, or 0 if none |
| 6 and 7 | Mac OS X volume identifier, consist of a 64-bit integer |

### File information

#### HFS file information {#hfs_file_information}

The HFS file information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x 1 = 4 | | File type, which consists of an array of unsigned 8-bit integers |
| 4 | 4 x 1 = 4 | | File creator, which consists of an array of unsigned 8-bit integers |
| 8 | 2 | | [Finder flags](#finder_flags) |
| 10 | 4 | | Location within the parent, which contains x and y-coordinate values. If set to {0, 0}, the Finder will place the item automatically |
| 14 | 2 | | File icon window, which contains the window in which the file's icon appears |

#### HFS extended file information {#hfs_extended_file_information}

The HFS extended file information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Finder icon identifier |
| 2 | 3 x 2 = 6 | | Unknown (Reserved), which consists of an array of signed 16-bit integers |
| 8 | 1 | | Extended finder script code flags |
| 9 | 1 | | [Extended finder flags](#extended_finder_flags) |
| 10 | 2 | | Finder comment identifier, which consists of a signed 16-bit integer |
| 12 | 4 | | Put away folder identifier (CNID) |

#### HFS+ and HFSX file information {#hfs_plus_file_information}

The HFS+ and HFSX file information (FileInfo) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x 1 = 4 | | File type, which consists of an array of unsigned 8-bit integers |
| 4 | 4 x 1 = 4 | | File creator, which consists of an array of unsigned 8-bit integers |
| 8 | 2 | | [Finder flags](#finder_flags) |
| 10 | 4 | | Location within the parent, which contains x and y-coordinate values. If set to {0, 0}, the Finder will place the item automatically |
| 14 | 2 | | Unknown (Reserved) |

#### HFS+ and HFSX extended file information {#hfs_plus_extended_file_information}

The HFS+ and HFSX extended file information (ExtendedFileInfo) is 16 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown (Reserved) |
| <td colspan="4">*If kHFSHasDateAddedMask is not set*</td> |
| 4 | 4 | | Unknown (Reserved) |
| <td colspan="4">*If kHFSHasDateAddedMask is set*</td> |
| 4 | 4 | | Added time, which contains a POSIX timestamp in UTC |
| <td colspan="4">*Common*</td> |
| 8 | 2 | | [Extended finder flags](#extended_finder_flags) |
| 10 | 2 | | Unknown (Reserved), which consists of a signed 16-bit integer |
| 12 | 4 | | Put away folder identifier (CNID) |

<!-- rumdl-enable MD033 MD056 -->

### Folder information

#### HFS folder information {#hfs_folder_information}

The HFS folder information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Window position and dimension (boundaries), which contains the top, left, bottom, right-coordinate values |
| 8 | 2 | | [Finder flags](#finder_flags) |
| 10 | 4 | | Location within the parent, which contains x and y-coordinate values. If set to {0, 0}, the Finder will place the item automatically |
| 14 | 2 | | Folder view |

#### HFS extended folder information {#hfs_extended_folder_information}

The HFS extended folder information is 16 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Scroll position for icon view, which contains x and y-coordinate values |
| <td colspan="4">*If kHFSHasDateAddedMask is not set*</td> |
| 4 | 4 | | Open folder identifier chain, which consists of a signed 32-bit integer |
| <td colspan="4">*If kHFSHasDateAddedMask is set*</td> |
| 4 | 4 | | Added time, which contains a POSIX timestamp in UTC |
| <td colspan="4">*Common*</td> |
| 8 | 1 | | Extended finder script code flags |
| 9 | 1 | | [Extended finder flags](#extended_finder_flags) |
| 10 | 2 | | Finder comment identifier, which consists of a signed 16-bit integer |
| 12 | 4 | | Put away folder identifier (CNID) |

<!-- rumdl-enable MD033 MD056 -->

#### HFS+ and HFSX folder information {#hfs_plus_folder_information}

The HFS+ and HFSX folder information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Window position and dimension (boundaries), which contains the top, left, bottom, right-coordinate values |
| 8 | 2 | | [Finder flags](#finder_flags) |
| 10 | 4 | | Location within the parent, which contains x and y-coordinate values. If set to {0, 0}, the Finder will place the item automatically |
| 14 | 2 | | Unknown (Reserved) |

#### HFS+ and HFSX extended folder information {#hfs_plus_extended_folder_information}

The HFS+ and HFSX extended folder information is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Scroll position for icon view, which contains x and y-coordinate values |
| 4 | 4 | | Unknown (Reserved), which consists of a signed 32-bit integer |
| 8 | 2 | | [Extended finder flags](#extended_finder_flags) |
| 10 | 2 | | Unknown (Reserved), which consists of a signed 16-bit integer |
| 12 | 4 | | Put away folder identifier (CNID) |

### Finder flags {#finder_flags}

The finder flags consists of the following values:

| Value(s) | Applies to | Description |
| --- | --- | --- |
| 0x0001 | Files and folders | Is on desktop |
| 0x000e | Files and folders | Color |
| 0x0040 | Files | Is shared |
| 0x0080 | Files | Has no INITs |
| 0x0100 | Files | Has been inited |
| 0x0400 | Files and folders | Has custom icon |
| 0x0800 | Files | Is stationary |
| 0x1000 | Files and folders | Name locked |
| 0x2000 | Files | Has bundle |
| 0x4000 | Files and folders | Is invisible |
| 0x8000 | Files | Is alias |

### Extended finder flags {#extended_finder_flags}

The extended finder flags consists of the following values:

| Value(s) | Description |
| --- | --- |
| 0x0004 | Has routing information |
| 0x0100 | Has custom badge resource |
| 0x8000 | Extended flags are invalid, which indicates that set the other extended flags should be ignored |

#### Notes

```text
struct Point {
  SInt16              v;
  SInt16              h;
};
typedef struct Point  Point;

struct Rect {
  SInt16              top;
  SInt16              left;
  SInt16              bottom;
  SInt16              right;
};
typedef struct Rect   Rect;

/* OSType is a 32-bit value made by packing four 1-byte characters
   together. */
typedef UInt32        FourCharCode;
typedef FourCharCode  OSType;
```

## File content

HFS supports multiple ways to store file content:

* Data fork
* Compressed data extended attribute
* Compressed data extended attribute with resource fork
* Resource fork
* Extended attribute (named fork)

### Data fork

The file content size is stored in the data fork descriptor of the
[catalog file record](#catalog_file_record).

The extents of the file content are stored in the fork descriptor and extents overflow file.

### Compressed data extended attribute

[Compression method](#compression_methods) should be 3, 5 or 7.

The file content size is stored in the compressed data header of a
"com.apple.decmpfs" extended attribute.

For compression method 3 or 7 the file content data is stored in a
"com.apple.decmpfs" extended attribute after the [compressed data header](#compressed_data_header).

For compression method 5 the file content data contains 0-byte values. There
are 12 bytes stored after the [compressed data header](#compressed_data_header)
that contain:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown (Seen: 1) |
| 4 | 4 | | Unknown |
| 8 | 4 | | Unknown (Seen: 0) |

### Compressed data extended attribute with resource fork

[Compression method](#compression_methods) should be 4 or 8.

The file content size is stored in the compressed data header of a "com.apple.decmpfs" extended
attribute.

The file content data is stored in a "com.apple.ResourceFork" extended attribute.

The compressed data starts with metadata that contains the offsets of the compressed data blocks.

#### ZLIB (DEFLATE) compressed data

* ZLIB (DEFLATE) compressed header
* Unknown (empty values)
* ZLIB (DEFLATE) compressed data block offsets and sizes
* ZLIB (DEFLATE) compressed data blocks
* ZLIB (DEFLATE) compressed footer

##### ZLIB (DEFLATE) compressed header

The ZLIB (DEFLATE) compressed header is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Compressed data block descriptors offset, where the offset is relative from the start of the ZLIB (DEFLATE) compressed data |
| 4 | 4 | | Compressed footer offset, where the offset is relative from the start of the ZLIB (DEFLATE) compressed data |
| 8 | 4 | | Compressed data block descriptors and data size |
| 12 | 4 | | Compressed footer size |

> Note that the values in the ZLIB (DEFLATE) compressed header are stored in big-endian.

##### ZLIB (DEFLATE) compressed data block descriptors

The ZLIB (DEFLATE) compressed data block descriptors are of variable size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Compressed data size |
| 4 | 4 | | Number of compressed data block offset and size tuples |
| 8 | 8 x ... | | Array of compressed data block descriptors |

##### ZLIB (DEFLATE) compressed data block descriptor

The ZLIB (DEFLATE) compressed data block descriptor is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Compressed block offset, where the offset is relative from the start of the ZLIB (DEFLATE) compressed data + 20 |
| 4 | 4 | | Compressed block size |

##### ZLIB (DEFLATE) compressed footer

The ZLIB (DEFLATE) compressed footer is 50 bytes size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 24 | | Unknown (empty values) |
| 24 | 2 | | Unknown |
| 26 | 2 | | Unknown |
| 28 | 2 | | Unknown |
| 30 | 2 | | Unknown |
| 32 | 4 | "cmpf" | Unknown (signature) |
| 36 | 4 | | Unknown |
| 40 | 4 | | Unknown |
| 44 | 6 | | Unknown (empty values) |

> Note that the values in the ZLIB (DEFLATE) compressed footer are stored in big-endian.

#### LZVN compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the LZVN compressed data |
| ... | ... | | LZVN compressed data blocks |

> Note that the compressed data block contains a maximum of 65536 bytes of data. The compressed
> data block therefore should not exceed 65537 bytes in size.

### Resource fork

The file content size is stored in the resource fork descriptor of the
[catalog file record](#catalog_file_record).

The extents of the file content are stored in the fork descriptor and extents overflow file.

### Extended attribute (named fork)

Extended attributes, also referred to as named forks, are stored in the
[HFS+ attributes file](#hfs_plus_attributes_file).

## HFS wrapper {#hfs_wrapper}

TODO: complete section

A HFSX volume cannot be wrapped in a HFS volume.

## References

* [hfs_format.h](https://github.com/apple-oss-distributions/hfs/blob/main/core/hfs_format.h)
* [Data Organization on Volumes](https://developer.apple.com/library/archive/documentation/mac/Files/Files-99.html), by Apple Inc.
* [Technical Note TN1150: HFS plus volume format](https://developer.apple.com/library/archive/technotes/tn/tn1150.html), by Apple Inc.
