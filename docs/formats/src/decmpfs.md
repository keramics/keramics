# Apple File System Compression (decmpfs)

[Hierarchical File System (HFS)](hfs.md) and [Apple File System (APFS)](apfs.md) use Apple File
System Compression (decmpfs) to compress file contents.

decmpfs is sometimes referred to as AFSC (Apple File System Compression) or HFS/HFS+ compression
and was introduced in Mac OS X 10.6 (Snow Leopard).

## Overview

An Apple File System Compression (decmpfs) compressed file consists of:

* an extended attribute named "com.apple.decmpfs"
* compressed file content data

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |

## decmpfs extended attribute

The decmpfs extended attribute consists of:

* decmpfs header
* optional compressed data

### decmpfs header

The decmpfs header is 16 bytes in size and consists of:

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
| 3 | kAFSCTypeZLibChunk | zlib compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 4 | kAFSCTypeZLib | 64k chunked zlib compressed resource fork, where the compressed data is stored in the resource fork |
| 5 | | Unknown (sparse compressed extended attribute data), where the uncompressed data contains 0-byte values. According to [copyfile.c](https://github.com/apple-oss-distributions/copyfile/blob/main/copyfile.c) specifies de-dup within the generation store. |
| 6 | | Unknown (unused) |
| <td colspan="3">*Added in Mac OS X Yosemite (10.10)*</td> |
| 7 | kAFSCTypeLZVNChunk | LZVN compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 8 | kAFSCTypeLZVN | 64k chunked LZVN compressed resource fork, where the compressed data is stored in the resource fork |
| 9 | kAFSCTypeRawChunk | Uncompressed (raw) extended attribute data |
| 10 | kAFSCTypeRaw | 64k chunked uncompressed (raw) data resource fork, where the compressed data is stored in the resource fork |
| <td colspan="3">*Added in Mac OS X El Capitan (10.11)*</td> |
| 11 | kAFSCTypeLZFSEChunk | LZFSE compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 12 | kAFSCTypeLZFSE | 64k chunked LZFSE compressed resource fork, where the compressed data is stored in the resource fork |
| <td colspan="3">*Added in macOS Ventura (13.0)*</td> |
| 13 | kAFSCTypeLZBitmapChunk | LZBITMAP compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 14 | kAFSCTypeLZBitmap | LZBITMAP compressed resource fork, where the compressed data is stored in the resource fork |
| | | |
| 255 | CMP_MAX | Maximum supported compression method |
| | | |
| 0x80000001 | DATALESS_CMPFS_TYPE | Unknown (faulting file or dataless file or directory) |
| 0x80000002 | DATALESS_PKG_CMPFS_TYPE | Unknown (dataless package) |

> Note that [copyfile.c](https://github.com/apple-oss-distributions/copyfile/blob/main/copyfile.c)
> indicates faulting files are deprecated since Mac OS X Yosemite (10.10).

## Compressed file content data

The location of the compressed file content data depends on the compression method:

* sparse data
* compressed data stored in extended attribute
* compressed data stored in resource fork

### Sparse data

[Compression method](decmpfs.md#compression_methods) 5 has been observed to be used for sparse data,
the file content data contains 0-byte values.

There are 12 bytes stored after the decmpfs compressed data header that consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Unknown (Seen: 1) |
| 4 | 4 | | Unknown |
| 8 | 4 | | Unknown (Seen: 0) |

### Compressed data stored in extended attribute

[Compression method](decmpfs.md#compression_methods) 3, 5, 7, 9 and 11 store the compressed file
content data in the extended attribute after the decmpfs compressed data header.

The compressed data consist of 1 compressed data block.

### Compressed data stored in resource fork

[Compression method](decmpfs.md#compression_methods) 4, 8, 10 and 12 store the compressed file
content data in the resource fork of the file.

The compressed data starts with metadata that contains the offsets of the compressed data blocks.

## LZFSE compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the LZFSE compressed data |
| ... | ... | | LZFSE compressed data blocks |

### LZFSE compressed data block

If the first byte in the LZFSE compressed data block is 0xff, the block contains uncompressed data,
otherwise the block should start with a LZFSE block marker.

## LZVN compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the LZVN compressed data |
| ... | ... | | LZVN compressed data blocks |

### LZVN compressed data block

If the first byte in the LZVN compressed data block is 0x06 (end of stream oppcode), the block
contains uncompressed data.

A compressed data block can contains a maximum of 65536 bytes of data. The compressed data block
therefore should not exceed 65537 bytes in size.

## Raw compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the raw compressed data |
| ... | ... | | raw compressed data blocks |

### Raw compressed data block

If the first byte in the raw compressed data block is 0xcc, the block contains uncompressed data.

The behavior of other byte values is unknown, it has been observed that Mac OS returns no data.

## zlib compressed data

* zlib compressed header
* zlib compressed data block descriptors
* zlib compressed data blocks
* zlib compressed footer

### zlib compressed header

The zlib compressed header is 260 bytes size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | 256 | Unknown (header size or offset?) |
| 4 | 4 | | Compressed footer offset, where the offset is relative from the start of the zlib compressed data |
| 8 | 4 | | Unknown (total size - header size?) |
| 12 | 4 | 50 | Compressed footer size |
| 16 | 240 | | Unknown (empty values) |
| 256 | 4 | | Unknown |

> Note that the values in the zlib compressed header are stored in big-endian.

### zlib compressed data block descriptors

The zlib compressed data block descriptors are variable size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of block descriptors (offset and size tuples) |
| 4 | 8 x ... | | Array of compressed data block descriptors |

> Note that the values in the zlib compressed data block descriptors are store in little-endian.

#### zlib compressed data block descriptor

The zlib compressed data block descriptor is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Compressed block offset, where the offset is relative from the start of the zlib compressed data block descriptors |
| 4 | 4 | | Compressed block size |

### zlib compressed footer

The zlib compressed footer is 50 bytes size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 24 | | Unknown (empty values) |
| 24 | 2 | | Unknown (signature offset?) |
| 26 | 2 | | Unknown (footer size?) |
| 28 | 2 | | Unknown |
| 30 | 4 | "cmpf" | signature (DECMPFS_MAGIC) |
| 34 | 2 | | Unknown (empty values?) |
| 36 | 2 | | Unknown |
| 38 | 2 | | Unknown |
| 40 | 2 | | Unknown (uncompressed block size?) |
| 42 | 8 | | Unknown (empty values) |

> Note that the values in the zlib compressed header are stored in big-endian.

### zlib compressed data block

If the first byte in the zlib compressed data block is 0xff, the block contains uncompressed data,
otherwise the block should start with 0x78.
