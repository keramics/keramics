# Copy in and out (CPIO) archive format

The copy in and out (CPIO) archive format is an archive format that predates TAR but is still being
used e.g. in initramfs or rpm.

## Overview

There are multiple variants of the CPIO format:

* binary CPIO format
* portable ASCII CPIO format
* new ASCII CPIO format

A CPIO file consists of:

* An array of CPIO file records where the last record contains the path "TRAILER!!!"
* Trailing 0-byte values

> Note that the format does not require (or enforce) parent directory file records to exist within
> the archive file.

| Characteristics | Description |
| --- | --- |
| Byte order | format dependent |
| Date and time values | POSIX timestamp in UTC |
| Character strings | UTF-8 or a narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a system defined codepage |

## Binary CPIO format

The binary CPIO format (or "bin"), sometimes referred to as the old CPIO format, stores the values
as either big-endian or little-endian binary data.

### Binary CPIO file record

The binary CPIO file record is variable of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Signature, which is 070707 in octal where [0x71, 0xc7] indicates the format is in big-endian and [0xc7, 0x71] little-endian |
| 2 | 2 | | Device number of the host file system |
| 4 | 2 | | Inode number |
| 6 | 2 | | [File mode](#file_mode) (permissions and type) |
| 8 | 2 | | Owner identifier (uid) |
| 10 | 2 | | Group identifier (gid) |
| 12 | 2 | | Number of links |
| 14 | 2 | | Block or character special device identifier or 0 if not set |
| 16 | 4 | | Modification time, where the most significant 16-bits are stored in the first 2 bytes and the least significant 16-bits in the last 2 bytes |
| 20 | 2 | | Size of path string, including the end-of-string character (NUL) |
| 22 | 4 | | File data size, where the most the most significant 16-bits in the first 2 bytes and the least significant 16-bits in the last 2 bytes |
| 26 | ... | | Path string |
| ... | ... | | 16-bit alignment padding, which should be set to 0 |
| ... | ... | | File data |
| ... | ... | | 16-bit alignment padding, which should be set to 0 |

## Portable ASCII CPIO format

The portable ASCII CPIO format (or "odc"), sometimes referred to as the old character or POSIX.1
format, stores the values are as octal strings.

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 6 | "070707" | Signature |
| 6 | 6 | | Device number of the host file system |
| 12 | 6 | | Inode number |
| 18 | 6 | | [File mode](#file_mode) (permissions and type) |
| 24 | 6 | | Owner identifier (uid) |
| 30 | 6 | | Group identifier (gid) |
| 36 | 6 | | Number of links |
| 42 | 6 | | Block or character special device identifier or 0 if not set |
| 48 | 11 | | Modification time |
| 59 | 6 | | Size of path string, including the end-of-string character (NUL) |
| 65 | 11 | | File data size |
| 76 | ... | | Path string |
| ... | ... | | File data |

## New ASCII CPIO format

The new (SVR4) ASCII CPIO format stores the values as hexadecimal strings. The new ASCII CPIO
format is technically 2 different formats one without ("newc") and one with a checksum ("crc").
Both support file systems having more than 65536 inodes.

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 6 | "070701" or "070702" | Signature |
| 6 | 8 | | Inode number |
| 14 | 8 | | [File mode](#file_mode) (permissions and type) |
| 22 | 8 | | Owner identifier (uid) |
| 30 | 8 | | Group identifier (gid) |
| 38 | 8 | | Number of links |
| 46 | 8 | | Modification time |
| 54 | 8 | | File data size |
| 62 | 8 | | Device major number of the host file system |
| 70 | 8 | | Device minor number of the host file system |
| 78 | 8 | | Block or character special device major number |
| 86 | 8 | | Block or character special device minor number |
| 94 | 8 | | Size of path string, including the end-of-string character (NUL) |
| 102 | 8 | | Checksum, which contains a a Sum32 if signature is "070702", or 0 otherwise |
| 110 | ... | | Path string |
| ... | ... | | 32-bit alignment padding, which should be set to 0 |
| ... | ... | | File data |
| ... | ... | | 32-bit alignment padding, which Should be set to 0 |

### Checksum

TODO: complete section

## File mode {#file_mode}

<!-- rumdl-disable MD033 MD056 -->

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01ff | | Permissions |
| 0x0200 | | Sticky bit |
| 0x0400 | | SGID bit |
| 0x0800 | | SUID bit |
| <td colspan="4">*File type (0xf000)*</td> |
| 0x1000 | | Named pipe or FIFO |
| 0x2000 | | Character special device |
| 0x4000 | | Directory |
| 0x6000 | | Block special device |
| 0x8000 | | Regular file |
| 0xa000 | | Symbolic link |
| 0xc000 | | Socket |

<!-- rumdl-enable MD033 MD056 -->

## Symbolic links

The link target of a symbolic link is stored as file data.
