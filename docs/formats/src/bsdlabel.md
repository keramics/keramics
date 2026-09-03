# BSD disklabel format

The BSD disklabel format is a partitioning schema mainly used by BSD operating systems.

## Overview

A BSD disklabel consists of one or more partition entries, labeled alphabetically from "a" to "h"
(or "p" in some BSD variants).

> Note that BSD disklabel originally contained 8 entries for describing partitions and some BSD
> variants have since increased this to 16 partitions.

Certain labels have a predefined meaning, such as:

* "a" is the "root" partition
* "b" is the "swap" partition
* "c" is the volume used by disklabel
* "d" is the entire physical disk

> Note that information about partition "d" is not stored in the corresponding BSD disklabel
> partition entry and typically filled with 0-byte values.

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Date and time values | N/A |
| Character strings | ASCII |

The number of bytes per sector is 512.

## BSD disklabel

The BSD disklabel is stored at offset 512. It can be preceded by a [MBR](mbr.md) with a single
partition of type 0xa5 (FreeBSD).

The BSD disklabel is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "WEV\x82" | Signature |
| 4 | 2 | | Drive type |
| 6 | 2 | | Controller specific drive sub type |
| 8 | 16 | | Drive type name, which contains an ASCII string |
| 24 | 16 | | Unknown (Pack identifier?), which contains an ASCII string |
| 40 | 4 | | Bytes per sector |
| 44 | 4 | | (Data) Sectors per track |
| 48 | 4 | | Tracks per cylinder |
| 52 | 4 | | (Data) Cylinders per unit |
| 56 | 4 | | (Data) Sectors per cylinder |
| 60 | 4 | | (Data) Sectors per unit |
| 64 | 2 | | Spare sectors per track |
| 66 | 2 | | Spare sectors per cylinder |
| 68 | 4 | | Alternate cylinders per unit |
| 72 | 2 | | Unknown (Rotational speed?) |
| 74 | 2 | | Unknown (Hardware sector interleave?) |
| 76 | 2 | | Unknown (Sector 0 skew per track?) |
| 78 | 2 | | Unknown (Sector 0 skew per cylinder?) |
| 80 | 4 | | Unknown (Head switch time in microseconds?) |
| 84 | 4 | | Unknown (Track-to-track seek time in microseconds?) |
| 88 | 4 | | Flags |
| 92 | 5 x 4 | | Unknown (Drive-type specific information?) |
| 112 | 5 x 4 | | Unknown (Reserved) |
| 132 | 4 | "WEV\x82" | Signature |
| 136 | 2 | | Checksum, which contains a XOR of the BSD disklabel |
| 138 | 2 | | Number of partition entries, should not exceed 16 (MAXPARTITIONS) |
| 140 | 4 | | Boot area size in bytes |
| 144 | 4 | | Maximum superblock size in bytes |
| 148 | number of partitions x 16 | | Array of [partition entries](#partition_entry) |

> Note that the number of partition entries contains the total number of entries in the array, not
> the number of partitions in use.

The checksum is calculated as following:

* set the checksum value to 0
* XOR every 16-bit value in the disklabel

### Drive types {#drive_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | DTYPE_SMD | SMD, XSMD |
| 2 | DTYPE_MSCP | MSCP |
| 3 | DTYPE_DEC | DEC (rk, rl) |
| 4 | DTYPE_SCSI | SCSI |
| 5 | DTYPE_ESDI | ESDI |
| 6 | DTYPE_ST506 | ST506 |
| 7 | DTYPE_HPIB | CS/80 on HP-IB |
| 8 | DTYPE_HPFL | HP Fiber-link |
| | | |
| 10 | DTYPE_FLOPPY | Floppy drive |

### Flags

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | D_REMOVABLE | Removable media |
| 0x00000002 | D_ECC | Media supports error-correction codes (ECC) |
| 0x00000004 | D_BADSECT | Media suppors bad sectro forwarding |
| 0x00000008 | D_RAMDISK | Emulated media using RAM |
| 0x00000010 | D_CHAIN | Media can do back-to-back transfers |

### Partition entry {#partition_entry}

The partition entry is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of sectors |
| 4 | 4 | | Start sector |
| 8 | 4 | | File system (basic) fragment size |
| 12 | 1 | | File system type |
| 13 | 1 | | File system fragments per block |
| 14 | 2 | | Unknown (File system specific value) |

> Note that an emtpy partition entry consists of 0-byte values.

#### File system types {#file_system_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | FS_UNUSED | Unused |
| 1 | FS_SWAP | Swap |
| 2 | FS_V6 | 6th edition |
| 3 | FS_V7 | 7th edition |
| 4 | FS_SYSV | System V |
| 5 | FS_V71K | 7th edition with 1 KiB blocks |
| 6 | FS_V8 | 8th edition with 4 KiB blocks |
| 7 | FS_BSDFFS | BSD 4.2 fast file system (FFS) |
| 8 | FS_MSDOS | MS-DOS file system |
| 9 | FS_BSDLFS | BSD 4.4 log-structured file system |
| 10 | FS_OTHER | Other (unspecified) file system |
| 11 | FS_HPFS | OS/2 high-performance file system (HPFS) |
| 12 | FS_ISO9660 | ISO 9660 (CD-ROM) file system |
| 13 | FS_BOOT | Boot code |
| 14 | | Unknown (Vinum) |

## References

* [FreeBSD Manual Pages](https://man.freebsd.org/cgi/man.cgi?query=disklabel&sektion=5&apropos=0&manpath=4.4BSD+Lite2)
* [BSD disklabel](https://en.wikipedia.org/wiki/BSD_disklabel), by Wikipedia
