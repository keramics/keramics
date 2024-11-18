# Universal Disk Image Format (UDIF)

The Universal Disk Image Format (UDIF) (.dmg) is one of the disk image formats
supported natively by Mac OS.
 
## Overview

Known UDIF image types are:

| Identifier | Description
| --- | ---
| UDBZ | bzip2 compressed UDIF
| UDCO | Apple Data Compression (ADC) compressed UDIF
| UDIF | Read-write uncompressed UDIF
| UDRO | Read-only uncompressed UDIF
| UDxx | Uncompressed UDIF
| UDZO | zlib/DEFLATE compressed UDIF
| ULFO | LZFSE compressed UDIF
| ULMO | LZMA compressed UDIF

UDIF images are either uncompressed or compressed.

### Uncompressed image format

An uncompressed UDIF image consist of:

* data
* optional file footer

> Note that an uncompressed UDIF image without file footer is equivalent to a
RAW storage media image (CRawDiskImage).

### Compressed image format

A compressed UDIF image consist of:

* Data fork
* Optional resource fork
* Optional XML plist
* File footer the end of the image file

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | big-endian
| Date and time values | N/A
| Character strings | N/A

The number of bytes per sector is 512.

## File footer

The file footer (also known as resource file or metadata) is 512 bytes in size
and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "koly" | Signature
| 4 | 4 | 4 | Format version
| 8 | 4 | 512 | File footer size in bytes
| 12 | 4 | | [Image flags](#image_flags)
| 16 | 8 | | Unknown (RunningDataForkOffset)
| 24 | 8 | | Data fork offset, where the offset is relative from the start of the image file
| 32 | 8 | | Data fork size
| 40 | 8 | | Resource fork offset, where the offset is relative from the start of the image file
| 48 | 8 | | Resource fork size
| 56 | 4 | | Unknown (SegmentNumber)
| 60 | 4 | | Number of segments, which contains 0 if not set
| 64 | 16 | | Segment identifier, which contains an UUID
| 80 | 4 | | Data checksum type
| 84 | 4 | | Data checksum size, as number of bits
| 88 | 128 | | Data checksum
| 216 | 8 | | XML plist offset, where the offset is relative from the start of the image file
| 224 | 8 | | XML plist size
| 232 | 120 | | Unknown (Reserved)
| 352 | 4 | | Master checksum type
| 356 | 4 | | Master checksum size, as number of bits
| 360 | 128 | | Master checksum
| 488 | 4 | | [Image type](#image_type) (or variant)
| 492 | 8 | | Number of sectors
| 500 | 4 | | Unknown (reserved)
| 504 | 4 | | Unknown (reserved)
| 508 | 4 | | Unknown (reserved)

> Note that the XML plist size can be 0, such as in an UDIF stub (UDxx) image.

### <a name="image_flags"></a>Image flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | kUDIFFlagsFlattened | Unknown (flattened?)
| | |
| 0x00000004 | kUDIFFlagsInternetEnabled | Unknown (internet enabled?)

### <a name="image_types"></a>Image types

| Value | Identifier | Description
| --- | --- | ---
| 1 | kUDIFDeviceImageType | Device image
| 2 | kUDIFPartitionImageType | Paritition image

## XML plist

TODO: complete section

```
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
        <key>resource-fork</key>
        <dict>
                <key>blkx</key>
                <array>
                        <dict>
                                <key>Attributes</key>
                                <string>0x0050</string>
                                <key>CFName</key>
                                <string>Protective Master Boot Record (MBR : 0)</string>
                                <key>Data</key>
                                <data>
                                bWlzaAAAAAEAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAA
                                AAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAIAAAAgQfL6MwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAACgAAABQAAAAMAAAAAAAAAAAAAAAAAAAABAAAA
                                AAAAIA0AAAAAAAAAH/////8AAAAAAAAAAAAAAAEAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAA=
                                </data>
                                <key>ID</key>
                                <string>-1</string>
                                <key>Name</key>
                                <string>Protective Master Boot Record (MBR : 0)</string>
                        </dict>
                        ...
                </array>
                <key>plst</key>
                <array>
                        <dict>
                                <key>Attributes</key>
                                <string>0x0050</string>
                                <key>Data</key>
                                <data>
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAQAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                                AAAAAAAAAAAA
                                </data>
                                <key>ID</key>
                                <string>0</string>
                                <key>Name</key>
                                <string></string>
                        </dict>
                </array>
        </dict>
</dict>
</plist>
```

The XML plist contains the following key-value pairs:

| Identifier | Description
| --- | ---
| resource-fork | dictionary

### XML plist resource-fork dictionary

The resource-fork dictionary contains the following key-value pairs:

| Identifier | Description
| --- | ---
| blkx | array of dictionaries
| plst | array of dictionaries

### XML plist blkx array entry

A blkx array entry contains the following key-value pairs:

| Identifier | Description
| --- | ---
| Attributes | string that contains a hexadecimal formatted integer value
| CFName | string
| Data | string that contains base-64 encoded data of a [block table](#udif_block_table)
| ID | string that contains a decimal formatted integer value
| Name | string

## <a name="udif_block_table"></a>Block table

The block table (BLKXTable) is of variable size and consists of:

* block table header
* block table entries

### The block table header

The block table header is 204 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "mish" | Signature
| 4 | 4 | 1 | Format version
| 8 | 8 | | Start sector, which contains the sector number relative to the start of the media data
| 16 | 8 | | Number of sectors
| 24 | 8 | | Unknown (DataOffset), which seems to be always 0
| 32 | 4 | | Unknown (BuffersNeeded)
| 36 | 4 | | Unknown (BlockDescriptors). Does this value correspond to the number of block table entries?
| 40 | 4 | 0 | Unknown (reserved)
| 44 | 4 | 0 | Unknown (reserved)
| 48 | 4 | 0 | Unknown (reserved)
| 52 | 4 | 0 | Unknown (reserved)
| 56 | 4 | 0 | Unknown (reserved)
| 60 | 4 | 0 | Unknown (reserved)
| 64 | 4 | | Checksum type
| 68 | 4 | | Checksum size
| 72 | 128 | | Checksum
| 200 | 4 | | Number of entries

### Block table entry

The block table entry (BLKXChunkEntry) is 40 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | [Entry type](#udif_block_table_entry_types)
| 4 | 4 | | Unknown (comment)
| 8 | 8 | | Start sector, which contains the sector number relative to the start of the start sector of the block table
| 16 | 8 | | Number of sectors
| 24 | 8 | | Data offset, which contains the byte offset relative to the start of the UDIF image file
| 32 | 8 | | Data size, which contain the number of bytes of data stored, which is 0 for sparse data

#### <a name="udif_block_table_entry_types"></a>UDIF block table entry types

| Value | Identifier | Description 
| --- | --- | ---
| 0x00000000 | | Unknown (sparse)
| 0x00000001 | | Uncompressed (raw) data
| 0x00000002 | | Sparse (used for Apple_Free)
| | |
| 0x7ffffffe | | Comment
| | |
| 0x80000004 | | ADC compressed data
| 0x80000005 | | zlib compressed data
| 0x80000006 | | bzip2 compressed data 
| 0x80000007 | | LZFSE compressed data 
| 0x80000008 | | LZMA compressed data
| | |
| 0xffffffff | | Block table entries terminator

## UDIF comment

TODO: complete section

## UDIF data fork

TODO: complete section

## UDIF resource fork

TODO: complete section

## Notes

```
Is the maximum compressed chunk size 2048 sectors?
```

```
Comment seems to reference compressed data but has no size or number of sectors
value.
```
