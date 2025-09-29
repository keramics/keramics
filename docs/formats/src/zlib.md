# Zlib compressed data

Zlib compression is commonly used in file formats. The zlib compressed data
format, as defined in RFC1950, allows for multiple techniques but only the
Deflate compression method, a variation of LZ77, is used.

## Overview

Zlib compressed data consist of:

* data header
* compressed data
* Adler-32 checksum of the uncompressed data

### Characteristics

| Characteristics | Description
| --- | --- 
| Byte order | big-endian

## Data header

The data header is 2 or 6 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4"> *The bit values are stored a 8-bit values*
| 0.0 | 4 bits | | [Compression method](#compression_method)
| 0.4 | 4 bits | | [Compression information](#compression_information)
| <td colspan="4"> *Flags*
| 1.0 | 5 bits | | Check bits
| 1.5 | 1 bit | | Preset dictionary flag
| 1.6 | 2 bits | | [Compression level](#compression_level). The compression level is used mainly for re-compression
| <td colspan="4"> *If the dictionary identifier flag is set*
| 2 | 4 | | Preset dictionary identifier, which contains an Adler-32 used to identifier the preset dictionary
| <td colspan="4"> *Common*
| ... | ... | | Compressed data
| ... | 4 | | Checksum, which contains an Adler-32 of the compressed data

The check bits value must be such that when the first 2 bytes are represented
as a 16-bit unsigned integer in big-endian byte order the value is a multiple
of 31, such that:

```
( ( first x 256 ) + second ) mod 31 = 0
```

### <a name="compression_method"></a>Compression method

| Value | Identifier | Description
| --- | --- | ---
| 8 | | Deflate (RFC1951), with a maximum window size of 32 KiB
| | |
| 15 | | Reserved for additional header data

> Note that RFC1950 only defines 8 as a valid compression method.

### <a name="compression_information"></a>Compression information

The value of the compression information is dependent on the compression method.

#### Compression information - compression method 8 (Deflate)

For compression method 8 (Deflate) the compression information contains the
base-2 logarithm of the LZ77 window size minus 8.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 4 bits | | Window size, which consists of a base-2 logarithm (2n), with a maximum value of 7 (32 KiB)

To determine the corresponding window size:

```
1 << ( 7 + 8 )
```

E.g. a compression information value of 7 indicates a 32768 bytes window size.
Values larger than 7 are not allowed according to RFC1950 and thus the maximum
window size is 32768 bytes.

### <a name="compression_level"></a>Compression level

| Value | Identifier | Description
| --- | --- | ---
| 0 | | Fastest
| 1 | | Fast
| 2 | | Default
| 3 | | Slowest, maximum compression

## Compressed data

### Deflate compressed data

The deflate compressed data consists of one or more deflate compressed blocks.
Each block consists of:

* block header
* block data

> Note that a block can reference uncompressed data that is stored in a previous
> block.

#### Block header

The block header is 3 bits in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 bit | | Last block (in stream) marker, where 1 represents the last block and 0 otherwise
| 0.1 | 2 bits | | [Block type](#deflate_block_types)

#### <a name="deflate_block_types"></a>Block types

| Value | Identifier | Description
| --- | --- | ---
| 0 | | Uncompressed (or stored) block
| 1 | | Fixed Huffman compressed block
| 2 | | Dynamic Huffman compressed block
| 3 | | Reserved (not used)

#### Uncompressed block data

The uncompressed block data is variable of size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.3 | 5 bits | | Empty values (not used)
| 1 | 2 | | Uncompressed data size
| 3 | 2 | | Copy of uncompressed data size, which contains a 1s complement of the uncompressed data size
| 5 | ... | | Uncompressed data

The uncompressed data size can range between 0 and 65535 bytes.

#### Huffman compressed block data

The uncompressed block data is variable of size and consists of:

* Optional dynamic Huffman table
* Encoded bit-stream
* End-of-stream (or end-of-block or end-of-data) marker

##### Dynamic Huffman table

The dynamic Huffman table consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.3 | 5 bits | | Number of literal codes, which is value + 257. The number of literal codes must be smaller than 286
| 1.0 | 5 bits | | Number of distance codes, which is value + 1. The number of distance codes must be smaller than 30
| 1.5 | 4 bits | | The number of Huffman codes for the code sizes, which is value + 4
| 2.1 | ... | | The code sizes
| ... | ... | | Huffman encoded stream of the Huffman codes for the literals
| ... | ... | | Huffman encoded stream of the Huffman codes for the distances

A single code size value is 3 bits of size. A value of 0 means the code size is
not used in the Huffman encoding of the literal and distance codes.

The codes size values are stored in the following sequence:

```
16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
```

The first value applies to a code size of 16, the second to 17, etc. Code sizes
that are not stored default to 0.

The code size values are used to construct the code sizes Huffman table. This
must be a complete Huffman table which is used to decode the literal and
distance codes. The corresponding codes size Huffman encoding is defined as:

| Value | Identifier | Description
| --- | --- | ---
| 0 - 15 | | Represents a code size of 0 - 15
| 16 | | Copy the previous code size 3 - 6 times. The next 2 bits indicate repeat length (0 = 3, ... , 3 = 6), e.g. codes 8, 16 (+2 bits 11), 16 (+2 bits 10) will expand to 12 code lengths of 8 (1 + 6 + 5)
| 17 | | Repeat a code length of 0 for 3 - 10 times (3 bits of length)
| 18 | | Repeat a code length of 0 for 11 - 138 times (7 bits of length)

Both the literal and distance Huffman codes are stored Huffman encoded using
the code sizes Huffman table. Code sizes that are not stored default to 0.
The code size for the literal code 256 (end-of-block) should be set and thus
not 0.

##### Encoded bit-stream

The encoded bit-stream is stored in 8-bit integers, where bit values are stored
back-to-front. So that 3 least-significant bits (LSB) would represent a 3-bit
value at the start of the -stream. Note that the LSB of the 3-bit value is the
LSB of the byte value.

Deflate uses a Huffman tree of 288 Huffman codes (or symbols) where the values:

* 0 - 255; represent the literal byte values: 0 - 255
* 256: represents the end of (compressed) stream (or block)
* 257 - 285 (combined with extra-bits): represent a (size, offset) tuple (or match length) of 3 - 258 bytes
* 286, 287: are not used (reserved) and their use is considered illegal although the values are still part of the tree

This document refers to this Huffman tree as the literals Huffman tree.

The bits in the encoded bit-stream correspond to values in the literals
Huffman tree. If a symbol is found that represents a compression size and
offset tuple (or match length code) the bits following the literals symbol
contains a distance (Huffman) code. The match length coedes might require
additional (or extra) bits to store the length (or size).

The distances Huffman tree contains space for 32 symbols. See section
[Distance codes](#deflate_distance_codes). The distance code might require
additional (or extra) bits to store the distance.

##### Literal codes

The literal codes consist of:

| Value | Identifier | Description
| --- | --- | ---
| 0x00 – 0xff | | literal byte values
| 0x100 | | end-of-block marker
| <td colspan="3"> *0 additional bits*
| 0x101 | | Size of 3
| 0x102 | | Size of 4
| 0x103 | | Size of 5
| 0x104 | | Size of 6
| 0x105 | | Size of 7
| 0x106 | | Size of 8
| 0x107 | | Size of 9
| 0x108 | | Size of 10
| <td colspan="3"> *1 additional bit*
| 0x109 | | Size of 11 to 12
| 0x10a | | Size of 13 to 14
| 0x10b | | Size of 15 to 16
| 0x10c | | Size of 17 to 18
| <td colspan="3"> *2 additional bits*
| 0x10d | | Size of 19 to 22
| 0x10e | | Size of 23 to 26
| 0x10f | | Size of 27 to 30
| 0x110 | | Size of 31 to 34
| <td colspan="3"> *3 additional bits*
| 0x111 | | Size of 35 to 42
| 0x112 | | Size of 43 to 50
| 0x113 | | Size of 51 to 58
| 0x114 | | Size of 59 to 66
| <td colspan="3"> *4 additional bits*
| 0x115 | | Size of 67 to 82
| 0x116 | | Size of 83 to 98
| 0x117 | | Size of 99 to 114
| 0x118 | | Size of 115 to 130
| <td colspan="3"> *5 additional bits*
| 0x119 | | Size of 131 to 162
| 0x11a | | Size of 163 to 194
| 0x11b | | Size of 195 to 226
| 0x11c | | Size of 227 to 257
| <td colspan="3"> *0 additional bits*
| 0x11d | | Size of 258

##### <a name="deflate_distance_codes"></a>Distance codes

The distance codes consist of:

| Value | Identifier | Description
| --- | --- | ---
| 0 | distance of 1
| 1 | distance of 2
| 2 | distance of 3
| 3 | distance of 4
| <td colspan="3"> *1 additional bit*
| 4 | distance of 5 - 6
| 5 | distance of 7 - 8
| <td colspan="3"> *2 additional bits*
| 6 | distance of 9 - 12
| 7 | distance of 13 - 16
| <td colspan="3"> *3 additional bits*
| 8 | distance of 17 - 24
| 9 | distance of 25 - 32
| <td colspan="3"> *4 additional bits*
| 10 | distance of 33 - 48
| 11 | distance of 49 - 64
| <td colspan="3"> *5 additional bits*
| 12 | distance of 65 - 96
| 13 | distance of 97 - 128
| <td colspan="3"> *6 additional bits*
| 14 | distance of 129 - 192
| 15 | distance of 193 - 256
| <td colspan="3"> *7 additional bits*
| 16 | distance of 257 - 384
| 17 | distance of 385 - 512
| <td colspan="3"> *8 additional bits*
| 18 | distance of 513 - 768
| 19 | distance of 769 - 1024
| <td colspan="3"> *9 additional bits*
| 20 | distance of 1025 - 1536
| 21 | distance of 1537 - 2048
| <td colspan="3"> *10 additional bits*
| 22 | distance of 2049 - 3072
| 23 | distance of 3073 - 4096
| <td colspan="3"> *11 additional bits*
| 24 | distance of 4097 - 6144
| 25 | distance of 6145 - 8192
| <td colspan="3"> *12 additional bits*
| 26 | distance 8193 - 12288
| 27 | distance 12289 - 16384
| <td colspan="3"> *13 additional bits*
| 28 | distance 16385 - 24576
| 29 | distance 24577 - 32768
| <td colspan="3"> *other*
| 30-31 | not used, reserved and illegal but still part of the tree.

TODO: complete this section

#### Additional bits

The additional bits are stored in big-endian (MSB first) and indicate the index
into the corresponding array of size values (or base size + additional size).

| Value | Identifier | Description
| --- | --- | ---
| <td colspan="3"> *0 additional bits*
| 0 | | Offset of 1
| 1 | | Offset of 2
| 2 | | Offset of 3
| 3 | | Offset of 4
| <td colspan="3"> *1 additional bit*
| | |

TODO: complete this section

#### Decompression

The decompression in pseudo code:

```
if( block_header.type == HUFFMANN_FIXED )
{
    initialize the fixed Huffman trees
}

do
{
    read block_header from input stream

    if( block_header.type == UNCOMPRESSED )
    {
        align with next byte
        read and check block_header.size and block_header.size_copy
        read data of block_header.size
    }
    else
    {
        if( block_header.type == HUFFMANN_DYNAMIC )
        {
            read the dynamic Huffman trees (see subsection below)
        }
        loop (until end of block code recognized)
        {
            decode literal/length value from input stream
            if( value < 256 )
            {
                copy value (literal byte) to output stream
            }
            else if value = end of block (256)
            {
                 break from loop
             }
             else (value = 257..285)
             {
                 decode distance from input stream

                 move backwards distance bytes in the output
                 stream, and copy length bytes from this
                 position to the output stream.
            }
        }
    }
}
while( block_header.last_block_flag == 0 );
```

### Adler-32 checksum

Zlib provides a highly optimized version of the algorithm provided below.

```
uint32_t adler32(
          uint8_t *buffer,
          size_t buffer_size,
          uint32_t previous_key )
{
    size_t buffer_iterator = 0;
    uint32_t lower_word    = previous_key & 0xffff;
    uint32_t upper_word    = ( previous_key >> 16 ) & 0xffff;

    for( buffer_iterator = 0;
         buffer_iterator < buffer_size;
         buffer_iterator++ )
    {
        lower_word += buffer[ buffer_iterator ];
        upper_word += lower_word;

        if( ( buffer_iterator != 0 )
         && ( ( buffer_iterator % 0x15b0 == 0 )
          ||  ( buffer_iterator == buffer_size - 1 ) ) )
        {
            lower_word = lower_word % 0xfff1;
            upper_word = upper_word % 0xfff1;
        }
    }
    return( ( upper_word << 16 ) | lower_word );
}
```

## References

* [RFC1950 - ZLIB Compressed Data Format Specification](http://www.ietf.org/rfc/rfc1950.txt)
* [RFC1951 - DEFLATE Compressed Data Format Specification](http://www.ietf.org/rfc/rfc1951.txt)
