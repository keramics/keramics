# LZFSE compressed data format

LZFSE compression is used in various data formats used on Mac OS, including
[Universal Disk Image Format (UDIF)](udif.md) files (.dmg) and
[Apple File System Compression (decmpfs)](decmpfs.md), which is used in
[Hierarchical File System (HFS)](hfs.md) and [Apple File System (APFS)](apfs.md).

## Overview

LZFSE compressed data stream consist of:

* one or more [blocks](#lzfse_block)

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |

## LZFSE block {#lzfse_block}

A LZFSE block is of variable size and consits of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Block signature |
| 4 | ... | | Block data |

### Block signatures

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000000 | LZFSE_NO_BLOCK_MAGIC | Invalid |
| | | |
| 0x24787662 ("bvx$") | LZFSE_ENDOFSTREAM_BLOCK_MAGIC | End-of-stream block (marker) |
| 0x2d787662 ("bvx-") | LZFSE_UNCOMPRESSED_BLOCK_MAGIC | Uncompressed (raw) block |
| 0x31787662 ("bvx1") | LZFSE_COMPRESSEDV1_BLOCK_MAGIC | LZFSE compressed block with uncompressed tables |
| 0x32787662 ("bvx2") | LZFSE_COMPRESSEDV2_BLOCK_MAGIC | LZFSE compressed block with compressed tables |
| 0x6e787662 ("bvxn") | LZFSE_COMPRESSEDLZVN_BLOCK_MAGIC | LZVN compressed block |

### End-of-stream block

An end-of-stream block is 4 bytes in size and consits of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header*</td> |
| 0 | 4 | "bvx$" | Block signature |

<!-- rumdl-enable MD033 MD056 -->

### Uncompressed block

An uncompressed block is of variable size and consits of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header*</td> |
| 0 | 4 | "bvx-" | Block signature |
| 4 | 4 | | Uncompressed data size |
| <td colspan="4">&nbsp;</td> |
| 8 | ... | | Uncompressed data |

<!-- rumdl-enable MD033 MD056 -->

### LZFSE compressed block with uncompressed tables

A LZFSE compressed block with uncompressed tables (lzfse_compressed_block_header_v1) is of variable
size and consits of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header*</td> |
| 0 | 4 | "bvx1" | Block signature |
| 4 | 4 | | Uncompressed data size |
| 8 | 4 | | Compressed data size |
| 12 | 4 | | Number of literals |
| 16 | 4 | | Number of L, M, D values |
| 20 | 4 | | Number of bytes used to encode literals |
| 24 | 4 | | Number of bytes used to encode matches |
| 28 | 4 | | Unknown (Final accum_nbits for literals stream) |
| 32 | 2 | | First literal state |
| 34 | 2 | | Second literal state |
| 36 | 2 | | Third literal state |
| 38 | 2 | | Fourth literal state |
| 40 | 4 | | Unknown (accum_nbits for the l, m, d stream) |
| 44 | 2 | | L value state |
| 46 | 2 | | M value state |
| 48 | 2 | | D value state |
| 50 | 720 | | [Frequency table](#lzfse_frequency_table) |
| <td colspan="4">&nbsp;</td> |
| 770 | ... | | encoded literals |
| ... | ... | | encoded L, M, D values |

<!-- rumdl-enable MD033 MD056 -->

### LZFSE compressed block with compressed tables

A LZFSE compressed block with compressed tables (lzfse_compressed_block_header_v2)
is of variable size and consits of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header*</td> |
| 0 | 4 | "bvx2" | Block signature |
| 4 | 4 | | Uncompressed data size |
| <td colspan="4">*3 x 64-bit packed fields*</td> |
| 8.0 | 20 bits | | Number of literals |
| 10.4 | 20 bits | | Number of bytes used to encode literals |
| 13.0 | 20 bits | | Number of L, M, D values |
| 15.4 | 3 bits | | Unknown (Final accum_nbits for literals stream) |
| 14.7 | 1 bit | | Unknown (unused) |
| 16.0 | 10 bits | | First literal state |
| 17.2 | 10 bits | | Second literal state |
| 18.4 | 10 bits | | Third literal state |
| 19.6 | 10 bits | | Fourth literal state |
| 21.0 | 20 bits | | Number of bytes used to encode matches |
| 22.4 | 3 bits | | Unknown (accum_nbits for the l, m, d stream) |
| 23.7 | 1 bit | | Unknown (unused) |
| 24.0 | 32 bits | | Block header size |
| 28.0 | 10 bits | | L value state |
| 29.2 | 10 bits | | M value state |
| 30.4 | 10 bits | | D value state |
| 31.6 | 2 bits | | Unknown (unused) |
| <td colspan="4">*If block header size > 32*</td> |
| 32 | ... | | Bit stream containing Huffman encoded [frequency table](#lzfse_frequency_table) |
| <td colspan="4">&nbsp;</td> |
| ... | ... | | encoded literals |
| ... | ... | | encoded L, M, D values |

<!-- rumdl-enable MD033 MD056 -->

> Note that if the block header size is 32 the block does not contain frequency tables.

### LZVN compressed block

A LZVN compressed block is of variable size and consits of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Block header*</td> |
| 0 | 4 | "bvxn" | Block signature |
| 4 | 4 | | Uncompressed data size |
| 8 | 4 | | Compressed data size |
| <td colspan="4">&nbsp;</td> |
| 12 | ... | | [LZVN compressed data](lzvn.md) |

<!-- rumdl-enable MD033 MD056 -->

### LZFSE frequency table {#lzfse_frequency_table}

A LZFSE frequency table consist of 360 16-bit values:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 20 | | Literal run-lengths (L stream) frequency values |
| 20 | 20 | | Match sizes (M stream) frequency values |
| 40 | 64 | | Match distances (D stream) frequency values |
| 104 | 256 | | Literal frequency values |

#### Decoding the Huffman encoded frequency table

TODO: describe how to decode the Huffman encoded frequency table.

```text
5-bits encoded value, special cases 8 and 14

lzfse_freq_nbits_table[32] = {
    2, 3, 2, 5, 2, 3, 2, 8, 2, 3, 2, 5, 2, 3, 2, 14,
    2, 3, 2, 5, 2, 3, 2, 8, 2, 3, 2, 5, 2, 3, 2, 14};
lzfse_freq_value_table[32] = {
    0, 2, 1, 4, 0, 3, 1, -1, 0, 2, 1, 5, 0, 3, 1, -1,
    0, 2, 1, 6, 0, 3, 1, -1, 0, 2, 1, 7, 0, 3, 1, -1};
```
