# New Technologies File System (NTFS) format

The New Technologies File System (NTFS) format is the primary file system for
Microsoft Windows versions that are based on Windows NT.

## Overview

An New Technologies File System (NTFS) consists of:

* boot record
* boot loader
* Master File Table (MFT)
* Mirror Master File Table (MFT)

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | FILETIME in UTC
| Character strings | UCS-2 little-endian, which allows for unpaired Unicode surrogates such as "U+d800" and "U+dc00"

### Versions

| Format version | Remarks
| --- | ---
| 1.0 | Introduced in Windows NT 3.1
| 1.1 | Introduced in Windows NT 3.5, also seen to be used by Windows NT 3.1
| 1.2 | Introduced in Windows NT 3.51
| 3.0 | Introduced in Windows 2000
| 3.1 | Introduced in Windows XP

> Note that the format versions mentioned above are the version as used by NTFS.
> Another common versioning schema uses the Windows version, e.g. NTFS 5.0 is the
> version of NTFS used on Windows XP which is version 3.1 in schema mentioned
> above.

> Note Windows does not necessarily uses the latest version, e.g. Windows 10
> (1809) has been observed to use NTFS version 1.2 for 64k cluster block size.

### Terminology

#### Cluster

NTFS refers to it file system blocks as clusters. Note that these are not the
same as the physical clusters of a harddisk. For clarity this document will
refer to these as cluster blocks. In other sources they are also referred to as
logical clusters.

Typically a cluster block is 8 sectors (or 8 x 512 = 4096 bytes) in size. A
cluster block number is relative to the start of the boot record.

#### Virtual cluster

The term virtual cluster refers to cluster blocks which are relative to the
start of a data stream.

#### Long and short (file) name

In Windows terminology the name of a file (or directory) can either be short or
long. The short name is an equivalent of the filename in the (DOS) 8.3 format.
The long name is actual the (full) name of the file. The term long refers to
the aspect that the name is longer than the short variant. Because most
documentation refer to the (full) name as the long name, for clarity sake so
will this document.

#### Metadata files

NTFS uses the Master File Table (MFT) to store information about files and
directories. The MFT entries reference the different volume and file system
metadata. There are several predefined metadata files.

The following metadata files are predefined and use a fixed MFT entry number.

| MFT entry number | Filename | Description
| --- | --- | ---
| 0 | "$MFT" | Master File Table
| 1 | "$MFTMirr" | Back up of the first 4 entries of the Master File Table
| 2 | "$LogFile" | Metadata transaction journal
| 3 | "$Volume" | Volume information
| 4 | "$AttrDef" | MFT entry attribute definitions
| 5 | "." | Root directory
| 6 | "$Bitmap" | Cluster block allocation bitmap
| 7 | "$Boot" | Boot record (or boot code)
| 8 | "$BadClus" | Bad clusters
| <td colspan="3"> *Used in NTFS version 1.2 and earlier*
| 9 | "$Quota" | Quota information
| <td colspan="3"> *Used in NTFS version 3.0 and later*
| 9 | "$Secure" | Security and access control information
| <td colspan="3"> *Common*
| 10 | "$UpCase" | Case folding mappings
| 11 | "$Extend" | A directory containing extended metadata files
| 12-15 | | Unknown (Reserved), which are marked as in-use but are empty
| 16-23 | | Unused, which are marked as unused
| <td colspan="3"> *Used in NTFS version 3.0 and later*
| 24 | "$Extend\$Quota" | Quota information
| 25 | "$Extend\$ObjId" | Unique file identifiers for distributed link tracking
| 26 | "$Extend\$Reparse" | Backreferences to reparse points
| <td colspan="3"> *[Transactional NTFS metadata files](#transactional_ntfs), which have been observed in Windows Vista and later*
| 27 | "$Extend\$RmMetadata" | Resource manager metadata directory
| 28 | "$Extend\$RmMetadata\$Repair" | Repair information
| 29 or 30 | "$Extend\$RmMetadata\$TxfLog" | Transactional NTFS (TxF) log metadata directory
| 30 or 31 | "$Extend\$RmMetadata\$Txf" | Transactional NTFS (TxF) metadata directory
| 31 or 32 | "$Extend\$RmMetadata\$TxfLog\$Tops" | TxF Old Page Stream (TOPS) file, which is used to store data that has been overwritten inside a currently active transaction
| 32 or 33 | "$Extend\$RmMetadata\$TxfLog\$TxfLog.blf" | Transactional NTFS (TxF) base log metadata file
| <td colspan="3"> *Observed in Windows 10 and later*
| 29 | "$Extend\$Deleted" | Temporary location for files that have an open handle but a request has been made to delete them
| <td colspan="3"> *Common*
| | ... | A file or directory

The following metadata files are predefined, however the MFT entry number is
commonly used but not fixed.

| MFT entry number | Filename | Description
| --- | --- | ---
| | "$Extend\$UsnJrnl" | [USN change journal](#usn_change_journal)

## The boot record

The boot record is stored at the start of the volume (in the $Boot metadata
file) and contains:

* the file system signature
* the BIOS parameter block
* the boot loader

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 3 | | Boot entry point
| 3 | 8 | "NTFS\x20\x20\x20\x20" | File system signature (Also known as OEM identifier or dummy identifier)
| <td colspan="4"> *DOS version 2.0 BIOS parameter block (BPB)*
| 11 | 2 | | Bytes per sector. Note that the following values are supported by mkntfs: 256, 512, 1024, 2048 and 4096
| 13 | 1 | | Number of sectors per cluster block
| 14 | 2 | 0 | Unknown (Reserved Sectors), which is not used by NTFS and must be 0
| 16 | 1 | 0 | Number of File Allocation Tables (FATs), which is not used by NTFS and must be 0
| 17 | 2 | 0 | Number of root directory entries, which is not not used by NTFS and must be 0
| 19 | 2 | 0 | Number of sectors (16-bit), which is not used by NTFS must be 0
| 21 | 1 | | [Media descriptor](#media_descriptor)
| 22 | 2 | 0 | Sectors Per File Allocation Table (FAT), which is not used by NTFS and must be 0
| <td colspan="4"> *DOS version 3.4 BIOS parameter block (BPB)*
| 24 | 2 | 0x3f | Sectors per track, which is not used by NTFS
| 26 | 2 | 0xff | Number of heads, which is not used by NTFS
| 28 | 4 | 0x3f | Number of hidden sectors, which is not used by NTFS
| 32 | 4 | 0x00 | Number of sectors (32-bit), which is not used by NTFS must be 0
| <td colspan="4"> *NTFS version 8.0 BIOS parameter block (BPB) or extended BPB, which was introduced in Windows NT 3.1*
| 36 | 1 | 0x80 | Unknown (Disc unit number), which is not used by NTFS
| 37 | 1 | 0x00 | Unknown (Flags), which is not used by NTFS
| 38 | 1 | 0x80 | Unknown (BPB version signature byte), which is not used by NTFS
| 39 | 1 | 0x00 | Unknown (Reserved), which is not used by NTFS
| 40 | 8 | | Number of sectors (64-bit)
| 48 | 8 | | Master File Table (MFT) cluster block number
| 56 | 8 | | Mirror MFT cluster block number
| 64 | 4 | | MFT entry size
| 68 | 4 | | Index entry size
| 72 | 8 | | Volume serial number
| 80 | 4 | 0 | Checksum, which is not used by NTFS
| <td colspan="4"> &nbsp;
| 84 | 426 | | Boot code
| 510 | 2 | "\x55\xaa" | The (boot) signature

### Boot entry point

The boot entry point often contains a jump instruction to the boot code at
offset 84 followed by a no-operation, e.g.

```
eb52   jmp 0x52
90     nop
```

### Number of sectors per cluster block

The number of sectors per cluster block value as used by mkntfs is defined as
following:

* Values 0 to 128 represent sizes of 0 to 128 sectors.
* Values 244 to 255 represent sizes of `2^(256-n)` sectors.
* Other values are unknown.

### Cluster block size

The cluster block size can be determined as following:

```
cluster block size = bytes per sector x sectors per cluster block
```

Different NTFS implementations support different cluster block sizes. Known
supported cluster block size:

| Cluster block size | Bytes per sector | Supported by
| --- | --- | ---
| 256 | 256 | mkntfs
| 512 | 256 - 512 | mkntfs, ntfs3g, Windows
| 1024 | 256 - 1024 | mkntfs, ntfs3g, Windows
| 2048 | 256 - 2048 | mkntfs, ntfs3g, Windows
| 4096 | 256 - 4096 | mkntfs, ntfs3g, Windows
| 8192 | 256 - 4096 | mkntfs, ntfs3g, Windows
| 16K (16384) | 256 - 4096 | mkntfs, ntfs3g, Windows
| 32K (32768) | 256 - 4096 | mkntfs, ntfs3g, Windows
| 64K (65536) | 256 - 4096 | mkntfs, ntfs3g, Windows
| 128K (131072) | 256 - 4096 | mkntfs, ntfs3g, Windows 10 (1903)
| 256K (262144) | 256 - 4096 | mkntfs, ntfs3g, Windows 10 (1903)
| 512K (524288) | 256 - 4096 | mkntfs, ntfs3g, Windows 10 (1903)
| 1M (1048576) | 256 - 4096 | mkntfs, ntfs3g, Windows 10 (1903)
| 2M (2097152) | 512 - 4096 | mkntfs, ntfs3g, Windows 10 (1903)

> Note that Windows 10 (1903) requires the partition containing the NTFS file
> system to be aligned with the cluster block size. For example for a cluster
> block size of 128k the partition must 128 KiB aligned. The default partition
> partition alignment appears to be 64 KiB.

mkntfs restricts the cluster size to:

```
bytes per sector >= cluster size > 4096 x bytes per sector
```

### Master File Table (MFT) offset

The Master File Table (MFT) offset can be determined as following:

```
MFT offset = boot record offset + ( MFT cluster block number x Cluster block size )
```

The lower 32-bit part of the NTFS volume serial number is the Windows API
(WINAPI) volume serial number. This can be determined by comparing the output
of:

```
fsutil fsinfo volumeinfo C:
fsutil fsinfo ntfsinfo C:
```

Often the total number of sectors in the boot record will be smaller than the
underlying partition. A (nearly identical) backup of the boot record is stored
in last sector of cluster block, that follows the last cluster block of the
volume. Often this is the 512 bytes after the last sector of the volume, but
not necessarily. The backup boot record is not included in the total number of
sectors.

### Master File Table (MFT) and index entry size

The Master File Table (MFT) entry size and index entry size are defined as
following:

* Values 0 to 127 represent sizes of 0 to 127 cluster blocks.
* Values 128 to 255 represent sizes of `2^(256-n)` bytes; or `2^(-n)` if considered as a signed byte.
* Other values are not considered valid.

### BitLocker Drive Encryption (BDE)

BitLocker Drive Encryption (BDE) uses the file system signature: "-FVE-FS-".
Where FVE is an abbreviation of Full Volume Encryption.

The data structures of BDE on Windows Vista and 7 differ.

A Windows Vista BDE volume starts with:

```
eb 52 90 2d 46 56 45 26 46 53 2d
```

A Windows 7 BDE volume starts with:

```
eb 58 90 2d 46 56 45 26 46 53 2d
```

BDE is largely a stand-alone but has some integration with NTFS.

TODO: link to BDE format documentation

### Volume Shadow Snapshots (VSS)

Volume Shadow Snapshots (VSS) uses the GUID
3808876b-c176-4e48-b7ae-04046e6cc752 (stored in little-endian) to identify its
data.

VSS is largely a stand-alone but has some integration with NTFS.

TODO: link to VSS format documentation

### <a name="media_descriptor"></a>Media descriptor

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 1 bit | | Sides, where single-sided (0) and double-sided (1)
| 0.1 | 1 bit | | Track size, where 9 sectors per track (0) and 8 sectors per track (1)
| 0.2 | 1 bit | | Density, where 80 tracks (0) and 40 tracks (1)
| 0.3 | 1 bit | | Type, where Fixed disc (0) and Removable disc (1)
| 0.4 | 4 bits | | Always set to 1

## The boot loader

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 512 | | | Windows NT (boot) loader (NTLDR/BOOTMGR)

## The Master File Table (MFT)

The MFT consist of an array of MFT entries. The offset of the MFT table can be
found in the volume header and the size of the MFT is defined by the MFT entry
of the $MFT metadata file.

> Note that the MFT can consists of multiple data ranges, defined by the data
> runs in the $MFT metadata file.

### MFT entry

Although the size of a MFT entry is defined in the volume header is commonly
1024 bytes in size and consists of:

* The MFT entry header
* [The fix-up values](#fix_up_values)
* An array of MFT attribute values
* Padding, which should contain 0-byte values

> Note that the MFT entry can be filled entirely with 0-byte values. Seen in
> Windows XP for MFT entry numbers 16 - 23.

#### MFT entry header

The MFT entry header (FILE_RECORD_SEGMENT_HEADER) is 42 or 48 bytes in size
and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *MULTI_SECTOR_HEADER*
| 0 | 4 | "BAAD", "FILE" | Signature
| 4 | 2 | | The fix-up values (or update sequence array) offset, which contain an offset relative from the start of the MFT entry.
| 6 | 2 | | The number of fix-up values (or update sequence array size)
| <td colspan="4"> &nbsp;
| 8 | 8 | | Metadata transaction journal sequence number, which contains a $LogFile Sequence Number (LSN)
| 16 | 2 | | Sequence (number)
| 18 | 2 | | Reference (link) count
| 20 | 2 | | Attributes offset (or first attribute offset), which contains an offset relative from the start of the MFT entry
| 22 | 2 | | [MFT entry flags](#mft_entry_flags)
| 24 | 4 | | Used size in bytes
| 28 | 4 | | MFT entry size in bytes
| 32 | 8 | | [Base record file reference](#file_reference)
| 40 | 2 | | First available attribute identifier
| <td colspan="4"> *If NTFS version is 3.0*
| 42 | 2 | | Unknown (wfixupPattern)
| 44 | 4 | | Unknown
| <td colspan="4"> *If NTFS version is 3.1*
| 42 | 2 | | Unknown (wfixupPattern)
| 44 | 4 | | MFT entry number

##### "BAAD" signature

According to [NTFS documentation](https://flatcap.github.io/linux-ntfs/ntfs/) if
during chkdsk, when a multi-sector item is found where the multi-sector header
does not match the values at the end of the sector, it marks the item as "BAAD"
and fill it with 0-byte values except for a fix-up value at the end of the first
sector of the item. The "BAAD" signature has been seen to be used on Windows NT4
and XP.

##### Sequence number

According to [FILE_RECORD_SEGMENT_HEADER structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/file-record-segment-header)
the sequence number is incremented each time that a file record segment is
freed; it is 0 if the segment is not used.

##### Base record file reference

The base record file reference is used to store additional attributes for
another MFT entry, e.g. for attribute lists.

#### <a name="mft_entry_flags"></a>MFT entry flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | FILE_RECORD_SEGMENT_IN_USE, MFT_RECORD_IN_USE | In use
| 0x0002 | FILE_FILE_NAME_INDEX_PRESENT, FILE_NAME_INDEX_PRESENT, MFT_RECORD_IS_DIRECTORY | Has file name (or $I30) index. When this flag is set the file entry represents a directory
| 0x0004 | MFT_RECORD_IN_EXTEND | Unknown. According to [ntfs_layout.h](https://ultradefrag.net/doc/man/ntfs/ntfs_layout.h.html) this is set for all system files present in the $Extend directory
| 0x0008 | MFT_RECORD_IS_VIEW_INDEX | Is index. When this flag is set the file entry represents an index. According to [ntfs_layout.h](https://ultradefrag.net/doc/man/ntfs/ntfs_layout.h.html) this is set for all indices other than $I30

#### <a name="fix_up_values"></a>The fix-up values

The fix-up values are of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Fix-up placeholder value
| 2 | 2 x number of fix-up values | | Fix-up (original) value array

On disk the last 2 bytes for each 512 byte block is replaced by the fix-up
placeholder value. The original value is stored in the corresponding fix-up
(original) value array entry.

> Note that there can be more fix-up values than the number of 512 byte blocks
> in the data.

According to [MULTI_SECTOR_HEADER structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/multi-sector-header)
the update sequence array must end before the last USHORT value in the first
sector. It also states that the update sequence array size value contains the
number of bytes, but based on analysis of data samples it seems to be more
likely to the number of words.

In NT4 (version 1.2) the MFT entry is 42 bytes in size and the fix-up values
are stored at offset 42. This is likely where the name wfixupPattern originates
from.

TODO: provide examples on applying the fix-up values.

### <a name="file_reference"></a>The file reference

The file reference (FILE_REFERENCE or MFT_SEGMENT_REFERENCE) is 8 bytes in size
and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 6 | | MFT entry number
| 6 | 2 | | Sequence number

> Note that the index value in the MFT entry is 32-bit in size.

### MFT attribute

The MFT attribute consist of:

* the attribute header
* the attribute resident or non-resident data
* the [attribute name](#attribute_name)
* Unknown data, likely alignment padding (4-byte alignment)
* resident attribute data or non-resident attribute data runs
* alignment padding (8-byte alignment), can contain remnant data

#### MFT attribute header

The MFT attribute header (ATTRIBUTE_RECORD_HEADER) is 16 bytes in size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | [Attribute type](#attribute_types) (or type code)
| 4 | 4 | | Attribute size (or record length), which includes the 8 bytes of the attribute type and size
| 8 | 1 | | Non-resident flag (or form code), where RESIDENT_FORM (0) and NONRESIDENT_FORM (1)
| 9 | 1 | | Name size (or name length), which contains the number of characters without the end-of-string character
| 10 | 2 | | Name offset, which contains an offset relative from the start of the MFT entry
| 12 | 2 | | [Attribute data flags](#mft_attribute_data_flags)
| 14 | 2 | | Attribute identifier (or instance), which contains an unique identifier to distinguish between attributes that contain segmented data.

#### <a name="mft_attribute_data_flags"></a>MFT attribute data flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | | Is LZNT1 compressed
| | |
| 0x00ff | ATTRIBUTE_FLAG_COMPRESSION_MASK |
| | |
| 0x4000 | ATTRIBUTE_FLAG_ENCRYPTED | Is encrypted
| 0x8000 | ATTRIBUTE_FLAG_SPARSE | Is sparse

TODO: determine the meaning of compression flag in the context of resident
$INDEX_ROOT. Do the data flags have a different meaning for different
attributes?

#### Resident MFT attribute

The resident MFT attribute data is present when the non-resident flag is not
set (0). The resident data is 8 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Data size (or value length)
| 4 | 2 | | Data offset (or value size), which contains an offset relative from the start of the MFT attribute
| 6 | 1 | | Indexed flag
| 7 | 1 | 0x00 | Unknown (Padding)

TODO: determine the meaning of indexed flag bits, other than the LSB

#### Non-resident MFT attribute

The non-resident MFT attribute data is present when the non-resident flag is
set (1). The non-resident data is 48 or 56 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | First (or lowest) Virtual Cluster Number (VCN) of the data
| 8 | 8 | | Last (or highest) Virtual Cluster Number (VCN) of the data
| 16 | 2 | | Data runs offset (or mappings pairs offset), which contains an offset relative from the start of the MFT attribute
| 18 | 2 | | Compression unit size, which contains the compression unit size as `2^(n)` number of cluster blocks.
| 20 | 4 | | Unknown (Padding)
| 24 | 8 | | Allocated data size (or allocated length), which contains the allocated data size in number of bytes. This value is not valid if the first VCN is nonzero.
| 32 | 8 | | Data size (or file size), which contains the data size in number of bytes. This value is not valid if the first VCN is nonzero.
| 40 | 8 | | Valid data size (or valid data length), which contains the valid data size in number of bytes. This value is not valid if the first VCN is nonzero.
| <td colspan="4"> *If compression unit size > 0*
| 48 | 8 | | Compressed data size.

> Note that the total size of the data runs should be larger or equal to the
> data size.

> Note that Windows will fill data ranges beyond the valid data size with 0-byte
> values. The data size remains unchanged. This applies to compressed and
> uncompressed data. If the first VCN is zero a valid data size of 0 represents
> a file entirely filled with 0-byte values.

TODO: determine the meaning of a VCN of -1

For more information about compressed MFT attributes see [compression](#compression).

#### <a name="attribute_name"></a>Attribute name

The attribute name is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Name, which contains an UCS-2 little-endian string without end-of-string character

#### Data runs

The data runs are stored in a variable size (data) runlist. This runlist
consists of runlist elements.

A runlist element is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0  | 4 bits | | Number of cluster blocks value size, which contains the number of bytes used to store the data run size
| 0.4 | 4 bits | | Cluster block number value size, which contains the number of bytes used to store the data run size
| 1 | Size value size | | Data run number of cluster blocks, which contains the number of cluster blocks
| ... | Cluster block number value size | | Data run cluster block number

The data run cluster block number is a singed value, where the MSB is the
singed bit, e.g. if the data run cluster block contains "dbc8" it corresponds
to the 64-bit value 0xffffffffffffdbc8.

The first data run offset contains the absolute cluster block number where
successive data run offsets are relative to the last data run offset.

> Note that the cluster block number byte size is the first nibble when reading
> the byte stream, but here it is represented as the upper nibble of the first
> byte.

The last runlist element is (0, 0), which is stored as a 0-byte value.

According to [NTFS documentation](https://flatcap.github.io/linux-ntfs/ntfs/)
the size of the runlist is rounded up to the next multitude of 4 bytes, but
based on analysis of data samples it seems that the size of the trailing data
can be even larger than 3 and are not always 0-byte values.

TODO: provide examples of data runs

##### Sparse data runs

The MFT attribute data flag (ATTRIBUTE_FLAG_SPARSE) indicates if the data
stream is sparse or not, where the runlist can contain both sparse and
non-sparse data runs.

A sparse data run has a cluster block number value size of 0, representing
there is no offset (cluster block number). A sparse data run is filled with
0-byte values.

Compressed data streams also define sparse data runs without setting the
ATTRIBUTE_FLAG_SPARSE flag.

> Note that $BadClus:$Bad also defines a data run with a cluster block number
> value size of 0, without setting the ATTRIBUTE_FLAG_SPARSE flag.

##### Compresssed data runs

The MFT attribute data flags (0x00ff) indicate if the data stream is compressed
or not.

> Note that Windows supports compressed data runs for NTFS file systems with a
> cluster block size of 4096 bytes or less.

> Note that Windows 10 supports Windows Overlay Filter (WOF) compressed data,
> which stores the LZXPRESS Huffman or LZX compressed data in alternate data
> stream named WofCompressedData and links it to the default data stream using
> a reparse point.

The data is stored in compression unit blocks. A compression unit typically
consists of 16 cluster blocks. However the actual value is stored in the
non-resident MFT attribute.

Also see [compression](#compression).

## The attributes

### <a name="attribute_types"></a>Known attribute types

The attribute types are stored in the [$AttrDef metadata file](#attribute_definitions).

| Value | Identifier | Description
| --- | --- | ---
| 0x00000000 | | Unused
| 0x00000010 | $STANDARD_INFORMATION | Standard information
| 0x00000020 | $ATTRIBUTE_LIST | Attributes list
| 0x00000030 | $FILE_NAME | The file or directory name
| <td colspan="3"> *Used in NTFS version 1.2 and earlier*
| 0x00000040 | $VOLUME_VERSION | Volume version
| <td colspan="3"> *Used in NTFS version 3.0 and later*
| 0x00000040 | $OBJECT_ID | Object identifier
| <td colspan="3"> *Common*
| 0x00000050 | $SECURITY_DESCRIPTOR | Security descriptor
| 0x00000060 | $VOLUME_NAME | Volume label
| 0x00000070 | $VOLUME_INFORMATION | Volume information
| 0x00000080 | $DATA | Data stream
| 0x00000090 | $INDEX_ROOT | Index root
| 0x000000a0 | $INDEX_ALLOCATION | Index allocation
| 0x000000b0 | $BITMAP | Bitmap
| <td colspan="3"> *Used in NTFS version 1.2 and earlier*
| 0x000000c0 | $SYMBOLIC_LINK | Symbolic link
| <td colspan="3"> *Used in NTFS version 3.0 and later*
| 0x000000c0 | $REPARSE_POINT | Reparse point
| <td colspan="3"> *Common*
| 0x000000d0 | $EA_INFORMATION | (HPFS) extended attribute information
| 0x000000e0 | $EA | (HPFS) extended attribute
| <td colspan="3"> *Used in NTFS version 1.2 and earlier*
| 0x000000f0 | $PROPERTY_SET | Property set
| <td colspan="3"> *Used in NTFS version 3.0 and later*
| 0x00000100 | $LOGGED_UTILITY_STREAM | Logged utility stream
| <td colspan="3"> *Common*
| | |
| 0x00001000 | | First user defined attribute
| | |
| 0xffffffff | | End of attributes marker

### <a name="attribute_chains"></a>Attribute chains

Multiple attributes can be chained to make up a single attribute data stream,
e.g. the attributes:

1. $INDEX_ALLOCATION ($I30) VCN: 0
1. $INDEX_ALLOCATION ($I30) VCN: 596

The first attribute will contain the size of the data defined by all the
attributes and successive attributes should have a size of 0.

It is assumed that the attributes in a chain must be continuous and defined
in-order.

### The standard information attribute

The standard information attribute ($STANDARD_INFORMATION) contains the basic
file entry metadata. It is stored as a resident MFT attribute.

The standard information data (STANDARD_INFORMATION) is either 48 or 72 bytes
in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Creation date and time, which contains a FILETIME
| 8 | 8 | | Last modification (or last written) dat and time, which contains a FILETIME
| 16 | 8 | | MFT entry last modification date and time, which contains a FILETIME
| 24 | 8 | | Last access date and time, which contains a FILETIME
| 32 | 4 | | [File attribute flags](#file_attribute_flags)
| 36 | 4 | | Unknown (Maximum number of versions)
| 40 | 4 | | Unknown (Version number)
| 44 | 4 | | Unknown (Class identifier)
| <td colspan="4"> *If NTFS version 3.0 or later*
| 48 | 4 | | Owner identifier
| 52 | 4 | | Security descriptor identifier, which contains the entry number in the security ID index ($Secure:$SII). Also see [Access Control](#access_control)
| 56 | 8 | | Quota charged
| 64 | 8 | | Update Sequence Number (USN)

> Note that MFT entries have been observed without a $STANDARD_INFORMATION
> attribute, but with other attributes such as $FILE_NAME and an $I30 index.

### The attribute list attribute

The attribute list attribute ($ATTRIBUTE_LIST) is a list of attributes in an
MFT entry. The attributes stored in the list are placeholders for other
attributes. Some of these attributes could not be stored in the MFT entry due
to space limitations. The attribute list attribute can be stored as either a
resident (for a small amount of data) and non-resident MFT attribute.

The attribute list contains an attribute list entry for every cluster block of
attribute data.

> Note that MFT entry 0 also can contain an attribute list and allows to store
> listed attributes beyond the first data run.

#### The attribute list entry

The attribute list entry (ATTRIBUTE_LIST_ENTRY) is of variable size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | [Attribute type](#attribute_types) (or type code)
| 4 | 2 | | Size (or record length), which includes the 6 bytes of the attribute type and size
| 6 | 1 | | Name size (or name length), which contains the number of characters without the end-of-string character
| 7 | 1 | | Name offset, which contains an offset relative from the start of the attribute list entry
| 8 | 8 | | Data first (or lowest) VCN
| 16 | 8 | | [File reference](#file_reference) (or segment reference), which contains a reference to the MFT entry that contains (part of) the attribute data
| 24 | 2 | | Attribute identifier (or instance), which contains an unique identifier to distinguish between attributes that contain segmented data.
| 26 | ... | | Name, which contains an UCS-2 little-endian string without end-of-string character
| ... | ... | | alignment padding (8-byte alignment), can contain remnant data

### <a name="file_name_attribute"></a>The file name attribute

The file name attribute ($FILE_NAME) contains the basic file system information,
like the parent file entry, various date and time values and name. It is stored
as a resident MFT attribute.

The file name data (FILE_NAME) is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | [Parent file reference](#file_reference)
| 8 | 8 | | Creation date and time, which contains a FILETIME
| 16 | 8 | | Last modification (or last written) date and time, which contains a FILETIME
| 24 | 8 | | MFT entry last modification date and time, which contains a FILETIME
| 32 | 8 | | Last access date and time, which contains a FILETIME
| 40 | 8 | | Allocated (or reserved) file size
| 48 | 8 | | Data size
| 56 | 4 | | [File attribute flags](#file_attribute_flags)
| <td colspan="4"> *If FILE_ATTRIBUTE_REPARSE_POINT is set*
| 60 | 4 | | [Reparse point tag](#reparse_point_tag)
| <td colspan="4"> *If FILE_ATTRIBUTE_REPARSE_POINT is not set*
| 60 | 4 | | Unknown (extended attribute data size)
| <td colspan="4"> *Common*
| 64 | 1 | | Name string size, which contains the number of characters without the end-of-string character
| 65 | 1 | | Namespace of the name string
| 66 | ... | | Name, which contains an UCS-2 little-endian string without end-of-string character

TODO: determine if the allocated file size and file size values contain accurate values when the file name data is stored in a MFT attribute.

An MFT attribute can contain multiple file name attributes, e.g. for a separate
(long) name and short name.

In several cases on a Vista NTFS volume the MFT entry contained both a DOS &
Windows and POSIX name space $FILE_NAME attribute. However the directory entry
index ($I30) of the parent directory only contained the DOS & Windows name.

In case of a hard link the MFT entry will contain additional file name
attributes with the parent file reference of each hard link.

#### Namespace

| Value | Identifier | Description
| --- | --- | ---
| 0 | POSIX | Case sensitive character set that consists of all Unicode characters except for: "\0" (zero character), "/" (forward slash). The ":" (colon) is valid for NTFS but not for Windows.
| 1 | FILE_NAME_NTFS, WINDOWS | Case insensitive sub set of the POSIX character set that consists of all Unicode characters except for: `" * / : < > ? \ \| +`. Note that names cannot end with a "." (dot) or " " (space).
| 2 | FILE_NAME_DOS, DOS | Case insensitive sub set of the WINDOWS character set that consists of all upper case ASCII characters except for: `" * + , / : ; < = > ? \`. Note that the name must follow the 8.3 format.
| 3 | DOS_WINDOWS | Both the DOS and WINDOWS names are identical, which is the same as the DOS character set, with the exception that lower case is used as well.

> Note that the Windows API function CreateFile allows to create case sensitive
> file names when the flag FILE_FLAG_POSIX_SEMANTICS is set.

#### Long to short name conversion

A short name can be determined from a long name with the following approach. In
the long name:

* ignore Unicode characters beyond the first 8-bit (extended ASCII)
* ignore control characters and spaces (character < 0x20)
* ignore non-allowed characters `" * + , / : ; < = > ? \`
* ignore dots except the last one, which is used for the extension
* make all letters upper case

Additional observations:

* `[` or `]` are replaced by an underscore (`_`)

Make the name unique:

1. use the characters 1 to 6 add ~1 and if the long name has an extension add the a dot and its first 3 letters, e.g. "Program Files" becomes "PROGRA~1" or " ~PLAYMOVIE.REG" becomes "\~PLAYM~1.REG"
1. if the name already exists try ~2 up to ~9, e.g. "Program Data", in the same directory as "Program Files", becomes "PROGRA~2"
1. if the name already exists use a 16-bit hexadecimal value for characters 3 to 6 with ~1, e.g. "x86_microsoft-windows-r..ry-editor.resources_31bf3856ad364e35_6.0.6000.16386_en-us_f89a7b0005d42fd4" in a directory with a lot of filenames starting with "x86_microsoft", becomes "X8FCA6~1.163"

TODO: determine if the behavior is dependent on a setting that can be changed with fsutil

### The volume version attribute

The volume version attribute ($VOLUME_VERSION) contains volume version.

TODO: complete section. Need a pre NTFS version 3.0 volume with this attribute.
$AttrDef indicates the attribute to be 8 bytes in size.

### The object identifier attribute

The object identifier attribute ($OBJECT_ID) contains distributed link tracker
properties. It is stored as a resident MFT attribute.

The object identifier attribute data is either 16 or 64 bytes in size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Droid file identifier, which contains a GUID
| 16 | 16 | | Birth droid volume identifier, which contains a GUID
| 32 | 16 | | Birth droid file identifier, which contains a GUID
| 48 | 16 | | Birth droid domain identifier, which contains a GUID

Droid in this context refers to CDomainRelativeObjId.

### The security descriptor attribute

TODO: determine if this override any value in $Secure:$SDS?

The security descriptor attribute ($SECURITY_DESCRIPTOR) contains a Windows NT
security descriptor. It can be stored as either a resident (for a small amount
of data) and non-resident MFT attribute.

TODO: link to security descriptor format documentation

### The volume name attribute

The volume name attribute ($VOLUME_NAME) contains the volume label. It is
stored as a resident MFT attribute.

The volume name attribute data is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Volume label, which contains an UCS-2 little-endian string without end-of-string character

The volume name attribute is used in the $Volume metadata file MFT entry.

### The volume information attribute

The volume information attribute ($VOLUME_INFORMATION) contains information
about the volume. It is stored as a resident MFT attribute.

The volume information attribute data is 12 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Unknown
| 8 | 1 | | Major format version
| 9 | 1 | | Minor format version
| 10 | 2 | | [Volume flags](#volume_flags)

The volume information attribute is used in the $Volume metadata file MFT entry.

#### <a name="volume_flags"></a>Volume flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | VOLUME_IS_DIRTY | Is dirty
| 0x0002 | VOLUME_RESIZE_LOG_FILE | Re-size journal ($LogFile)
| 0x0004 | VOLUME_UPGRADE_ON_MOUNT | Upgrade on next mount
| 0x0008 | VOLUME_MOUNTED_ON_NT4 | Mounted on Windows NT 4
| 0x0010 | VOLUME_DELETE_USN_UNDERWAY | Delete USN in progress
| 0x0020 | VOLUME_REPAIR_OBJECT_ID | Repair object identifiers
| | |
| 0x0080 | | Unknown
| | |
| 0x4000 | VOLUME_CHKDSK_UNDERWAY | chkdsk in progress
| 0x8000 | VOLUME_MODIFIED_BY_CHKDSK | Modified by chkdsk

### The data stream attribute

The data stream attribute ($DATA) contains the file data. It can be stored as
either a resident (for a small amount of data) and non-resident MFT attribute.

Multiple data attributes for the same data stream can be used in the attribute
list to define different parts of the data stream data. The first data stream
attribute will contain the size of the entire data stream data. Other data
stream attributes should have a size of 0. Also see [attribute chains](#attribute_chains).

### The index root attribute

The index root attribute ($INDEX_ROOT) contains the root of the index tree. It
is stored as a resident MFT attribute.

Also see [the index](#index) and [the index root](#index_root).

### The index allocation attribute

The index allocation attribute ($INDEX_ALLOCATION) contains an array of index
entries. It is stored as a non-resident MFT attribute.

The index allocation attribute itself does not define which attribute type it
contains in the index value data. For this information it needs the
corresponding index root attribute.

Multiple index allocation attributes for the same index can be used in the
attribute list to define different parts of the index allocation data. The
first index allocation attribute will contain the size of the entire index
allocation data. Other index allocation attributes should have a size of 0.
Also see [attribute chains](#attribute_chains).

Also see [the index](#index).

### The bitmap attribute

The bitmap attribute ($BITMAP) contains the allocation bitmap. It can be stored
as either a resident (for a small amount of data) and non-resident MFT
attribute.

It is used to maintain information about which entry is used and which is not.
Every bit in the bitmap represents an entry. The index is stored byte-wise with
the LSB of the byte corresponds to the first allocation element. The allocation
element can represent different things:

* an MFT entry in the MFT (nameless) bitmap;
* an index entry in an index ($I##).

The allocation element is allocated if the corresponding bit contains 1 or
unallocated if 0.

### The symbolic link attribute

The symbolic link attribute ($SYMBOLIC_LINK) contains a symbolic link.

TODO: complete section. Need a pre NTFS version 3.0 volume with this attribute.
$AttrDef indicates the attribute is of variable size.

### The reparse point attribute

The reparse point attribute ($REPARSE_POINT) contains information about a file
system-level link. It is stored as a resident MFT attribute.

Als see [the reparse point](#reparse_point).

### The (HPFS) extended attribute information

The (HPFS) extended attribute information ($EA_INFORMATION) contains
information about the extended attribute ($EA).

The extended attribute information data is 8 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Size of an extended attribute entry
| 2 | 2 | | Number of extended attributes which have the NEED_EA flag set
| 4 | 4 | | Size of the extended attribute ($EA) data

### The (HPFS) extended attribute

The (HPFS) extended attribute ($EA) contains the extended attribute data.

The extended attribute data is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Offset to next extended attribute entry, where the offset is relative from the start of the extended attribute data
| 4 | 1 | | [Extended attribute flags](#extended_attribute_flags)
| 5 | 1 | | Number of characters of the extended attribute name
| 6 | 2 | | Value data size
| 8 | ... | | The extended attribute name, which contains an ASCII string
| ... | ... | | Value data
| ... | ... | | Unknown

TODO: determine if the name is 2-byte aligned

#### <a name="extended_attribute_flags"></a>Extended attribute flags

| Value | Identifier | Description
| --- | --- | ---
| 0x80 | NEED_EA | Unknown (Need EA) flag

TODO: determine what the NEED_EA flag is used for

#### UNITATTR extended attribute value data

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Unknown (equivalent of st_mode?)

### The property set attribute

The property set attribute ($PROPERTY_SET) contains a property set.

TODO: complete section. Need a pre NTFS version 3.0 volume with this attribute.
$AttrDef does not seem to always define this attribute.

### The logged utility stream attribute

TODO: complete section

| Value | Identifier | Description
| --- | --- | ---
| $EFS | | Encrypted NTFS (EFS)
| $TXF_DATA | | Transactional NTFS (TxF)

## <a name="attribute_types"></a>The attribute types

The attribute types are stored in the `$AttrDef` metadata file.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 128 | | Attribute which contains an UCS-2 little-endian string with end-of-string character. Unused bytes are filled with 0-byte values.
| 128 | 4 | | [Attribute type](#attribute_types) (or type code)
| 132 | 8 | | Unknown
| 140 | 4 | | Unknown (flags?)
| 144 | 8 | | Unknown (minimum attribute size?)
| 152 | 8 | | Unknown (maximum attribute size?)

## <a name="index"></a>The index

The index structures are used for various purposes one of which are the
directory entries.

The root of the index is stored in index root. The index root attribute defines
which type of attribute is stored in the index and the root index node.

If the index is too large part of the index is stored in an index allocation
attribute with the same attribute name. The index allocation attribute defines
a data stream which contains index entries. Each index entry contains an index
node.

An index consists of a tree, where both the branch and index leaf nodes contain
the actual data. E.g. in case of a directory entries index, any node that
contains index value data make up for the directory entries.

The index value data in a branch node signifies the upper bound of the values
in the that specific branch. E.g. if directory entries index branch node
contains the name "textfile.txt" all names in that index branch are smaller
than "textfile.txt".

> Note the actual sorting order is dependent on the collation type defined in
> the index root attribute.

The index allocation attribute is accompanied by a bitmap attribute with the
corresponding attribute name. The bitmap attribute defines the allocation of
virtual cluster blocks within the index allocation attribute data stream.

> Note that the index allocation attribute can be present even though it is not
> used.

### Common used indexes

Indexes commonly used by NTFS are:

| Value | Identifier | Description
| --- | --- | ---
| $I30 | | Directory entries (used by directories)
| $SDH | | Security descriptor hashes (used by $Secure)
| $SII | | Security descriptor identifiers (used by $Secure)
| $O | | Object identifiers (used by $ObjId)
| $O | | Owner identifiers (used by $Quota)
| $Q | | Quotas (used by $Quota)
| $R | | Reparse points (used by $Reparse)

### <a name="index_root"></a>The index root

The index root consists of:

* index root header
* index node header
* an array of index values

#### The index root header

The index root header is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Attribute type, which contains the type of the indexed attribute or 0 if none
| 4 | 4 | | [Collation type](#collation_type), which contains a value to indicate the ordering of the index entries
| 8 | 4 | | Index entry size
| 12 | 4 | | Index entry number of cluster blocks

> Note that for NTFS version 1.2 the index entry size does not have to match
> the index entry size in the volume header. The correct size seems to be the
> value in the index root header.

#### <a name="collation_type"></a>Collation type

| Value | Identifier | Description
| --- | --- | ---
| 0x00000000 | COLLATION_BINARY | Binary, where the first byte is most significant
| 0x00000001 | COLLATION_FILENAME | UCS-2 strings case-insensitive, where the case folding is stored in $UpCase
| 0x00000002 | COLLATION_UNICODE_STRING | UCS-2 strings case-sensitive, where upper case letters should come first
| | |
| 0x00000010 | COLLATION_NTOFS_ULONG | Unsigned 32-bit little-endian integer
| 0x00000011 | COLLATION_NTOFS_SID | NT security identifier (SID)
| 0x00000012 | COLLATION_NTOFS_SECURITY_HASH | Security hash first, then NT security identifier
| 0x00000013 | COLLATION_NTOFS_ULONGS | An array of unsigned 32-bit little-endian integer values

### The index entry

The index entry consists of:

* the index entry header
* the index node header
* [The fix-up values](#fix_up_values)
* alignment padding (8-byte alignment), contains zero-bytes
* an array of index values

#### The index entry header

The index entry header is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "INDX" | Signature
| 4 | 2 | | The fix-up values offset, which contains an offset relative from the start of the index entry header.
| 6 | 2 | | The number of fix-up values
| 8 | 8 | | Metadata transaction journal sequence number, which contains a $LogFile Sequence Number (LSN)
| 16 | 8 | | Virtual Cluster Number (VCN) of the index entry

> Note that there can be more fix-up value than supported by the index entry
> data size.

### The index node header

The index node header is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Index values offset, where the offset is relative from the start of the index node header
| 4 | 4 | | Index node size, where the value includes the size of the index node header
| 8 | 4 | | Allocated index node size, where the value includes the size of the index node header
| 12 | 4 | | [Index node flags](#index_node_flags)

In an index entry (index allocation attribute) the index node size includes the
size of the fix-up values and the alignment padding following it.

The remainder of the index node contains remnant data and/or zero-byte values.

#### <a name="index_node_flags"></a>The index node flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | | Is branch node, which is used to indicate if the node is a branch node that has sub nodes

### The index value

The index value is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | [File reference](#file_reference)
| 8 | 2 | | Size, which includes the 10 bytes of the file reference and size
| 10 | 2 | | Key data size
| 12 | 4 | | Index value flags
| <td colspan="4"> *If index key data size > 0*
| 16 | ... | | Key data
| ... | ... | | Data
| <td colspan="4"> *If index value flag 0x00000001 (is branch node) is set*
| ... | 8 | | Sub node Virtual Cluster Number (VCN)

The index values are stored 8 byte aligned.

> Note that some other sources define the index value flags as a 16-bit value
> followed by 2 bytes of padding.

#### The index value flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | | Has sub node, when set the index value contains a sub node Virtual Cluster Number (VCN)
| 0x00000002 | | Is last, when set the index value is the last in the index values array

### Index key and value data

#### Directory entry index value

The MFT attribute name of the directory entry index is: $I30.

The directory entry index value contains a [file name attribute](#file_name_attribute)
in the index key data.

> Note that the index value data can contain remnant data.

The short and long names of the same file have a separate index values. The
short name uses the DOS name space and the long name the WINDOWS name space.
Index values with a single name use either the POSIX or DOS_WINDOWS name space.

A hard link to a file in the same directory has separate index values.

#### Security descriptor hash index value

The MFT attribute name of the security descriptor hash index is: $SDH.
It appears to only to be used by the $Secure metadata file.

Also see [the security descriptor hash index value](#security_descriptor_hash_index_value).

#### Security descriptor identifier index value

The MFT attribute name of the security descriptor identifier index is: $SII.
It appears to only to be used by the $Secure metadata file.

Also see [the security descriptor identifier index value](#security_descriptor_identifier_index_value).

## <a name="compression"></a>Compression

Typically NTFS compression groups 16 cluster blocks together. This group of 16
cluster blocks also named a compression unit, which is either "compressed" or
uncompressed data.

The term compressed is quoted here because the group of cluster blocks can also
contain uncompressed data. A group of cluster blocks is "compressed" when it is
compressed size is smaller than its uncompressed data size. Within a group of
cluster blocks each of the 16 blocks is "compressed" individually see
[block based storage](#compression_block_based_storage).

The compression unit size is stored in the non-resident MFT attribute. The
maximum uncompressed data size is always the cluster size (in most case
4096).

The data runs in the $DATA attribute define cluster block ranges, e.g.

```
21 02 35 52
```

This data run defines 2 data blocks starting at block number 21045 followed by
14 sparse blocks. The total number of blocks is 16 which is the size of the
compression unit. The data is stored compressed in the first 2 blocks and the
14 sparse blocks are only there to make sure the data runs add up to the
compression unit size. They do not define actual sparse data.

Another example:

```
21 40 37 52
```

This data run defines 64 data blocks starting at block number 21047. Since
this data run is larger than the compression unit size the data is stored
uncompressed.

If the data run was e.g. 60 data blocks followed by 4 sparse blocks the first 3
compression units (blocks 1 to 48) would be uncompressed and the last
compression unit (blocks 49 to 64) would be compressed.

Also "sparse data" and "sparse compression unit" data runs can be mixed. If in
the previous example the 60 data blocks would be followed by 20 sparse blocks
the last compression unit (blocks 65 to 80) would be sparse.

A compression unit can consists of multiple compressed data runs, e.g. 1 data
block followed by 4 data blocks followed by 11 sparse blocks. Data runs have
been observed where the last data run size does not align with the compression
unit size.

The sparse blocks data run can be stored in a subsequent attribute in an
attribute chain and can be stored in multiple data runs.

### <a name="compression_block_based_storage"></a>Block based storage

NTFS compression stores the "compressed" data in blocks. Each block has a 2
byte block header.

The block is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Block size
| 2 | compressed data size | | Uncompressed or LZNT1 compressed data

The upper 4 bits of the block size are used as flags:

| Bit(s) | Description
| --- | ---
| 0 - 11 | Compressed data size
| 12 - 14 | Unknown
| 15 | Data is compressed

TODO: link to LZNT1 documentation

### Windows Overlay Filter (WOF) compressed data

A MFT entry that contains Windows Overlay Filter (WOF) compressed data has the
following attributes:

* reparse point attribute with tag 0x80000017, which defines the compression method
* a nameless data attribute that is sparse and contains the uncompressed data size
* a data attribute named WofCompressedData that contains LZXPRESS Huffman or LZX compressed data

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *Chunk offset table*
| 0 | ... | | Array of 32-bit of 64-bit compressed data chunk offsets, where the offset is relative from the start of the data chunks
| <td colspan="4"> *Data chunks*
| ... | ... | | One or more compressed or uncompressed data chunks

> Note that if the chunk size equals the size of the uncompressed data the chunk
> is stored (as-is) uncompressed.

The size of the chunk offset table is:

```
number of chunk offsets = uncompressed size / compression unit size
```

The offset of the first compressed data chunk is at the end of the chunk offset
table and is not stored in the chunk offset table.

Also see [Windows Overlay Filter (WOF) compression method](#wof_compression_method).

## <a name="reparse_point"></a>The reparse point

The reparse point is used to create file system-level links. Reparse data is
stored in the reparse point attribute. The reparse point data
(REPARSE_DATA_BUFFER) is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Reparse point tag
| 4 | 2 | | Reparse data size
| 6 | 2 | 0 | Unknown (Reserved)
| 8 | ... | | Reparse data

TODO: determine if non-native (Microsoft) reparse points are stored with their GUID

### <a name="reparse_point_tag"></a>The reparse point tag

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0  | 16 bits | | Type
| 2.0  | 12 bits | | Unknown (Reserved)
| 3.4 | 4 bits | | Flags

#### Reparse point tag flags

| Value | Identifier | Description
| --- | --- | ---
| 0x1 | | Unknown (Reserved)
| 0x2 | | Is alias (Name surrogate bit), when this bit is set, the file or directory represents another named entity in the system.
| 0x4 | | Is high-latency media (Reserved)
| 0x8 | | Is native (Microsoft-bit)

#### Known reparse point tags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000000 | IO_REPARSE_TAG_RESERVED_ZERO | Unknown (Reserved)
| 0x00000001 | IO_REPARSE_TAG_RESERVED_ONE | Unknown (Reserved)
| 0x00000002 | IO_REPARSE_TAG_RESERVED_TWO | Unknown (Reserved)
| | |
| 0x80000005 | IO_REPARSE_TAG_DRIVE_EXTENDER | Used by Home server drive extender
| 0x80000006 | IO_REPARSE_TAG_HSM2 | Used by Hierarchical Storage Manager Product
| 0x80000007 | IO_REPARSE_TAG_SIS | Used by single-instance storage (SIS) filter driver
| 0x80000008 | IO_REPARSE_TAG_WIM | Used by the WIM Mount filter
| 0x80000009 | IO_REPARSE_TAG_CSV | Used by Clustered Shared Volumes (CSV) version 1
| 0x8000000a | IO_REPARSE_TAG_DFS | Used by the Distributed File System (DFS)
| 0x8000000b | IO_REPARSE_TAG_FILTER_MANAGER | Used by filter manager test harness
| | |
| 0x80000012 | IO_REPARSE_TAG_DFSR | Used by the Distributed File System (DFS)
| 0x80000013 | IO_REPARSE_TAG_DEDUP | Used by the Data Deduplication (Dedup)
| 0x80000014 | IO_REPARSE_TAG_NFS | Used by the Network File System (NFS)
| 0x80000015 | IO_REPARSE_TAG_FILE_PLACEHOLDER | Used by Windows Shell for placeholder files
| 0x80000016 | IO_REPARSE_TAG_DFM | Used by Dynamic File filter
| 0x80000017 | IO_REPARSE_TAG_WOF | Used by [Windows Overlay Filter (WOF)](#wof_reparse_data), for either WIMBoot or compression
| 0x80000018 | IO_REPARSE_TAG_WCI | Used by [Windows Container Isolation (WCI)](#wci_reparse_data)
| | |
| 0x8000001b | IO_REPARSE_TAG_APPEXECLINK | Used by Universal Windows Platform (UWP) packages to encode information that allows the application to be launched by CreateProcess
| | |
| 0x8000001e | IO_REPARSE_TAG_STORAGE_SYNC | Used by the Azure File Sync (AFS) filter
| | |
| 0x80000020 | IO_REPARSE_TAG_UNHANDLED | Used by Windows Container Isolation (WCI)
| 0x80000021 | IO_REPARSE_TAG_ONEDRIVE | Unknown (Not used)
| | |
| 0x80000023 | IO_REPARSE_TAG_AF_UNIX | Used by the Windows Subsystem for Linux (WSL) to represent a UNIX domain socket
| 0x80000024 | IO_REPARSE_TAG_LX_FIFO | Used by the Windows Subsystem for Linux (WSL) to represent a UNIX FIFO (named pipe)
| 0x80000025 | IO_REPARSE_TAG_LX_CHR | Used by the Windows Subsystem for Linux (WSL) to represent a UNIX character special file
| 0x80000036 | IO_REPARSE_TAG_LX_BLK | Used by the Windows Subsystem for Linux (WSL) to represent a UNIX block special file
| | |
| 0x9000001c | IO_REPARSE_TAG_PROJFS | Used by the Windows Projected File System filter, for files managed by a user mode provider such as VFS for Git
| | |
| 0x90001018 | IO_REPARSE_TAG_WCI_1 | Used by Windows Container Isolation (WCI)
| | |
| 0x9000101a | IO_REPARSE_TAG_CLOUD_1 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000201a | IO_REPARSE_TAG_CLOUD_2 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000301a | IO_REPARSE_TAG_CLOUD_3 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000401a | IO_REPARSE_TAG_CLOUD_4 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000501a | IO_REPARSE_TAG_CLOUD_5 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000601a | IO_REPARSE_TAG_CLOUD_6 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000701a | IO_REPARSE_TAG_CLOUD_7 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000801a | IO_REPARSE_TAG_CLOUD_8 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000901a | IO_REPARSE_TAG_CLOUD_9 | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000a01a | IO_REPARSE_TAG_CLOUD_A | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000b01a | IO_REPARSE_TAG_CLOUD_B | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000c01a | IO_REPARSE_TAG_CLOUD_C | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000d01a | IO_REPARSE_TAG_CLOUD_D | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000e01a | IO_REPARSE_TAG_CLOUD_E | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0x9000f01a | IO_REPARSE_TAG_CLOUD_F | Used by the Cloud Files filter, for files managed by a sync engine such as OneDrive
| | |
| 0xa0000003 | IO_REPARSE_TAG_MOUNT_POINT | [Junction](#junction_reparse_data) (or mount point)
| | |
| 0xa000000c | IO_REPARSE_TAG_SYMLINK | [Symbolic link](#symbolic_link_reparse_data)
| | |
| 0xa0000010 | IO_REPARSE_TAG_IIS_CACHE | Used by Microsoft Internet Information Services (IIS) caching
| | |
| 0xa0000019 | IO_REPARSE_TAG_GLOBAL_REPARSE | Used by NPFS to indicate a named pipe symbolic link from a server silo into the host silo
| 0xa000001a | IO_REPARSE_TAG_CLOUD | Used by the Cloud Files filter, for files managed by a sync engine such as Microsoft OneDrive
| | |
| 0xa000001d | IO_REPARSE_TAG_LX_SYMLINK | Used by the Windows Subsystem for Linux (WSL) to represent a UNIX symbolic link
| | |
| 0xa000001f | IO_REPARSE_TAG_WCI_TOMBSTONE | Used by Windows Container Isolation (WCI)
| | |
| 0xa0000022 | IO_REPARSE_TAG_PROJFS_TOMBSTONE | Used by the Windows Projected File System filter, for files managed by a user mode provider such as VFS for Git
| | |
| 0xa0000027 | IO_REPARSE_TAG_WCI_LINK | Used by Windows Container Isolation (WCI)
| | |
| 0xa0001027 | IO_REPARSE_TAG_WCI_LINK_1 | Used by Windows Container Isolation (WCI)
| | |
| 0xc0000004 | IO_REPARSE_TAG_HSM | Used by Hierarchical Storage Manager Product
| | |
| 0xc0000014 | IO_REPARSE_TAG_APPXSTRM | Unknown (Not used)

### <a name="junction_reparse_data"></a>Junction or mount point reparse data

A reparse point with tag IO_REPARSE_TAG_MOUNT_POINT (0xa0000003) contains
junction or mount point reparse data. The junction or mount point reparse data
is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Substitute name offset, where the offset is relative from the start of the reparse name data
| 2 | 2 | | Substitute name size in bytes, where the size of the end-of-string character is not included
| 4 | 2 | | Display name offset, where the offset is relative from the start of the reparse name data
| 6 | 2 | | Display name size in bytes, where the size of the end-of-string character is not included
| <td colspan="4"> *Reparse name data*
| 8 | ... | | Substitute name, which contains an UCS-2 little-endian string without end-of-string character
| ... | ... | | Display name, which contains an UCS-2 little-endian string without end-of-string character

> Note that it is currently unclear if the names contain an end-of-string
> character or if they are followed by alignment padding.

TODO: determine what character values like 0x0002 represent in the substitute name

```
00000010: 5c 00 3f 00 3f 00 02 00  43 00 3a 00 5c 00 55 00   \.?.?... C.:.\.U.
00000020: 73 00 65 00 72 00 73 00  5c 00 74 00 65 00 73 00   s.e.r.s. \.t.e.s.
00000030: 74 00 5c 00 44 00 6f 00  63 00 75 00 6d 00 65 00   t.\.D.o. c.u.m.e.
00000040: 6e 00 74 00 73 00 00 00                            n.t.s...
```

### <a name="symbolic_link_reparse_data"></a>Symbolic link reparse data

A reparse point with tag IO_REPARSE_TAG_SYMLINK (0xa000000c) contains symbolic
link reparse data. The symbolic link reparse data is of variable size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Substitute name offset, where the offset is relative from the start of the reparse name data
| 2 | 2 | | Substitute name size in bytes
| 4 | 2 | | Display name offset, where the offset is relative from the start of the reparse name data
| 6 | 2 | | Display name size, in bytes
| 8 | 4 | | Symbolic link flags
| <td colspan="4"> *Reparse name data*
| 12 | ... | | Substitute name, which contains an UCS-2 little-endian string without end-of-string character
| ... | ... | | Display name, which contains an UCS-2 little-endian string without end-of-string character

#### Symbolic link flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | SYMLINK_FLAG_RELATIVE | The substitute name is a path name relative to the directory containing the symbolic link.

### <a name="wof_reparse_data"></a>Windows Overlay Filter (WOF) reparse data

A reparse point with tag IO_REPARSE_TAG_WOF (0x80000017) contains Windows
Overlay Filter (WOF) reparse data. The Windows Overlay Filter (WOF) reparse
data is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *External provider information*
| 0 | 4 | 1 | Unknown (WOF version)
| 4 | 4 | 2 | Unknown (WOF provider)
| <td colspan="4"> *Internal provider information*
| 8 | 4 | 1 | Unknown (file information version)
| 12 | 4 | | [Compression method](#wof_compression_method)

### <a name="wof_compression_method"></a>Windows Overlay Filter (WOF) compression method

| Value | Identifier | Description
| --- | --- | ---
| 0 | | LZXPRESS Huffman with 4k window (compression unit)
| 1 | | LZX with 32k window (compression unit)
| 2 | | LZXPRESS Huffman with 8k window (compression unit)
| 3 | | LZXPRESS Huffman with 16k window (compression unit)

TODO: link to LZXPRESS Huffman and LZX documentation

### <a name="wci_reparse_data">Windows Container Isolation (WCI) reparse data

A reparse point with tag IO_REPARSE_TAG_WCI (0x80000018) contains Windows
Container Isolation (WCI) reparse data. The Windows Container Isolation (WCI)
reparse data is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | 1 | Version
| 4 | 4 | 0 | Unknown (reserved)
| 8 | 16 | | Look-up identifier, which contains a GUID
| 24 | 2 | | Name size in bytes
| 26 | ... | | Name, which contains an UCS-2 little-endian string without end-of-string character

## The allocation bitmap

The metadata file $Bitmap contains the allocation bitmap.

Every bit in the allocation bitmap represents a block the size of the cluster
block, where the LSB is the first bit in a byte.

TODO: describe what the $SRAT data stream is used for.

## <a name="access_control"></a>Access control

The $Secure metadata file contains the security descriptors used for access control.

| Type | Name | Description
| --- | --- | ---
| Data | $SDS | Security descriptor data stream, which contains all the Security descriptors on the volume
| Index | $SDH | Security descriptor hash index
| Index | $SII | Security descriptor identifier index, which contains the mapping of the security descriptor identifier (in $STANDARD_INFORMATION) to the offset of the security descriptor data (in $Secure:$SDS)

### Security descriptor hash ($SDH) index

#### <a name="security_descriptor_hash_index_value"></a>The security descriptor hash index value

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *Key data*
| 0 | 4 | | Security descriptor hash
| 4 | 4 | | Security descriptor identifier
| <td colspan="4"> *Value data*
| 8 | 4 | | Security descriptor hash
| 12 | 4 | | Security descriptor identifier
| 16 | 8 | | Security descriptor data offset (in $SDS)
| 24 | 4 | | Security descriptor data size (in $SDS)
| 28 | 4 | | Unknown

### Security descriptor identifier ($SII) index

#### <a name="security_descriptor_identifier_index_value"></a>The security descriptor identifier index value

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *Key data*
| 0 | 4 | | Security descriptor identifier
| <td colspan="4"> *Value data*
| 4 | 4 | | Security descriptor hash
| 8 | 4 | | Security descriptor identifier
| 12 | 8 | | Security descriptor data offset (in $SDS)
| 20 | 4 | | Security descriptor data size (in $SDS)

TODO: describe the hash algorithm

### Security descriptor ($SDS) data stream

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Security descriptor hash
| 4 | 4 | | Security descriptor identifier
| 12 | 8 | | Security descriptor data offset (in $SDS)
| 20 | 4 | | Security descriptor data size (in $SDS)
| 24 | ... | | Security descriptor data
| ... | ... | | Alignment padding (2-byte alignment)

TODO: link to security descriptor format documentation

## The object identifiers

### $ObjID:$O

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *Key data*
| 0 | 16 | | File (or object) identifier, which contains a GUID
| <td colspan="4"> *Value data*
| 4 | 8 | | [File reference](#file_reference)
| 12 | 16 | | Birth droid volume identifier, which contains a GUID
| 28 | 16 | | Birth droid file (or object) identifier, which contains a GUID
| 44 | 16 | | Birth droid domain identifier, which contains a GUID

## <a name="log_file"></a>Metadata transaction journal (log file)

TODO: complete section

The metadata file $LogFile contains the metadata transaction journal and
consists of:

* Log File Service restart page header
* [The fix-up values](#fix_up_values)

### Log File service restart page header

The Log File service restart page header (LFS_RESTART_PAGE_HEADER) is 30 bytes
in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *MULTI_SECTOR_HEADER*
| 0 | 4 | "CHKD", "RCRD", "RSTR" | Signature
| 4 | 2 | | The fix-up values (or update sequence array) offset, which contain an offset relative from the start of the restart page header.
| 6 | 2 | | The number of fix-up values (or update sequence array size)
| <td colspan="4"> &nbsp;
| 8 | 8 | | Checkdisk last LSN
| 16 | 4 | | System page size
| 20 | 4 | | Log page size
| 24 | 2 | | Restart offset
| 26 | 2 | | Minor format version
| 28 | 2 | | Major format version

#### Log File service restart page versions

| Major format version | Remarks
| --- | ---
| -1 | Beta Version
| 0 | Transition
| 1 | Update sequence support

## <a name="usn_change_journal"></a>USN change journal

The metadata file $Extend\$UsnJrnl contains the USN change journal. It is a
sparse file in which NTFS stores records of changes to files and directories.
Applications make use of the journal to respond to file and directory changes
as they occur, like e.g. the Windows File Replication Service (FRS) and the
Windows (Desktop) Search service.

The USN change journal consists of:

* the $UsnJrnl:$Max data stream, containing metadata like the maximum size of the journal
* the $UsnJrnl:$J data stream, containing the update (or change) entries. The $UsnJrnl:$J data stream is sparse.

### USN change journal metadata

The USN change journal metadata is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Maximum size in bytes
| 8 | 8 | | Allocation (size) delta in bytes
| 16 | 8 | | Update (USN) journal identifier, which contains a FILETIME
| 24 | 8 | | Unknown (empty)

## USN change journal entries

The $UsnJrnl:$J data stream consists of an array of USN change journal entries.
The USN change journal entries are stored on a per block-basis and 8-byte
aligned. Therefore the remainder of the block can contain 0-byte values.

TODO: describe journal block size

Once the stream reaches maximum size the earliest USN change journal entries
are removed from the stream and replaced with a sparse data run.

### USN change journal entry

The USN change journal entry (USN_RECORD_V2) is of variable size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Entry (or record) size
| 4 | 2 | 2 | Major format version
| 6 | 2 | 0 | Minor format version
| 8 | 8 | | File reference
| 16 | 8 | | Parent file reference
| 24 | 8 | | Update sequence number (USN), which contains the file offset of the USN change journal entry which is used as a unique identifier
| 32 | 8 | | Update date and time, which contains a FILETIME
| 40 | 4 | | [Update reason flags](#update_reason_flags)
| 44 | 4 | | [Update source flags](#update_source_flags)
| 48 | 4 | | Security descriptor identifier, which contains the entry number in the security ID index ($Secure:$SII). Also see [Access Control](#access_control)
| 52 | 4 | | [File attribute flags](#file_attribute_flags)
| 56 | 2 | | Name size in bytes
| 58 | 2 | | Name offset, which is relative from the start of the USN change journal entry
| 60 | (name size) | | Name, which contains an UCS-2 little-endian string without end-of-string character
| ... | ... | 0x00 | Unknown (Padding)

#### <a name="update_reason_flags"></a>Update reason flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | USN_REASON_DATA_OVERWRITE | The data in the file or directory is overwritten.
| 0x00000002 | USN_REASON_DATA_EXTEND | The file or directory is extended.
| 0x00000004 | USN_REASON_DATA_TRUNCATION | The file or directory is truncated.
| | |
| 0x00000010 | USN_REASON_NAMED_DATA_OVERWRITE | One or more named data streams ($DATA attributes) of file were overwritten
| 0x00000020 | USN_REASON_NAMED_DATA_EXTEND | One or more named data streams ($DATA attributes) of file were extended
| 0x00000040 | USN_REASON_NAMED_DATA_TRUNCATION | One or more named data streams ($DATA attributes) of a file were truncated
| | |
| 0x00000100 | USN_REASON_FILE_CREATE | The file or directory was created
| 0x00000200 | USN_REASON_FILE_DELETE | The file or directory was deleted
| 0x00000400 | USN_REASON_EA_CHANGE | The extended attributes of the file were changed
| 0x00000800 | USN_REASON_SECURITY_CHANGE | The access rights (security descriptor) of a file or directory were changed
| 0x00001000 | USN_REASON_RENAME_OLD_NAME | The name changed, where the USN change journal entry contains the old name
| 0x00002000 | USN_REASON_RENAME_NEW_NAME | The name changed, where the USN change journal entry contains the new name
| 0x00004000 | USN_REASON_INDEXABLE_CHANGE | Content indexed status changed. The file attribute FILE_ATTRIBUTE_NOT_CONTENT_INDEXED was changed
| 0x00008000 | USN_REASON_BASIC_INFO_CHANGE | Basic file or directory attributes changed. One or more file or directory attributes were changed e.g. read-only, hidden, system, archive, or sparse attribute, or one or more time stamps.
| 0x00010000 | USN_REASON_HARD_LINK_CHANGE | A hard link was created or deleted
| 0x00020000 | USN_REASON_COMPRESSION_CHANGE | The file or directory was compressed or decompressed
| 0x00040000 | USN_REASON_ENCRYPTION_CHANGE | The file or directory was encrypted or decrypted
| 0x00080000 | USN_REASON_OBJECT_ID_CHANGE | The object identifier of a file or directory was changed
| 0x00100000 | USN_REASON_REPARSE_POINT_CHANGE | The reparse point that in a file or directory was changed, or a reparse point was added to or deleted from a file or directory.
| 0x00200000 | USN_REASON_STREAM_CHANGE | A named data stream ($DATA attribute) is added to or removed from a file, or a named stream is renamed
| 0x00400000 | USN_REASON_TRANSACTED_CHANGE | Unknown
| | |
| 0x80000000 | USN_REASON_CLOSE | The file or directory was closed

#### <a name="update_source_flags"></a>Update source flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | USN_SOURCE_DATA_MANAGEMENT | The operation added a private data stream to a file or directory. The modifications did not change the application data.
| 0x00000002 | USN_SOURCE_AUXILIARY_DATA | The operation was caused by the operating system. Although a write operation is performed on the item, the data was not changed.
| 0x00000004 | USN_SOURCE_REPLICATION_MANAGEMENT | The operation was caused by file replication

## Alternate data streams (ADS)

| Data stream name | Description
| --- | ---
| "♣BnhqlkugBim0elg1M1pt2tjdZe", "♣SummaryInformation", "{4c8cc155-6c1e-11d1-8e41-00c04fb9386d}" | Used to store properties, where ♣ (black club) is Unicode character U+2663
| "{59828bbb-3f72-4c1b-a420-b51ad66eb5d3}.XPRESS" | Used during remote differential compression
| "AFP_AfpInfo", "AFP_Resource" | Used to store Macintosh operating system property lists
| "encryptable" | Used to store attributes relating to thumbnails in the thumbnails database
| "favicon" | Used to store favorite icons for web pages
| "ms-properties" | Used to store properties
| "OECustomProperty" | Used to store custom properties related to email files
| "Zone.Identifier" | Used to store the Internet Explorere URL security zone of the origin

### ms-properties

The ms-properties alternate data stream contains a Windows Serialized Property
Store (SPS).

TODO: link to Windows Serialized Property Store (SPS) format documentation

### Zone.Identifier

The Zone.Identifier alternate data stream contains ASCII text in the form:

```
[ZoneTransfer]
ZoneId=3
```

Where ZoneId refers to the [Internet Explorer URL security zone](https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/ms537183(v=vs.85))
of the origin.

## <a name="transactional_ntfs"></a>Transactional NTFS (TxF)

As of Vista Transactional NTFS (TxF) was added.

In TxF the resource manager (RM) keeps track of transactional metadata and log
files. The TxF related metadata files are stored in the metadata directory:

```
$Extend\$RmMetadata
```

### Resource manager repair information

The resource manager repair information metadata file:
$Extend\$RmMetadata\$Repair consists of the following data streams:

* the default (unnamed) data stream
* the $Config data stream, contains the resource manager repair configuration information

TODO: determine the purpose of the default (unnamed) data stream

#### Resource manager repair configuration information

TODO: complete section

The $Repair:$Config data streams contains:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Unknown
| 4 | 4 | | Unknown

### Transactional NTFS (TxF) metadata directory

TODO: complete section

The transactional NTFS (TxF) metadata directory: $Extend\$RmMetadata\$Txf is
used to isolate files for delete or overwrite operations.

### TxF Old Page Stream (TOPS) file

The TxF Old Page Stream (TOPS) file: $Extend\$RmMetadata\$TxfLog\$Tops consists
of the following data streams:

* the default (unnamed) data stream, contains metadata about the resource manager, such as its GUID, its CLFS log policy, and the LSN at which recovery should start
* the $T data stream, contains the file data that is partially overwritten by a transaction as opposed to a full overwrite, which would move the file into the Transactional NTFS (TxF) metadata directory

#### TxF Old Page Stream (TOPS) metadata

TODO: complete section

The $Tops default (unnamed) data streams contains:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Unknown
| 2 | 2 | | Size of TOPS metadata
| 4 | 4 | | Unknown (Number of resource managers/streams?)
| 8 | 16 | | Resource Manager (RM) identifier, which contains a GUID
| 24 | 8 | | Unknown (empty)
| 32 | 8 | | Base (or log start) LSN of TxFLog stream
| 40 | 8 | | Unknown
| 48 | 8 | | Last flushed LSN of TxFLog stream
| 56 | 8 | | Unknown
| 64 | 8 | | Unknown (empty)
| 72 | 8 | | Unknown (Restart LSN?)
| 80 | 20 | | Unknown

#### TxF Old Page Stream (TOPS) file data

The $Tops:$T data streams contains the file data that is partially overwritten
by a transaction. It consists of multiple pending transaction XML-documents.

TODO: describe start of each sector containing 0x0001

A pending transaction XML-document starts with an UTF-8 byte-order-mark. Is
roughly contains the following data:

```
<?xml version='1.0' encoding='utf-8'?>
<PendingTransaction Version="2.0" Identifier="...">
   <Transactions>
      <Transaction TransactionId="...">
      <Install Application="..., Culture=..., Version=..., PublicKeyToken=...,
                           ProcessorArchitecture=..., versionScope=..."
               RefGuid="..."
               RefIdentifier="..."
               RefExtra="..."/>
      ...
      </Transaction>
   </Transactions>
   <ChangeList>
      <Change Family="..., Culture=..., PublicKeyToken=...,
                     ProcessorArchitecture=..., versionScope=..."
              New="..."/>
      ...
   </ChangeList>
   <POQ>
      <BeginTransaction id="..."/>

      <CreateFile path="..."
                  fileAttribute="..."/>
      <DeleteFile path="..."/>
      <MoveFile source="..." destination="..."/>
      <HardlinkFile source="..." destination="..."/>
      <SetFileInformation path="..."
                          securityDescriptor="binary base64:..."
                          flags="..."/>

       <CreateKey path="..."/>
       <SetKeyValue path="..."
                    name="..."
                    type="..."
                    encoding="base64"
                    value="..."/>
      <DeleteKeyValue path="..."
                      name="..."/>

      ...
   </POQ>
   <InstallerQueue Length="...">
      <Action Installer="..."
              Mode="..."
              Phase="..."
              Family="..., Culture=..., PublicKeyToken=...,
                     ProcessorArchitecture=..., versionScope=..."
              Old="..."
              New="..."/>

      ...
   </InstallerQueue >
</PendingTransaction>
```

### Transactional NTFS (TxF) Common Log File System (CLFS) files

TxF uses a Common Log File System (CLFS) log store and the logged utility
stream attribute named $TXF_DATA.

TODO: link to CLFS format documentation

The base log file (BLF) of the TxF log store is:

```
$Extend\$RmMetadata\$TxfLog\TxfLog.blf
```

Commonly the corresponding container files are:

```
$Extend\$RmMetadata\$TxfLog\TxfLogContainer00000000000000000001
$Extend\$RmMetadata\$TxfLog\TxfLogContainer00000000000000000002
```

TxF uses a multiplexed log store which contains the following streams:

* the KtmLog stream used for Kernel Transaction Manager (KTM) metadata records
* TxfLog stream, which contains the TxF log records.

### Transactional data logged utility stream attribute

The transactional data ($TXF_DATA) logged utility stream attribute is 56 bytes
in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 6 | | Unknown (remnant data)
| 6 | 8 | | Resource manager root file reference, which contains an NTFS file reference that refers to the MFT
| 14 | 8 | | Unknown (USN index?)
| 22 | 8 | | File identifier (TxID), which contains a TxF file identifier
| 30 | 8 | | Data LSN, which contains a CLFS LSN of file data transaction records
| 38 | 8 | | Metadata LSN, which contains a CLFS LSN of file system metadata transaction records
| 46 | 8 | | Directory index LSN, which contains a CLFS LSN of directory index transaction records
| 54 | 2 | | Unknown (Flags?)

> Note that a single MFT entry can contain multiple Transactional data logged
> utility stream attributes.

## Windows definitions

### <a name="file_attribute_flags"></a>File attribute flags

The file attribute flags consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | FILE_ATTRIBUTE_READONLY | Is read-only
| 0x00000002 | FILE_ATTRIBUTE_HIDDEN | Is hidden
| 0x00000004 | FILE_ATTRIBUTE_SYSTEM | Is a system file or directory
| 0x00000008 | | Is a volume label, which is not used by NTFS
| 0x00000010 | FILE_ATTRIBUTE_DIRECTORY | Is a directory, which is not used by NTFS
| 0x00000020 | FILE_ATTRIBUTE_ARCHIVE | Should be archived
| 0x00000040 | FILE_ATTRIBUTE_DEVICE | Is a device, which is not used by NTFS
| 0x00000080 | FILE_ATTRIBUTE_NORMAL | Is normal file. Note that none of the other flags should be set
| 0x00000100 | FILE_ATTRIBUTE_TEMPORARY | Is temporary
| 0x00000200 | FILE_ATTRIBUTE_SPARSE_FILE | Is a sparse file
| 0x00000400 | FILE_ATTRIBUTE_REPARSE_POINT | Is a reparse point or symbolic link
| 0x00000800 | FILE_ATTRIBUTE_COMPRESSED | Is compressed
| 0x00001000 | FILE_ATTRIBUTE_OFFLINE | Is offline. The data of the file is stored on an offline storage.
| 0x00002000 | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED | Do not index content. The content of the file or directory should not be indexed by the indexing service.
| 0x00004000 | FILE_ATTRIBUTE_ENCRYPTED | Is encrypted
| 0x00008000 | | Unknown (seen on Windows 95 FAT)
| 0x00010000 | FILE_ATTRIBUTE_VIRTUAL | Is virtual

The following flags are mainly used in the file name attribute and sparsely in
the standard information attribute. It could be that they have a different
meaning in both types of attributes or that the standard information flags are
not updated. For now the latter is assumed.

| Value | Identifier | Description
| --- | --- | ---
| 0x10000000 | | Unknown (Is directory or has $I30 index? Note that an $Extend directory without this flag has been observed)
| 0x20000000 | | Is index view

## <a name="corruption_scenarios"></a>Corruption scenarios

### Data steam with inconsistent data flags

An MFT entry contains an $ATTRIBUTE_LIST attribute that contains multiple $DATA
attributes. The $DATA attributes define a LZNT1 compressed data stream though
only the first $DATA attribute has the compressed data flag set.

> Note that it is unclear if this is a corruption scenario or not.

```
MFT entry: 220 information:
    Is allocated                   : true
    File reference                 : 220-59
    Base record file reference     : Not set (0)
    Journal sequence number        : 51876429013
    Number of attributes           : 5

Attribute: 1
    Type                           : $STANDARD_INFORMATION (0x00000010)
    Creation time                  : Jun 05, 2019 06:56:26.032730300 UTC
    Modification time              : Oct 05, 2019 06:56:04.150940700 UTC
    Access time                    : Oct 05, 2019 06:56:04.150940700 UTC
    Entry modification time        : Oct 05, 2019 06:56:04.150940700 UTC
    Owner identifier               : 0
    Security descriptor identifier : 5862
    Update sequence number         : 11553149976
    File attribute flags           : 0x00000820
       Should be archived (FILE_ATTRIBUTE_ARCHIVE)
       Is compressed (FILE_ATTRIBUTE_COMPRESSED)

Attribute: 2
    Type                           : $ATTRIBUTE_LIST (0x00000020)

Attribute: 3
    Type                           : $FILE_NAME (0x00000030)
    Parent file reference          : 33996-57
    Creation time                  : Jun 05, 2019 06:56:26.032730300 UTC
    Modification time              : Oct 05, 2019 06:56:03.510061800 UTC
    Access time                    : Oct 05, 2019 06:56:03.510061800 UTC
    Entry modification time        : Oct 05, 2019 06:56:03.510061800 UTC
    File attribute flags           : 0x00000020
       Should be archived (FILE_ATTRIBUTE_ARCHIVE)
    Namespace                      : POSIX (0)
    Name                           : setupapi.dev.20191005_085603.log

Attribute: 4
    Type                           : $DATA (0x00000080)
    Data VCN range                 : 513 - 1103
    Data flags                     : 0x0000

Attribute: 5
    Type                           : $DATA (0x00000080)
    Data VCN range                 : 0 - 512
    Data size                      : 4487594 bytes
    Data flags                     : 0x0001
```

### Directory entry with outdated file reference

The directory entry: \ProgramData\McAfee\Common Framework\Task\5.ini

```
File entry:
    Path                           : \ProgramData\McAfee\Common Framework\Task\5.ini
    File reference                 : 51106-400
    Name                           : 5.ini
    Parent file reference          : 65804-10
    Size                           : 723
    Creation time                  : Sep 16, 2011 20:47:54.561041200 UTC
    Modification time              : Apr 07, 2012 21:07:02.684060000 UTC
    Access time                    : Apr 07, 2012 21:07:02.652810200 UTC
    Entry modification time        : Apr 07, 2012 21:07:02.684060000 UTC
    File attribute flags           : 0x00002020
       Should be archived (FILE_ATTRIBUTE_ARCHIVE)
       Content should not be indexed (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED)
```

The corresponding MFT entry:

```
MFT entry: 51106 information:
    Is allocated                   : true
    File reference                 : 51106-496
    Base record file reference     : Not set (0)
    Journal sequence number        : 0
    Number of attributes           : 3

Attribute: 1
    Type                           : $STANDARD_INFORMATION (0x00000010)
    Creation time                  : Sep 16, 2011 20:47:54.561041200 UTC
    Modification time              : Apr 07, 2012 21:07:02.684060000 UTC
    Access time                    : Apr 07, 2012 21:07:02.652810200 UTC
    Entry modification time        : Apr 07, 2012 21:07:02.684060000 UTC
    Owner identifier               : 0
    Security descriptor identifier : 1368
    Update sequence number         : 1947271600
    File attribute flags           : 0x00002020
       Should be archived (FILE_ATTRIBUTE_ARCHIVE)
       Content should not be indexed (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED)

Attribute: 2
    Type                           : $FILE_NAME (0x00000030)
    Parent file reference          : 65804-10
    Creation time                  : Sep 16, 2011 20:47:54.561041200 UTC
    Modification time              : Apr 07, 2012 21:07:02.652810200 UTC
    Access time                    : Apr 07, 2012 21:07:02.652810200 UTC
    Entry modification time        : Apr 07, 2012 21:07:02.652810200 UTC
    File attribute flags           : 0x00002020
       Should be archived (FILE_ATTRIBUTE_ARCHIVE)
       Content should not be indexed (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED)
    Namespace                      : DOS and Windows (3)
    Name                           : 1.ini

Attribute: 3
    Type                           : $DATA (0x00000080)
    Data size                      : 723 bytes
    Data flags                     : 0x0000
```

TODO: determine if $LogFile could be used to recover from this corruption scenario

### LZNT1 compressed block with data size of 0

Not sure if this is a corruption scenario or a data format edge case.

A compression unit (index 30) consisting of the following data runs:

```
reading data run: 60.
data run:
00000000: 11 01 01                                           ...

value sizes                               : 1, 1
number of cluster blocks                  : 1 (size: 4096)
cluster block number                      : 687143 (1) (offset: 0xa7c27000)

reading data run: 61.
data run:
00000000: 01 0f                                              ..

value sizes                               : 1, 0
number of cluster blocks                  : 15 (size: 61440)
cluster block number                      : 0 (0) (offset: 0x00000000)
        Is sparse
```

Contains the following data:

```
a7c27000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
...
a7c27ff0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
```

This relates to an empty LZNT1 compressed block.

```
compressed data offset                    : 0 (0x00000000)
compression chunk header                  : 0x0000
compressed chunk size                     : 1
signature value                           : 0
is compressed flag                        : 0
```

It was observed in 2 differnt NTFS implementations that the entire block is
filled with 0-byte values.

TODO: verify behavior of Windows NTFS implementation.

### Truncated LZNT1 compressed block

Not sure if this is a corruption scenario or a data format edge case.

A compression unit (index 0) consisting of the following data runs:

```
reading data run: 0.
data run:
00000000: 31 08 48 d8 01                                     1.H..

value sizes                               : 1, 3
number of cluster blocks                  : 8 (size: 32768)
cluster block number                      : 120904 (120904) (offset: 0x1d848000)

reading data run: 1.
data run:
00000000: 01 08                                              ..

value sizes                               : 1, 0
number of cluster blocks                  : 8 (size: 32768)
cluster block number                      : 0 (0) (offset: 0x00000000)
        Is sparse
```

Contains the following data:

```
1d848000  bd b7 50 44 46 50 00 01  00 01 00 40 e0 00 07 0b  |..PDFP.....@....|
...
1d84c000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
*
1d84fff0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
```

This relates to a LZNT1 compressed block that appears to be truncated at offset
16384 (0x00004000).

```
compressed data offset                    : 16384 (0x00004000)
compression flag byte                     : 0x00
```

Different behavior was observed in 2 differnt NTFS implementations:
* one implementation fills the compressed block with the uncompressed data it could read and the rest with with 0-byte values
* another implementation seems to provide the data that was already in its buffer

TODO: verify behavior of Windows NTFS implementation.

## References

* [How NTFS Works](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc781134(v=ws.10)), by Microsoft
* [Master File Table](https://learn.microsoft.com/en-us/windows/win32/devnotes/master-file-table), by Microsoft
* [NTFS Attribute Types](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/a82e9105-2405-4e37-b2c3-28c773902d85), by Microsoft
* [File Attribute Constants](https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants), by Microsoft
* [Reparse Tags](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/c8e77b37-3909-4fe6-a4ea-2b9d423b1ee4), by Microsoft
* [NTFS documentation](https://flatcap.github.io/linux-ntfs/ntfs/), by Richard Russon
* [ATTRIBUTE_LIST_ENTRY structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/attribute-list-entry), by Microsoft
* [ATTRIBUTE_RECORD_HEADER structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/attribute-record-header), by Microsoft
* [FILE_RECORD_SEGMENT_HEADER structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/file-record-segment-header), by Microsoft
* [MULTI_SECTOR_HEADER structure](https://learn.microsoft.com/en-us/windows/win32/devnotes/multi-sector-header), by Microsoft
* [REPARSE_DATA_BUFFER structure (ntifs.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_reparse_data_buffer), by Microsoft
* [REPARSE_DATA_BUFFER_EX structure (ntifs.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_reparse_data_buffer_ex), by Microsoft
* [USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2), by Microsoft
* [Zone.Identifier Stream Name](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/6e3f7352-d11c-4d76-8c39-2516a9df36e8), by Microsoft
* [the Internet Explorer URL security zone](https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/ms537183(v=vs.85)), by Microsoft
* [ntfs_layout.h](https://ultradefrag.net/doc/man/ntfs/ntfs_layout.h.html), by Anton Altaparmakov
