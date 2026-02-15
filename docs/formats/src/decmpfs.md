# Apple File System Compression (decmpfs)

[Hierarchical File System (HFS)](hfs.md) and [Apple File System (APFS)](apfs.md) use Apple File
System Compression (decmpfs) to compress file contents.

## Overview

An Apple File System Compression (decmpfs) compressed file consists of:

* an extended attribute named "com.apple.decmpfs"

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
