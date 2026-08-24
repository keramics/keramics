# Universal Disk Image Format (UDIF)

The Universal Disk Image Format (UDIF) (.dmg) is one of the disk image formats supported natively
by Mac OS. UDIF supersedes the New Disk Image Format (NDIF) format and was introduced in Max OS
X 10.0 (Cheetah).

Mac OS referers to the UDIF image format as CUDIFEncoding.

## Overview

An UDIF image can consists of one or more segment files, where:

* the first segment file is named: "image.dmg"
* successive segment files are named: "image.###.dmgpart", where "###" represents a numeric value
  starting with 2 with 0 padding, e.g. "image.002.dmgpart". Segment files after 999 are assumed to
  be named without the 0 padding, e.g. "image.1234.dmgpart".

The data forks of the segment files are used as a contiguous data stream. A compressed block can
be stored across multiple segment files.

Only the first segment file contains a resource fork or XML plist.

Known UDIF image types are:

| Identifier | Description |
| --- | --- |
| UDBZ | bzip2 compressed UDIF |
| UDCO | Apple Data Compression (ADC) compressed UDIF |
| UDIF | Read-write uncompressed UDIF |
| UDRO | Read-only uncompressed UDIF |
| UDxx | Uncompressed UDIF |
| UDZO | zlib/DEFLATE compressed UDIF |
| ULFO | LZFSE compressed UDIF |
| ULMO | LZMA compressed UDIF |

UDIF images can be encrypted. An encrypted UDIF image consists of one of more UDIF segment files,
where each segment file uses a [Encrypted Encoding container](cdsaencr.md) with its own key
protectors.

### Terminology

| Term | Description |
| --- | --- |
| Flattened image | The disk image is a self-contained, a resource fork is stored within the image |
| Unflattened image | The disk image uses the file system to store a resource fork |

### Image formats

Known types of UDIF segment files are:

* Uncompressed segment file
* Compressed segment file
* Encrypted segment file

#### Uncompressed segment file format

An uncompressed UDIF segment file consist of:

* Image data
* [File footer](#file_footer) at the end of the file

> Note that an uncompressed UDIF image without file footer is equivalent to a RAW storage media
> image (CRawDiskImage).

#### Compressed segment file format

A compressed UDIF segment file consist of:

* Data fork, containing the image data
* Optional XML plist
* Optional resource fork
* [File footer](#file_footer) at the end of the file

#### Encrypted Encoding version 1 encrypted UDIF segment file

An Encrypted Encoding version 1 encryped UDIF segment file consists of:

* Data fork, containing encrypted UDIF data
* [Encrypted Encoding container footer](cdsaencr.md#encypted_container_footer) at the end of the
  file

> Note that the encrypted UDIF data can contain an uncompressed UDIF image without file footer.

#### Encrypted Encoding version 2 encrypted UDIF segment file

An Encrypted Encoding version 2 encryped UDIF segment file consists of:

* [Encrypted Encoding container header](cdsaencr.md#encypted_container_header) at the start of the
  file
* Key protectors
* Unknown (empty values), probably reserved for the key protectors
* Data fork, containing encrypted UDIF data

> Note that the encrypted UDIF data can contain an uncompressed UDIF image without file footer.

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | N/A |
| Character strings | N/A |

The number of bytes per sector is 512.

## File footer {#file_footer}

The file footer (also known as resource file or metadata) (UDIFResourceFile) is 512 bytes in size
and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "koly" | Signature |
| 4 | 4 | 4 | Format version |
| 8 | 4 | 512 | File footer size, in number of bytes |
| 12 | 4 | | [Image flags](#image_flags) |
| 16 | 8 | | Segment logical offset |
| 24 | 8 | | Data fork offset, where the offset is relative from the start of the image file |
| 32 | 8 | | Data fork size, in number of bytes |
| 40 | 8 | | Resource fork offset, where the offset is relative from the start of the image file |
| 48 | 8 | | Resource fork size, in number of bytes |
| 56 | 4 | | Segment number, where 1 represents the first segment and contains 0 if not set |
| 60 | 4 | | Number of segments, which contains 0 if not set |
| 64 | 16 | | Segment set identifier, which contains an UUID |
| 80 | 4 | | Data [checksum type](#checksum_types) |
| 84 | 4 | | Data checksum size, in number of bits |
| 88 | 128 | | Data checksum |
| <td colspan="4">*Introduced in Mac OS 10.2*</td> |
| 216 | 8 | | XML plist offset, where the offset is relative from the start of the image file |
| 224 | 8 | | XML plist size |
| 232 | 120 | | Unknown (Reserved) |
| 352 | 4 | | Master [checksum type](#checksum_types) |
| 356 | 4 | | Master checksum size, in number of bits |
| 360 | 128 | | Master checksum |
| 488 | 4 | | [Image type](#image_types) (or variant) |
| 492 | 8 | | Media size, in number of sectors, which contains the total number of sectors in the (uncompressed) image |
| 500 | 4 | | Unknown (reserved) |
| 504 | 4 | | Unknown (reserved) |
| 508 | 4 | | Unknown (reserved) |

<!-- rumdl-enable MD033 MD056 -->

In an encrypted image file the offsets are relative from the start of the unencrypted image file.

> Note that both the XML plist and resource fork size can be 0, such as in an UDIF stub (UDxx)
> image.

### Image flags {#image_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | kUDIFFlagsFlattened | Flattened image (set by `hdiutil flatten/unflatten`) |
| 0x00000002 | kUDIFFlagsInPlace | |
| 0x00000004 | kUDIFFlagsInternetEnabled | Internet enabled (set by `hdiutil internet-enable`) |
| 0x00000008 | kUDIFFlagsIsEncrypted | |

### Checksum types {#checksum_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 2 | | CRC-32 |
| | | |
| 4 | | MD5 |

### Image types {#image_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | kUDIFDeviceImageType | Device image |
| 2 | kUDIFPartitionImageType | Paritition image |

## Resource fork

In older UDIF images the resource fork contains the image metadata, such as the
[block table](#udif_block_table). The resource fork consists of:

* Resource fork header
* Resource data
* Resource map

### Resource fork header

The resource fork header is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Resource data offset, which contains the byte offset relative to the start of the resource fork |
| 4 | 4 | | Resource map offset, which contains the byte offset relative to the start of the resource fork |
| 8 | 4 | | Resource data size, in number of bytes |
| 12 | 4 | | Resource map size, in number of bytes |

### Resource data {#resource_data}

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Size, in number of bytes |
| 4 | ... | | Data |

### Resource map

The resource map consists of:

* Resource map header
* Entries list
* Names

#### Resource map header

The resource map header is 28 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Unknown (reserved) |
| 16 | 4 | | Unknown (next resource map) |
| 20 | 2 | | Unknown (file reference number) |
| 22 | 2 | | Unknown (resource file attribute flags) |
| 24 | 2 | | Entries list offset, which contains the byte offset relative to the start of the resource map |
| 26 | 2 | | Names list offset, which contains the byte offset relative to the start of the resource map |

#### Resource map entries list

The entries (or type) list is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Number of entries, stored as `value - 1` |
| 2 | ... | | Array of [entries](#resource_map_entry) |

#### Resource map entry {#resource_map_entry}

The resource map entry is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Type indicator (or signature) |
| 4 | 2 | | Number of resource descriptors, stored as `value - 1` |
| 6 | 2 | | Resource descriptors offset, which contains the byte offset relative to the start of the entries list |

A resource map entry is comparable to an item in the
[XML plist resource-fork dictionary](#xml_plist_resource_fork_dictionary) such as the "blkx"
item.

#### Resource descriptor

The resource descriptor (or reference list) is 12 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Resource identifier. Corresponds to the "ID" value in the XML plist. |
| 2 | 2 | | [Resource name](#resource_name) offset, which contains the byte offset relative to the start of the names list where 0xffff indicates the resource has no name. Corresponds to the "Name" value in the XML plist. |
| 4 | 1 | | Resource flags (0x20: Purgeable, 0x40: Protected). Corresponds to the "Attributes" value in the XML plist. |
| 5 | 3 | | [Resource data](#resource_data) offset, which contains the byte offset relative to the start of the resource data |
| 8 | 4 | | Unknown (reserved) |

#### Resource name {#resource_name}

The resource name is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Name size |
| 2 | ... | | Name string, without an end-of-string character |

## XML plist

The XML plist contains image metadata such as the [block table](#udif_block_table).

```xml
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

| Identifier | Description |
| --- | --- |
| resource-fork | dictionary |

### XML plist resource-fork dictionary {#xml_plist_resource_fork_dictionary}

The resource-fork dictionary contains the following key-value pairs:

| Identifier | Description |
| --- | --- |
| blkx | array of dictionaries, which contains [Block table](#udif_block_table) (or block extents) values |
| LPic | optional array of dictionaries, which contains values related to license information |
| plst | array of dictionaries, which contains values related to image properties |
| STR# | optional array of dictionaries, which contains values related to license information |
| TEXT | optional array of dictionaries, which contains values related to license information |

### XML plist array entry

An array entry contains the following key-value pairs:

| Identifier | Description |
| --- | --- |
| Attributes | string that contains a hexadecimal formatted integer value |
| CFName | string |
| Data | string that contains base-64 encoded data |
| ID | string that contains a decimal formatted integer value |
| Name | string |

> Note the the blkx array appears the only one that uses CFName.

## Block table {#udif_block_table}

The block table (BLKXTable) is of variable size and consists of:

* block table header
* block table entries

### The block table header

The block table header is 204 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | "mish" | Signature |
| 4 | 4 | 1 | Format version |
| 8 | 8 | | Start sector, which contains the sector number relative to the start of the media data |
| 16 | 8 | | Number of sectors |
| 24 | 8 | | Base data offset, which contains the byte offset relative to the start of the segment data stream |
| 32 | 4 | | Unknown (BuffersNeeded) |
| 36 | 4 | | Unknown (BlockDescriptors) |
| 40 | 6 x 4 = 24 | 0 | Unknown (reserved) |
| 64 | 4 | | Checksum type |
| 68 | 4 | | Checksum size |
| 72 | 128 | | Checksum |
| 200 | 4 | | Number of entries |

### Block table entry

The block table entry (BLKXChunkEntry) is 40 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Entry type](#udif_block_table_entry_types) |
| 4 | 4 | | Unknown (comment related?) |
| 8 | 8 | | Start sector, which contains the sector number relative to the start of the start sector of the block table |
| 16 | 8 | | Number of sectors |
| 24 | 8 | | Data offset, which contains the byte offset relative to the base data offset in the block table header |
| 32 | 8 | | Data size, which contain the number of bytes of data stored, which is 0 for sparse data |

#### UDIF block table entry types {#udif_block_table_entry_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000000 | | Unknown (sparse) |
| 0x00000001 | | Uncompressed (raw) data |
| 0x00000002 | | Sparse (used for Apple_Free) |
| | | |
| 0x7ffffffe | | Comment |
| | | |
| 0x80000004 | | ADC compressed data |
| 0x80000005 | | zlib compressed data |
| 0x80000006 | | bzip2 compressed data |
| 0x80000007 | | LZFSE compressed data |
| 0x80000008 | | LZMA compressed data |
| | | |
| 0xffffffff | | Block table entries terminator |

## UDIF comment

TODO: complete section

## Notes

Is the maximum compressed chunk size 2048 sectors?

Comment seems to reference compressed data but has no size or number of sectors value.

## Format edge cases and corruption scenarios

### Non-sequential segment files

It is currently unknown if non-sequential segment files are supported.

### XML plist and resource fork both in use

The XML plist and resource fork could be used simultaneously, allowing for a single UDIF to
contain multiple images.

It is currently assumed that the XML plist is leading.

### XML plist and/or resource fork in non-first segment files

The XML plist and/or resource fork could be used in non-first segment files.
