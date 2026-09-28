# Virtual Disk Image (VDI) format

The Virtual Disk Image (VDI) format is used by VirtualBox virtualization products as one of its
image formats.

## Overview

A VDI file consists of:

* file header
* block map
* media (or image) data blocks

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Character strings | ASCII |

## File header

The file header is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Pre-header*</td> |
| 0 | 64 | | Unknown (file banner text), which contains an ASCII string, seen "<<< Sun xVM VirtualBox Disk Image >>>\n", "<<< Oracle VM VirtualBox Disk Image >>>\n" and "<<< QEMU VM Virtual Disk Image >>>\n" |
| <td colspan="4">*Header*</td> |
| 64 | 4 | "\x7f\x10\xda\xbe" | Signature |
| 68 | 2 | 1 | Minor format version |
| 70 | 2 | 1 | Major format version |
| 72 | 4 | | Header size, which does not include the size of the pre-header |
| 76 | 4 | | [Image type](#image_types) |
| 80 | 4 | | [Image flags](#image_flags) |
| 84 | 256 | | Image description, which contains an ASCII string |
| 340 | 4 | | Blocks map offset, relative to the start of the file |
| 344 | 4 | | Data (blocks) area offset, relative to the start of the file |
| <td colspan="4">*Geometry*</td> |
| 348 | 4 | 0 | Number of cylinders (no longer used?) |
| 352 | 4 | 0 | Number of heads (no longer used?) |
| 356 | 4 | 0 | Number of sectors (no longer used?) |
| 360 | 4 | | Bytes per sector |
| <td colspan="4">&nbsp;</td> |
| 364 | 4 | 0 | Unknown (unused) |
| 368 | 8 | | Data (or disk or media) size, in number of bytes |
| 376 | 4 | | Block size, in number of bytes |
| 380 | 4 | 0 | Unknown (block extra data) |
| 384 | 4 | | Number of (media data) blocks |
| 388 | 4 | | Number of allocated blocks |
| 392 | 16 | | (Disk) identifier, contains an UUID |
| 408 | 16 | | Last snapshot identifier, contains an UUID |
| <td colspan="4">*If header size > 360*</td> |
| 424 | 16 | | Link identifier, contains an UUID |
| 440 | 16 | | Parent (disk) identifier, contains an UUID |
| <td colspan="4">*If header size > 392*</td> |
| 456 | ... | | Unknown |

<!-- rumdl-enable MD033 MD056 -->

### Format versions

| Format version | Description |
| --- | --- |
| 1.0 | Presumably format version used in VirtualBox pre-release |
| 1.1 | Version used by first release of VirtualBox |

### Image types {#image_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | | Dynamic-size (or sparse) VDI image (variant Standard) |
| 2 | | Fixed-size (static) VDI image (variant Fixed) |
| 3 | | Undo (temporary differential image) |
| 4 | | Differential (or differencing) VDI image |

### Image flags {#image_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | VDI_IMAGE_FLAGS_FIXED | Is a fixed image, all blocks were pre-allocated |
| 0x00000002 | VDI_IMAGE_FLAGS_DIFF | Is a differential image |
| | | |
| 0x00000100 | VDI_IMAGE_FLAGS_ZERO_EXPAND | Zero-Expansion Optimization, 0-byte filled block is stored as a block number of -2 (0xfffffffe) on write |

## Block map

The block (or image or index) map contains an array of 32-bit block numbers.

Where block number:

* -1 (0xffffffff) represents an unallocated block. The block is sparse or stored in the parent file.
* -2 (0xfffffffe) represents a zero block. The block is sparse.
* physical block number, where 0 represents the first block. The block is stored in the file.

```python
block_physical_offset = data_area_offset + (physical_block_number * block_size)
```
