# xz compressed data format

The xz (or LMZA2) format is a stream and block container format in which one or more filters are
applied to the data. The filter compresses the data with the LZMA algorithm, adding support for
uncompressed chunks and a way to reset the LZMA state and dictionary between chunks.

The symbols (literals and matches) of the (LZMA2) chunks are decoded with the same range coder and
probability models as used in [LZMA1 compressed data](lzma.md), with the difference that a chunk
decodes a known number of bytes and does not use the
[end of payload marker](lzma.md#end_of_payload_marker).

## Overview

xz compressed data consist of:

* [Stream Header](#stream_header)
* zero or more [blocks](#block)
* [Index](#index)
* [Stream Footer](#stream_footer)

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |

## Stream Header {#stream_header}

The stream header is 12 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 6 | "\xfd\x37\x7a\x58\x5a\x00" | Signature |
| 6 | 2 | | [Stream Flags](#stream_flags) |
| 8 | 4 | | CRC32 of the Stream Flags |

### Stream Flags {#stream_flags}

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 8 bits | 0 | Unknown (Reserved, must be 0) |
| 1.0 | 4 bits | | [Checksum type](#checksum_type) |
| 1.4 | 4 bits | 0 | Unknown (Reserved, must be 0) |

### Checksum type {#checksum_type}

| Checksum type | Size of checksum | Description |
| --- | --- | --- |
| 0x00 | 0 bytes | None |
| 0x01 | 4 bytes | CRC32 |
| 0x02 | 4 bytes | Unknown (Reserved) |
| 0x03 | 4 bytes | Unknown (Reserved) |
| 0x04 | 8 bytes | CRC64 |
| 0x05 | 8 bytes | Unknown (Reserved) |
| 0x06 | 8 bytes | Unknown (Reserved) |
| 0x07 | 16 bytes | Unknown (Reserved) |
| 0x08 | 16 bytes | Unknown (Reserved) |
| 0x09 | 16 bytes | Unknown (Reserved) |
| 0x0a | 32 bytes | SHA-256 |
| 0x0b | 32 bytes | Unknown (Reserved) |
| 0x0c | 32 bytes | Unknown (Reserved) |
| 0x0d | 64 bytes | Unknown (Reserved) |
| 0x0e | 64 bytes | Unknown (Reserved) |
| 0x0f | 64 bytes | Unknown (Reserved) |

## Variable-length integer {#variable_length_integer}

Sizes that are typically small values are stored with a variable-length integer (VLI). The value is
limited to 63 bits and uses 1 to 9 bytes, where:

* numbers in the range \[0, 127] occupy 1 byte;
* numbers outside the range \[0, 127] occupy 2 or more bytes, of which all bytes except the last
  have their highest bit (0x80) set.

The bits of the value are stored in groups of seven bits, least significant group first:

```text
value = 0
for index, byte in VLI bytes
    value |= (byte & 0x7f) << (index * 7)
    if byte & 0x80 == 0
       break
```

A multi-byte VLI must **not** end with a 0-byte value (or non-canonical encoding).

## Block {#block}

A block consists of:

* [Block Header](#block_header)
* [Compressed Data](#compressed_data), which for the filter is one or more
  [Chunks](#lzma2_chunk), terminated by an end-of-blocks marker
* [Block padding](#block_padding)
* [Checksum](#checksum), whose size depends on the [checksum type](#checksum_type)

The total size of a block (block header, compressed data, block padding, and checksum) is always a
multiple of 4 bytes. The first byte of a block header must **not** be 0, as a value of 0 indicates
the start of the [index](#index).

### Block Header {#block_header}

The block header is ((first byte value) + 1) * 4 bytes in size, which must be in the range
\[8, 1024] bytes. It consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Block Header Size, where `size = (value + 1) * 4` |
| 1 | 1 | | [Block Flags](#block_flags) |
| 2 | 1 - 9 | | [compressed size](#block_compressed_size), if defined by the block Flags |
| ... | 1 - 9 | | [Uncompressed size](#block_uncompressed_size), if defined by the block Flags |
| ... | ... | | [List of Filter Flags](#filter_flags) |
| ... | ... | | Header padding, null bytes up to the block header size |
| ... | 4 | | CRC32 of the block header except the CRC32 value itself |

### Block Flags {#block_flags}

The block flags are 2 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 1.0 | 2 bits | | Number of filters, where `number = value + 1` in range \[1, 4] |
| 1.2 | 4 bits | 0 | Unknown (Reserved, must be 0) |
| 1.6 | 1 bit | | Compressed size present (1) |
| 1.7 | 1 bit | | Uncompressed size present (1) |

> Note that Keramics currently only supports a single filter chain consisting of the filter and
> rejects blocks with more than one filter.

### compressed size {#block_compressed_size}

The compressed Size value is a [variable-length integer](#variable_length_integer) containing the
size, in bytes, of the Compressed Data value. The value must be non-zero, and if it does not match
the size of the decoded compressed data, the decoding must indicate an error.

### Uncompressed size {#block_uncompressed_size}

The uncompressed size value is a [variable-length integer](#variable_length_integer) containing the
size, in bytes, of the uncompressed data of the block. If the value does not match the actual
uncompressed size, the decoding must indicate an error.

### List of Filter Flags {#filter_flags}

The List of Filter Flags consists of one filter definition per filter indicated by the
[block flags](#block_flags). Each filter definition consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| ... | 1 - 9 | | [Filter identifier](#filter_identifier), which consist of a [variable-length integer](#variable_length_integer) |
| ... | 1 - 9 | | Size of Filter Properties, which consist of a [variable-length integer](#variable_length_integer) |
| ... | ... | | [Filter Properties](#filter_properties) |

### Filter identifier {#filter_identifier}

| Filter identifier | Description |
| --- | --- |
| 0x21 | LZMA2 |

### Filter Properties {#filter_properties}

The Filter Properties of the filter is one byte:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 6 bits | | Dictionary Size |
| 0.6 | 2 bits | 0 | Unknown (Reserved, must be 0) |

The Dictionary Size is encoded with a one-bit mantissa and a 5-bit exponent. The smallest
dictionary size is 4 KiB (value 0) and the biggest is 4 GiB - 1 (value 40):

```python
if dictionary_value > 40:
    error "Dictionary size too big"

if dictionary_value == 40:
    dictionary_size = 0xffffffff
else:
    dictionary_size = (
        2 | (dictionary_value & 1)) << ((dictionary_value / 2) + 11
    )
```

### Compressed Data {#compressed_data}

The Compressed Data of a block which uses a single filter consists of one or more
[chunks](#lzma2_chunk), where the last chunk of the Compressed Data is the end-of-blocks marker (a
single 0-byte value).

### Chunk {#lzma2_chunk}

A chunk consists of:

* a [control byte](#lzma2_control)
* 1 or 2 size values
* chunk data

#### Uncompressed chunk

An [uncompressed chunk](#lzma2_uncompressed_chunk) consists of:

* a [control byte](#lzma2_control) in the range 0x01 - 0x02
* 1 size value
* uncompressed data

#### LZMA compressed chunk

An [LZMA compressed chunk](#lzma2_lzma_chunk) consists of:

* a [control byte](#lzma2_control) in the range 0x80 - 0xff
* 2 size values
* optional LZMA2 properties byte
* LZMA encoded data

### Control bytes {#lzma2_control}

The control byte determines the type of chunk and which LZMA state must be reset before decoding
the chunk:

| Control byte | Description |
| --- | --- |
| 0x00 | End-of-blocks marker, the last chunk of a block (no size values and no chunk data) |
| 0x01 | Uncompressed chunk with dictionary reset |
| 0x02 | Uncompressed chunk without dictionary reset |
| 0x03 - 0x7f | Invalid |
| 0x80 - 0x9f | LZMA compressed chunk, where the LZMA state is kept |
| 0xa0 - 0xbf | LZMA compressed chunk, where the LZMA state is reset |
| 0xc0 - 0xdf | LZMA compressed chunk, where the LZMA state is reset and new [LZMA2 properties](#lzma2_properties) |
| 0xe0 - 0xff | LZMA compressed chunk, where the LZMA state and dictionary are reset, and new [LZMA2 properties](#lzma2_properties) |

An LZMA state reset resets the range coder, the LZMA state value, the distance history (rep0 to
rep3), the last decoded byte, and all probability models. A dictionary reset sets the start of the
dictionary region to the start of the chunk's output data; a match decoded in a subsequent chunk
can only reference data from the current dictionary region onwards.

> Note that the first chunk of a block must perform a dictionary reset (control byte 0x01 or
> 0xe0 - 0xff); all other control byte values are only valid for subsequent chunks. A chunk without
> new properties (control byte 0x80 - 0xbf) may therefore not be the first chunk of a block, as no
> properties are defined before it.

### LZMA2 properties {#lzma2_properties}

The LZMA2 properties of an LZMA compressed chunk is one byte and is decoded exactly as the
[lZMA1 properties](lzma.md#properties) for (lc, lp, pb; lc + lp <= 4, pb <= 4).

An LMZA compressed chunk without new properties (control byte 0x80 - 0xbf) reuses the LZMA2
properties of the most recent LMZA compressed chunk of the block which defined them. The properties
are therefore always defined before the first LMZA compressed chunk of a block is decoded, and can
be changed at any point by an LMZA compressed chunk with a control byte in the range \[0xc0, 0xff].

### Uncompressed chunk {#lzma2_uncompressed_chunk}

An uncompressed chunk consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | 0x01 or 0x02 | [Control byte](#lzma2_control) |
| 1 | 2 | | Uncompressed size, where `size = value + 1`, in range \[1, 65536] |
| 3 | ... | | Uncompressed data, in bytes |

The uncompressed size and the compressed size of an uncompressed chunk are equal.

### LZMA compressed chunk {#lzma2_lzma_chunk}

An LZMA compressed chunk consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | 0x80 - 0xff | [Control byte](#lzma2_control) |
| 1 | 2 | | [Uncompressed size](#lzma2_uncompressed_size), low 16 bits |
| 3 | 2 | | [Compressed size](#lzma2_compressed_size), where `size = value + 1` |
| 5 | 1 | | [LZMA properties](#lzma2_properties), if the control byte is in the range \[0xc0, 0xff] |
| 6 | ... | | LZMA encoded data, compressed size bytes |

#### Uncompressed size {#lzma2_uncompressed_size}

The uncompressed size of an LZMA compressed chunk is the low 16 bits of the size value combined
with the low 5 bits of the control byte:

```python
uncompressed_size = (((control_byte & 0x1f) << 16) | (size_value)) + 1
```

The maximum uncompressed size of an LZMA compressed chunk is 2 MiB (2^21 - 1).

#### compressed size {#lzma2_compressed_size}

The compressed size of an LZMA compressed chunk is a 16-bit value, where `size = value + 1`.
The maximum compressed size of an LZMA compressed chunk is 64 KiB (2^16 - 1).

An LZMA compressed chunk is finished when its uncompressed size has been decoded, the range coder
is finished, i.e. the code value is zero after a final normalization, and all of its Compressed
Size bytes have been consumed. The [end of payload marker](lzma.md#end_of_payload_marker) is not
used in an LZMA compressed chunk.

### Block padding {#block_padding}

The block padding consists of 0 to 3 0-byte values, added so that the total size of the block
(block header, compressed data, block padding, and checksum) is a multiple of 4 bytes (32-bit).

### Checksum {#checksum}

The checksum is calculated from the original uncompressed data of the block. Its type and size
are determined by the [checksum type](#checksum_type) in the [Stream Flags](#stream_flags) of
the Stream Header. For a CRC32 checksum, the checksum is stored as an unsigned 32-bit
little-endian integer.

## Index {#index}

The index serves to verify that all blocks in a Stream have been processed, to find out
the uncompressed size of the Stream, and to quickly access the beginning of any block.
It consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | 0 | Index indicator (must be 0) |
| 1 | 1 - 9 | | [Number of records](#index_number_of_records), which consists of a [variable-length integer](#variable_length_integer) |
| ... | ... | | [List of records](#index_records), one record per block |
| ... | 0 - 3 | | Index padding, null bytes to make the index a multiple of four bytes |
| ... | 4 | | CRC32 of everything in the index except the CRC32 field itself, little-endian |

### Number of records {#index_number_of_records}

The Number of records must match the number of blocks of the Stream, otherwise the
decoding must indicate an error.

### List of records {#index_records}

Each record in the List of records describes one block, in the order in which the blocks
appear in the Stream:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| ... | 1 - 9 | | Unpadded size: the size of the block header, compressed data, and checksum fields (excluding the block padding), a [variable-length integer](#variable_length_integer) |
| ... | 1 - 9 | | Uncompressed size of the block, which consists of a [variable-length integer](#variable_length_integer) |

## Stream Footer {#stream_footer}

The Stream Footer is 12 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | CRC32 of the Backward Size and Stream Flags fields, little-endian |
| 4 | 4 | | [Backward Size](#backward_size), little-endian |
| 8 | 2 | | [Stream Flags](#stream_flags) |
| 10 | 2 | "\x59\x5a" | Signature |

The Stream Flags of the Stream Footer is a copy of the Stream Flags of the Stream Header.
If the two do not match, the decoding must indicate an error.

### Backward Size {#backward_size}

The Backward Size indicates the size of the index field as a multiple of four bytes,
with a minimum size of four bytes:

```text
index_size = (backward_size + 1) * 4
```

If the calculated size does not match the actual size of the index field, the decoding
must indicate an error.

## Decoding

The main loop of the xz decoder is as described below:

```text
read and verify the Stream Header (Signature, Stream Flags, CRC32)

loop (until the byte at the current offset is 0x00, the index indicator)
{
    read and verify the block header (size, flags, sizes, filters, padding, CRC32)
    decode the chunks of the Compressed Data of the block
    verify the stored block sizes (if defined) against the decoded sizes
    read and verify the block padding
    read and verify the checksum (if defined) against the uncompressed data of the block
    remember the unpadded size and the uncompressed size of the block,
    as a record for the index
}

read and verify the index (indicator, records, padding, CRC32),
verifying the records against the decoded blocks
read and verify the Stream Footer (CRC32, Backward Size, Stream Flags, Signature)
```

The chunks of a block are decoded as described below:

```text
properties     = undefined
dictionary     = reset required (no dictionary region started yet)
dictionary_start = start of the output data of the block

loop
{
    read the Control byte
    if the Control byte is 0x00
      break (end-of-blocks marker)

    if the Control byte is 0x01 or 0xe0 - 0xff
      dictionary = reset required
    else if dictionary != reset required
      error "dictionary reset required"

    if an uncompressed chunk (Control 0x01 - 0x02)
      read the Size field (16 bits, value + 1)
      if dictionary == reset required
        dictionary_start = current output offset
        dictionary = reset done
      copy the data of Size bytes to the output
    else (an LZMA compressed chunk, Control 0x80 - 0xff)
      read the uncompressed size (low five bits of the Control + 16 bits, value + 1)
      read the compressed size (16 bits, value + 1)

      if the Control byte is 0xc0 - 0xff (new properties)
        read the LZMA properties byte
        reset the LZMA state and properties
        properties = defined
      else if properties == undefined
        error "properties required"
      else if the Control byte is 0xa0 - 0xff (state reset)
        reset the LZMA state

      if dictionary == reset required
        dictionary_start = current output offset
        dictionary = reset done

      decode an LZMA stream of exactly uncompressed size bytes,
      using the defined LZMA properties and the LZMA state,
      where a match can only reference data from dictionary_start onwards,
      and where the end of payload marker is not used

      verify that the number of decoded bytes equals the uncompressed size,
      and that exactly compressed size bytes were consumed
}
```

## References

* [The .xz file format (version 1.2.1)](https://tukaani.org/xz/xz-file-format.txt)
* [Wikipedia: LZMA](https://en.wikipedia.org/wiki/LZMA)
