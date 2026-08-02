# Apple File System Compression (decmpfs)

[Hierarchical File System (HFS)](hfs.md) and [Apple File System (APFS)](apfs.md) use Apple File
System Compression (decmpfs) to compress file contents.

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
| 3 | | zlib compressed extended attribute data, where the compressed data is stored in the extended attribute after the compressed data header |
| 4 | | 64k chunked zlib compressed resource fork, where the compressed data is stored in the resource fork |
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

[Compression method](decmpfs.md#compression_methods) 3 and 7 store the compressed file content data
in the extended attribute after the decmpfs compressed data header.

The compressed data consist of 1 compressed data block.

### Compressed data stored in resource fork

[Compression method](decmpfs.md#compression_methods) 4 and 8 store the compressed file content data
in the resource fork of the file.

The compressed data starts with metadata that contains the offsets of the compressed data blocks.

## LZFSE compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the LZFSE compressed data |
| ... | ... | | LZFSE compressed data blocks |

## LZVN compressed data

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 x ... | | Array of compressed data block offsets, where an offset is relative from the start of the LZVN compressed data |
| ... | ... | | LZVN compressed data blocks |

> Note that if the LZVN compressed data starts with 0x06 (end of stream oppcode) the data is stored
> uncompressed after the first compressed data byte. The compressed data block contains a maximum
> of 65536 bytes of data. The compressed data block therefore should not exceed 65537 bytes in size.

## zlib compressed data

* zlib compressed header
* Unknown (empty values)
* zlib compressed data block descriptors
* zlib compressed data blocks
* zlib compressed footer

> Note that if the zlib compressed data starts with 0xff the data is stored uncompressed after the
> first compressed data byte.

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
| 30 | 4 | "cmpf" | Unknown (signature) |
| 34 | 2 | | Unknown (empty values?) |
| 36 | 2 | | Unknown |
| 38 | 2 | | Unknown |
| 40 | 2 | | Unknown (uncompressed block size?) |
| 42 | 8 | | Unknown (empty values) |

> Note that the values in the zlib compressed header are stored in big-endian.

### zlib compressed data block

If the first byte in the zlib compressed data block is:

* 0x78, the block contains [zlib compressed data](zlib.md);
* 0xff, the block contains uncompressed data.
