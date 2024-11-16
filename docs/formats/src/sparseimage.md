# Mac OS sparse image (.sparseimage) format

The Mac OS sparse image (.sparseimage) format is one of the disk image formats
supported natively by Mac OS.

## Overview

A sparse disk image consists of:

* header data
* bands data

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | big-endian
| Date and time values | N/A
| Character strings | N/A

The number of bytes per sector is 512.

## Header data

The header data is 4096 bytes in size and consist of:

* file header
* band numbers array
* trailing data, which should be filled with 0-byte values

### File header

The file header is 64 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "sprs" | Signature
| 4 | 4 | | Unknown (format version?), seen 3
| 8 | 4 | | Number of sectors in band
| 12 | 4 | | Unknown, seen 1
| 16 | 4 | | The media data size in sectors
| 20 | 12 | 0 | Unknown (0-byte values)
| 32 | 4 | | Unknown
| 36 | 28 | 0 | Unknown (0-byte values)

### Band numbers array

The band numbers array consists of:

* one or more band numbers

#### Band number

A band number is 4 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Band number, where 0 indicates a sparse range and any other value refers to a location in the media data.

Where the corresponding media offset can be calculated as following:

```
media_offset = ( band_number - 1 ) * sectors_per_band * 512
```

The offset of band data can be calculated as following:

```
band_data_offset = 4096 + ( array_index * sectors_per_band * 512 )
```

For example if the first array entry contains a band number of 4, then the
band data is located at offset 4096 and the corresponding media offset is:
`3 * sectors_per_band * 512`.
