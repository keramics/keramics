# Macintosh File System (MFS)

The Macintosh File System (MFS) is the first file system created for Mac OS, intended for 400 KiB
floppy disks.

## Overview

A MFS file system consists of:

* optional [boot block](#boot_block)
* [master directory block (MDB)](#master_directory_block)
* file directory area
* data area
* optional backup (or alternate) [master directory block (MDB)](#master_directory_block)

The backup master directory block (MDB), is stored in the last 2 sectors of the volume.

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | TODO |
| Character strings | Narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a system defined codepage |

### Terminology

| Term | Description |
| --- | --- |
| Clump size | Size of the group of (allocation) blocks (or clump), in bytes, to avoid fragmentation |

## Boot Block {#boot_block}

If a volume is bootable, the first 2 blocks of the volume contain boot block. The boot block
consists of:

* boot block header
* boot code
* unknown (filler)

### Boot Block Header

The boot block header is 138 or 144 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "LK" (or "\x4c\x4b") | Boot block signature |
| 2 | 4 | | Boot code entry point |
| 6 | 1 | | [Flags](#boot_block_header_flags) |
| 7 | 1 | | Format version |
| 8 | 2 | | Page flags (or Secondary Sound and Video Pages) |
| 10 | 1 | | System file name size, with a maximum of 15 |
| 11 | 15 | | System file name |
| 26 | 1 | | Finder (or shell) file name size, with a maximum of 15 |
| 27 | 15 | | Finder (or shell) file name, typically "Finder" |
| 42 | 1 | | Debugger file name size, with a maximum of 15  |
| 43 | 15 | | Debugger file name, typically "Macsbug" |
| 58 | 1 | | Disassembler (or second debugger) file name size, with a maximum of 15 |
| 59 | 15 | | Disassembler (or second debugger) file name, typically "Disassembler" |
| 74 | 1 | | Startup screen file name size, with a maximum of 15 |
| 75 | 15 | | Startup screen file name, typically "StartUpScreen" |
| 90 | 1 | | Startup (or bootup) file name size, with a maximum of 15 |
| 91 | 15 | | Startup (or bootup) file name, typically "Finder" |
| 106 | 1 | | Clipboard (or scrap) file name size, with a maximum of 15 |
| 107 | 15 | | Clipboard (or scrap) file name, typically "Clipboard" |
| 122 | 2 | | Number of allocated file control blocks (FCBs) |
| 124 | 2 | | Number of elements in the event queue, typically 20 |
| 126 | 4 | | System heap size on Macintosh computer with 128 KiB of RAM |
| 130 | 4 | | System heap size on Macintosh computer with 256 KiB of RAM |
| 134 | 4 | | System heap size on Macintosh computer with +512 KiB of RAM |
| <td colspan="4">*Newer boot block header format*</td> |
| 138 | 4 | | Additional system heap space |
| 140 | 4 | | Fraction of available RAM for the system heap |

<!-- rumdl-enable MD033 MD056 -->

> Note that "LK" presumably is short for "Larry Kenyon" who originally designed MFS.

### Boot code entry point

The boot code entry point contains machine-language instructions that translate to:

```text
BRA.S *+ 0x90
```

Or for older versions of the boot block header:

```text
BRA.S *+ 0x88
```

```text
BRA.W *+ 0x88
```

```text
BRA     $88(PC)         * $6000,$0086
```

This instruction jumps to the main boot code following the boot block header.

This field is ignored, however, if bit 6 is clear in the high-order byte of the boot block version
number or if the low-order byte contains 0x0d.

### Boot Block Header Flags {#boot_block_header_flags}

| Bit(s) | Description |
| --- | --- |
| 0 - 4 | Unknown (Reserved), should contain 0 |
| 5 | Use relative system heap sizing |
| 6 | Execute boot code |
| 7 | Newer boot block header format is used |

If bit 7 of the flag byte is clear, then bits 5 and 6 are ignored and the version number is set
in the format version value.

If the format version value is:

* less than 21, the values in the system heap size on 128K Mac and 256K Mac should be ignored and
  the value in system heap size on all machines should be used.
* 13 the boot code should be executed using the value in boot code entry point.
* greater than or equal to 21 the value in system heap size on all machines should be used.

If bit 7 of the flag byte is set

* bit 6 should be used to determine whether to execute the boot code using the value in boot code
  entry point.
* bit 5 should be used to determine whether to use relative System heap sizing. If bit 5 is
  * clear the value in system heap size on all machines should be used.
  * is set the System heap is extended by the value in the additional system heap space plus the
    fraction of available RAM for the system heap.

## Master Directory Block (MDB) {#master_directory_block}

The Master Directory Block (MDB) is located at offset 1024 of the volume and consists of:

* master directory block header
* block map

### Master Directory Block (MDB) header

The Master Directory Block (MDB) header is 64 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | "\xd2\xd7" | Volume signature |
| 2 | 4 | | Creation date and time, which contains a HFS timestamp in local time |
| 6 | 4 | | Last modification date and time, which contains a HFS timestamp in local time |
| 10 | 2 | | [Volume attribute flags](hfs.md#volume_attribute_flags) |
| 12 | 2 | | Number of files in the root directory |
| 14 | 2 | | File directory area sector number, contains a sector number relative from the start of the volume, where 0 is the first sector number |
| 16 | 2 | | File directory area size, in number of sectors |
| 18 | 2 | | Number of blocks |
| 20 | 4 | | Block size, in bytes, must be a multitude of 512 |
| 24 | 4 | | Clump size, in bytes |
| 28 | 2 | | Data area sector number, contains a sector number relative from the start of the volume, where 0 is the first sector number |
| 30 | 4 | | Next available file identifier |
| 34 | 2 | | Number of unused blocks |
| 36 | 1 | | Volume label size, with a maximum of 27 |
| 37 | 27 | | Volume label |

### Block map

TODO: describe similar to FAT-12 block allocation table

## File Directory Area

The file directory area consists of:

* one or more file directory entries, where an individual file directory entry does not span
  multiple blocks

### File Directory Entry

A file directory entry is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Flags, where 0x80 indicates the file directory entry is in use |
| 1 | 1 | 0 | Format version |
| 2 | 4 | "\x3f\x3f\x3f\x3f" | File type |
| 6 | 4 | | File creator |
| 10 | 2 | | [Finder flags](hfs.md#finder_flags) |
| 12 | 4 | | Window position and dimension (boundaries), which contains the top, left, bottom, right-coordinate values |
| 16 | 2 | | Folder file identifier, where 0 represents the main volume, -2 the desktop, -3 the trash, otherwise, if positive, a file identifier |
| 18 | 4 | | File identifier |
| 22 | 2 | | Data fork block number, contains 0 if the file entry has no data fork |
| 24 | 4 | | Data fork size, in bytes |
| 28 | 4 | | Data fork allocated size, in bytes |
| 32 | 2 | | Resource fork block number, contains 0 if the file entry has no resource fork |
| 34 | 4 | | Resource fork size, in bytes |
| 38 | 4 | | Resource fork allocated size, in bytes |
| 42 | 4 | | Creation date and time, which contains a HFS timestamp in local time |
| 46 | 4 | | (Content) modification date and time, which contains a HFS timestamp in local time |
| 50 | 1 | | File name size, with a maximum of 255 |
| 51 | ... | | File name |
| ... | ... | | 16-bit alignment padding |
