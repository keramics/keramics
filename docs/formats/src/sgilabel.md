# SGI (Silicon Graphics) disklabel format

The SGI disklabel format is a partitioning schema mainly used by the SGI Irix operating system.

## Overview

A SGI disklabel consists of one or more partition entries.

Certain paritions have a predefined meaning, such as:

* entry 9 the volume header (partition type 0);
* entry 11 the entire volume (partition type 6).

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | N/A |
| Character strings | ASCII |

## SGI disklabel

The SGI disklabel (or volume header) is stored at offset 0.

The SGI disklabel is 512 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "\x0b\xe5\xa9\x41" | Signature |
| 4 | 2 | | Root partition number |
| 6 | 2 | | Swap partition number |
| 8 | 16 | | ARCS boot file name, with consists of an ASCII string  |
| 24 | 48 | | Device parameters |
| 72 | 15 x 16 = 240 | | Array of [volume descriptors](#volume_descriptor) |
| 312 | 16 x 12 = 192 | | Array of [partition entries](#partition_entry) |
| 504 | 4 | | Checksum |
| 508 | 4 | | Unknown (padding) |

### Device parameters

The device parameters are 48 bytes of size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Skew |
| 1 | 1 | | Gap 1 size |
| 2 | 1 | | Gap 2 size |
| 3 | 1 | | Number of spare cylinders (per volume) |
| 4 | 2 | | Number of (physical) cylinders |
| 6 | 2 | | Heads per volume |
| 8 | 2 | | Tracks per cylinder |
| 10 | 1 | | Unknown (cmd_tag_queue_depth) |
| 11 | 3 | | Unknown (unused) |
| 14 | 2 | | Sectors per track |
| 16 | 2 | | Bytes per sector |
| 18 | 2 | | Unknown (ilfact) |
| 20 | 4 | | Unknown (flags) |
| 24 | 4 | | Unknown (datarate) |
| 28 | 4 | | Unknown (retries_on_error) |
| 32 | 4 | | Unknown (ms_per_word) |
| 36 | 2 | | Unknown (xylogics_gap1) |
| 38 | 2 | | Unknown (xylogics_syncdelay) |
| 40 | 2 | | Unknown (xylogics_readdelay) |
| 42 | 2 | | Unknown (xylogics_gap2) |
| 44 | 2 | | Unknown (xylogics_readgate) |
| 46 | 2 | | Unknown (xylogics_writecont) |

### Volume descriptor {#volume_descriptor}

The volume descriptors contain a flat file system, where each individual descriptor describes a
execute boot file. These execute boot files are stored within the volume header partition.

The volume descriptor is 16 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | File name, with consists of an ASCII string |
| 8 | 4 | | Start sector number, relative to the start of the volume header partition |
| 12 | 4 | | Size, in number of bytes |

### Partition entry {#partition_entry}

The partition entry is 12 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of sectors, relative to the start of the SGI disklabel |
| 4 | 4 | | Start sector number |
| 8 | 4 | | Partition type |

### Partition types

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | SGI_VOLHDR | Volume header |
| 1 | SGI_TRKREPL | Track Replacements |
| 2 | SGI_SECREPL | Sector Replacements |
| 3 | SGI_SWAP | IRIX Swap |
| 4 | SGI_BSD or SGI_RAW | SGI BSD or raw |
| 5 | SGI_SYSV or SGI_BOARD | SGI SystemV, board or overlay |
| 6 | SGI_VOLUME | Entire Volume |
| 7 | SGI_EFS | IRIX EFS |
| 8 | SGI_LVOL | SGI Logical Volume |
| 9 | SGI_RVOL | SGI raw logical volume |
| 10 | SGI_XFS | XFS |
| 11 | SGI_XFSLOG | XFS journal |
| 12 | SGI_XLV | SGI XLV Volume Manager |
| 13 | SGI_XVM | SGI XVM Volume Manager |
