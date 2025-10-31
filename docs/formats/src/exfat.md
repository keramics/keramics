# Extensible File Allocation Table (exFAT) file system format

The Extensible File Allocation Table (exFAT) file system format is a successor
of the [File Allocation Table (FAT)](fat.md) file system format.

## Overview

An exFAT file system consists of:

* One or more reserved sectors
  * a boot record (or boot sector)
* One or more cluster block allocation tables
* File and directory data

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | FAT date and time
| Character strings | UCS-2 little-endian, which allows for unpaired Unicode surrogates such as "U+d800" and "U+dc00"

### Boot record

The boot record is stored in the first sector of the volume.

The boot record is at least 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 3 | "\xeb\x76\x90" | Boot entry point (JMP +120, NOP)
| 3 | 8 | "EXFAT\x20\x20\x20" | File system signature (or OEM name)
| 11 | 53 | 0 | Unknown (reserved), which must be 0.
| 64 | 8 | | Partition offset
| 72 | 8 | | Total number of sectors
| 80 | 4 | | Cluster block allocation table start sector
| 84 | 4 | | Cluster block allocation table size in number of sectors, which must be non 0
| 88 | 4 | | Data cluster start sector
| 92 | 4 | | Total number of data clusters
| 96 | 4 | | Root directory start cluster
| 100 | 4 | | Volume serial number
| 104 | 1 | | Format revision minor number
| 105 | 1 | 1 | Format revision major number
| 106 | 2 | | Volume flags
| 108 | 1 | | Bytes per sector, which is stored as `2^n`, for example 9 is `2^9 = 512`. The bytes per sector value must be 512, 1024, 2048 or 4096.
| 109 | 1 | | Sectors per cluster block, which is stored as `2^n`, for example 3 is `2^3 = 8`. The sectors per cluster block must be 1 upto 32M (`2^25`).
| 110 | 1 | | Number of cluster block allocation tables
| 111 | 1 | | Drive number
| 112 | 1 | | Unknown (percent in use), which contains the percentage of allocated cluster blocks in the cluster heap of 0xff if not available
| 113 | 7 | | Unknown (reserved)
| 120 | 390 | | Used for boot code
| 510 | 2 | "\x55\xaa" | Sector signature

### Volume flags

| Value | Identifier | Description
| --- | --- | ---
| 0x0001 | ActiveFat | Active FAT, 0 for the first FAT, 1 for the second FAT
| 0x0002 | VolumeDirty | Is dirty
| 0x0004 | MediaFailure | Has media failures
| 0x0008 | ClearToZero | Must be cleared
| 0xfff0 | | Unknown (reserved)

## Cluster block allocation table

A cluster block allocation table consists of:

* One ore more cluster block allocation table entries

### Cluster block allocation table entry

A cluster block allocation table entry is 32 bits in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 32 bits | | Data cluster number

Where the data cluster number has the following meanings:

| Value(s) | Description
| --- | ---
| 0x00000000 | Unused (free) cluster
| 0x00000001 | Unknown (invalid)
| 0x00000002 - 0xffffffef | Used cluster
| 0xfffffff0 - 0xfffffff6 | Reserved
| 0xfffffff7 | Bad cluster
| 0xfffffff8 - 0xffffffff | End of cluster chain

## Directory

A directory consists of:

* Zero or more directory entries
* Terminator directory entry

### Directory entry

A directory entry is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | | Entry type
| 1 | 19 | | Entry data
| 20 | 4 | | Data stream start cluster
| 24 | 8 | | Data stream size

#### Directory entry type

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 5 bits | | Type type code
| 0.5 | 1 bit | | Is non-critical (also referred to as type importance)
| 0.6 | 1 bit | | Is secondary entry (also referred to as type category)
| 0.7 | 1 bit | | In use

| Value | Description
| --- | ---
| 0x00 | Terminator directory entry
| 0x01 - 0x7f | Unused
| 0x80 | Invalid
| 0x81 - 0xff | Used

##### Directory entry type codes

| Value | Description
| --- | ---
| <td colspan="4"> *Critical and primary*
| 0x81 | Allocation bitmap
| 0x82 | Case folding mappings
| 0x83 | Volume label
| |
| 0x85 | File entry
| |
| <td colspan="4"> *Non-critical and primary*
| 0xa0 | Volume identifier
| 0xa1 | TexFAT padding
| |
| <td colspan="4"> *Critical and secondary*
| 0xc0 | Data stream
| 0xc1 | File entry name
| |
| <td colspan="4"> *Non-critical and secondary*
| 0xe0 | Vendor extension
| 0xe1 | Vendor allocation

##### Allocation bitmap directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0x81 | Entry type
| 1 | 1 | | Bitmap flags
| 2 | 18 | 0 | Unknown (Reserved)
| 20 | 4 | | Data stream start cluster
| 24 | 8 | | Data stream size

##### Case folding mappings directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0x82 | Entry type
| 1 | 3 | 0 | Unknown (Reserved)
| 4 | 4 | | Checksum
| 8 | 12 | 0 | Unknown (Reserved)
| 20 | 4 | | Data stream start cluster
| 24 | 8 | | Data stream size

##### Volume label directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0x83 | Entry type
| 1 | 1 | | Name number of characters
| 2 | 22 | | Name string, which contains an UCS-2 little-endian string without an end-of-string character
| 24 | 8 | 0 | Unknown (Reserved)

> Note that the volume label directory entry should only be stored in the first
> and/or second directory entry of the root directory.

##### File entry directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0x85 | Entry type
| 1 | 1 | | Unknown (Secondary count)
| 2 | 2 | | Unknown (Set checksum)
| 4 | 2 | | [File attribute flags](#file_attribute_flags)
| 6 | 2 | 0 | Unknown (Reserved)
| 8 | 2 | | Creation time
| 10 | 2 | | Creation date
| 12 | 2 | | Last modification time
| 14 | 2 | | Last modification date
| 16 | 2 | | Last access time
| 18 | 2 | | Last access date
| 20 | 1 | | Creation time fraction of seconds, which contains fraction of 2-seconds in 10 ms intervals
| 21 | 1 | | Last modification time fraction of seconds, which contains fraction of 2-seconds in 10 ms intervals
| 22 | 1 | | Creation time UTC offset, which contains number of 15 minute intervals of the time relative to UTC, where the MSB indicates the offset is valid
| 23 | 1 | | Last modification time UTC offset, which contains number of 15 minute intervals of the time relative to UTC, where the MSB indicates the offset is valid
| 24 | 1 | | Last access time UTC offset, which contains number of 15 minute intervals of the time relative to UTC, where the MSB indicates the offset is valid
| 25 | 7 | 0 | Unknown (Reserved)

##### Volume identifier directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0xa0 | Entry type
| 1 | 1 | | Unknown (Secondary count)
| 2 | 2 | | Unknown (Set checksum)
| 4 | 2 | | Unknown (Flags)
| 6 | 16 | | Volume identifier, which contains a GUID
| 22 | 10 | 0 | Unknown (Reserved)

##### Data stream directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0xc0 | Entry type
| 1 | 1 | | Unknown (Flags)
| 2 | 1 | 0 | Unknown (Reserved)
| 3 | 1 | | Name number of characters
| 4 | 2 | | Name hash
| 6 | 2 | 0 | Unknown (Reserved)
| 8 | 8 | | Data stream valid data size
| 16 | 4 | 0 | Unknown (Reserved)
| 20 | 4 | | Data stream start cluster
| 24 | 8 | | Data stream size

##### File entry name directory entry

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | 0xc1 | Entry type
| 1 | 1 | | Unknown (Flags)
| 2 | 30 | | Name string, which contains an UCS-2 little-endian string without an end-of-string character

### <a name="file_attribute_flags"></a>File attribute flags

| Value | Description
| --- | ---
| 0x0001 | Read-only
| 0x0002 | Hidden
| 0x0004 | System
| 0x0008 | Is volume label
| 0x0010 | Is directory
| 0x0020 | Archive
| 0x0040 | Is device
| 0x0080 | Unused (reserved)

## References

* [exFAT file system specification](https://learn.microsoft.com/en-gb/windows/win32/fileio/exfat-specification), by Microsoft
* [exFAT](https://en.wikipedia.org/wiki/ExFAT), by Wikipedia
