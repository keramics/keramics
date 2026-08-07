# Universal Disk Image Format (UDIF)

The Universal Disk Image Format (UDIF) (.dmg) is one of the disk image formats supported natively
by Mac OS. UDIF supersedes the New Disk Image Format (NDIF) format and was introduced in Max OS
X 10.0 (Cheetah).

## Overview

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

An UDIF image can consists of one or more segment files, where:

* the first segment file is named: "image.dmg"
* successive segment files are named: "image.###.dmgpart", where "###" represents a numeric value
  starting with 2 with 0 padding, e.g. "image.002.dmgpart". Segment files after 999 are assumed to
  be named without the 0 padding, e.g. "image.1234.dmgpart".

The data forks of the segment files are used as a contiguous data stream. A compressed block can
be stored across multiple segment files.

Only the first segment file contains a resource fork or XML plist.

### Terminology

| Term | Description |
| --- | --- |
| Flattened image | The disk image is a self-contained, a resource fork is stored within the image |
| Unflattened image | The disk image uses the file system to store a resource fork |

### Uncompressed image format

An uncompressed UDIF image consist of:

* data
* optional file footer

> Note that an uncompressed UDIF image without file footer is equivalent to a RAW storage media
> image (CRawDiskImage).

### Compressed image format

A compressed UDIF image consist of:

* Data fork
* Optional resource fork or XML plist
* [File footer](#file_footer) at the end of the image file

### Encrypted image format

#### Encrypted image format version 1

An encrypted UDIF image (version 1) consist of:

* Encyrypted uncompressed or compressed UDIF image data
* [Encrypted file footer](#encypted_file_footer) at the end of the image file

#### Encrypted image format version 2

An encrypted UDIF image (version 2) consist of:

* [Encrypted file header](#encypted_file_header) at the start of the image file
* Encyrypted uncompressed or compressed UDIF image data

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
| 492 | 8 | | Number of sectors |
| 500 | 4 | | Unknown (reserved) |
| 504 | 4 | | Unknown (reserved) |
| 508 | 4 | | Unknown (reserved) |

<!-- rumdl-enable MD033 MD056 -->

> Note that the XML plist size can be 0, such as in an UDIF stub (UDxx) image.

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

In older UDIF images the resource fork contains the [block table](#udif_block_table). The
resource fork consists of:

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

TODO: complete section

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
| blkx | array of dictionaries | [Block table](#udif_block_table) (or block extents) values
| LPic | array of dictionaries | Optional values related to license information
| plst | array of dictionaries | Values related to image properties
| STR# | array of dictionaries | Optional values related to license information
| TEXT | array of dictionaries | Optional values related to license information

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
| 24 | 8 | | Unknown (DataOffset), which seems to be always 0 |
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
| 4 | 4 | | Unknown (comment?) |
| 8 | 8 | | Start sector, which contains the sector number relative to the start of the start sector of the block table |
| 16 | 8 | | Number of sectors |
| 24 | 8 | | Data offset, which contains the byte offset relative to the start of the segment data stream |
| 32 | 8 | | Data size, which contain the number of bytes of data stored, which is 0 for sparse data |

#### UDIF block table entry types {#udif_block_table_entry_types}

| Value | Identifier | Description  |
| --- | --- | --- |
| 0x00000000 | | Unknown (sparse) |
| 0x00000001 | | Uncompressed (raw) data |
| 0x00000002 | | Sparse (used for Apple_Free) |
| | | |
| 0x7ffffffe | | Comment |
| | | |
| 0x80000004 | | ADC compressed data |
| 0x80000005 | | zlib compressed data |
| 0x80000006 | | bzip2 compressed data  |
| 0x80000007 | | LZFSE compressed data  |
| 0x80000008 | | LZMA compressed data |
| | | |
| 0xffffffff | | Block table entries terminator |

## UDIF comment

TODO: complete section

## Notes

Is the maximum compressed chunk size 2048 sectors?

Comment seems to reference compressed data but has no size or number of sectors value.

## Encrypted file

### Encrypted file footer {#encypted_file_footer}

The encrypted file footer is 1276 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Unknown (UUID) |
| 16 | 4 | | Block size, in number of bytes |
| 20 | 4 | | Blob [encryption method](#encryption_methods) (or algorithm) |
| 24 | 4 | | Blob [encryption padding type](#encryption_padding_types) |
| 28 | 4 | | Blob [encryption mode](#encryption_modes) |
| 32 | 4 | | Blob key size, in number of bits |
| 36 | 4 | | Blob initialization vector size |
| 40 | 4 | | Key derivation [encryption method](#encryption_methods) (or algorithm) |
| 44 | 4 | | Key derivation iteration count |
| 48 | 4 | | Unknown |
| 52 | 4 | | Key derivation salt size |
| 56 | 32 | | Key derivation salt |
| 88 | 4 | | Block initialization vector size |
| 92 | 4 | | Block [encryption mode](#encryption_modes) |
| 96 | 4 | | Data (or block) [encryption method](#encryption_methods) (or algorithm) |
| 100 | 4 | | Block key size, in number of bits |
| 104 | 32 | | Block initialization vector |
| 136 | 4 | | Wrapped AES key size |
| 140 | 256 | | Wrapped AES key |
| 396 | 4 | | HMAC [encryption method](#encryption_methods) (or algorithm) |
| 400 | 4 | | Unknown (HMAC number of bits?) |
| 404 | 32 | | HMAC initialization vector |
| 436 | 4 | | Wrapped HMAC key size |
| 440 | 256 | | Wrapped HMAC key |
| 696 | 4 | | Integrity [encryption method](#encryption_methods) (or algorithm) |
| 700 | 4 | | Unknown (Integrity number of bits?) |
| 704 | 32 | | Integrity initialization vector |
| 736 | 4 | | Wrapped integrity key size |
| 740 | 256 | | Wrapped integrity key |
| 996 | 4 | | Unknown (data size) |
| 1000 | 256 | | Unknown (data) |
| 1256 | 4 | | Data area offset, where the offset is relative from the start of the image file |
| 1260 | 4 | | Data area size, in number of bytes |
| 1264 | 4 | 1 | Encrypted file format version |
| 1268 | 8 | "cdsaencr" | Signature |

### Encrypted file header {#encypted_file_header}

The encrypted file header is 512 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "encrcdsa" | Signature |
| 8 | 4 | 2 | Encrypted file format version |
| 12 | 4 | | Block initialization vector size |
| 16 | 4 | | Block [encryption mode](#encryption_modes) |
| 20 | 4 | | Data (or block) [encryption method](#encryption_methods) (or algorithm) |
| 24 | 4 | | Block key size, in number of bits |
| 28 | 4 | | initialization vector [encryption method](#encryption_methods) (or algorithm) |
| 32 | 4 | | initialization vector size |
| 36 | 16 | | Unknown (UUID) |
| 52 | 4 | | Block size, in number of bytes |
| 56 | 8 | | Data area offset, where the offset is relative from the start of the image file |
| 64 | 8 | | Data area size, in number of bytes |
| 72 | 4 | | Number of item descriptors |
| 76 | ... | | Array of item descriptors |
| ... | 436 | | Unknown |

#### Item descriptor

The item descriptor is 20 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Type |
| 4 | 8 | | Offset, where the offset is relative from the start of the image file |
| 12 | 8 | | Size |

#### Item types

| Value | Identifier | Description  |
| --- | --- | --- |
| 1 | CSSM_APPLE_UNLOCK_TYPE_KEY_DIRECT | Master secret key stored directly |
| 1 | CSSM_APPLE_UNLOCK_TYPE_WRAPPED_PRIVATE | Master key wrapped by public key |
| 3 | CSSM_APPLE_UNLOCK_TYPE_KEYBAG | Master key wrapped by keybag |

Defined in cssmapple.h

### Encryption methods {#encryption_methods}

| Value | Identifier | Description  |
| --- | --- | --- |
| 0 | CSSM_ALGID_NONE | None |
| 1 | CSSM_ALGID_CUSTOM | |
| 2 | CSSM_ALGID_DH | |
| 3 | CSSM_ALGID_PH | |
| 4 | CSSM_ALGID_KEA | |
| 5 | CSSM_ALGID_MD2 | |
| 6 | CSSM_ALGID_MD4 | |
| 7 | CSSM_ALGID_MD5 | |
| 8 | CSSM_ALGID_SHA1 | |
| 9 | CSSM_ALGID_NHASH | |
| 10 | CSSM_ALGID_HAVAL: | |
| 11 | CSSM_ALGID_RIPEMD | |
| 12 | CSSM_ALGID_IBCHASH | |
| 13 | CSSM_ALGID_RIPEMAC | |
| 14 | CSSM_ALGID_DES | |
| 15 | CSSM_ALGID_DESX | |
| 16 | CSSM_ALGID_RDES | |
| 17 | CSSM_ALGID_3DES_3KEY_EDE | |
| 18 | CSSM_ALGID_3DES_2KEY_EDE | |
| 19 | CSSM_ALGID_3DES_1KEY_EEE | |
| 20 | CSSM_ALGID_3DES_3KEY_EEE | |
| 21 | CSSM_ALGID_3DES_2KEY_EEE | |
| 22 | CSSM_ALGID_IDEA | |
| 23 | CSSM_ALGID_RC2 | |
| 24 | CSSM_ALGID_RC5 | |
| 25 | CSSM_ALGID_RC4 | |
| 26 | CSSM_ALGID_SEAL | |
| 27 | CSSM_ALGID_CAST | |
| 28 | CSSM_ALGID_BLOWFISH | |
| 29 | CSSM_ALGID_SKIPJACK | |
| 30 | CSSM_ALGID_LUCIFER | |
| 31 | CSSM_ALGID_MADRYGA | |
| 32 | CSSM_ALGID_FEAL | |
| 33 | CSSM_ALGID_REDOC | |
| 34 | CSSM_ALGID_REDOC3 | |
| 35 | CSSM_ALGID_LOKI | |
| 36 | CSSM_ALGID_KHUFU | |
| 37 | CSSM_ALGID_KHAFRE | |
| 38 | CSSM_ALGID_MMB | |
| 39 | CSSM_ALGID_GOST | |
| 40 | CSSM_ALGID_SAFER | |
| 41 | CSSM_ALGID_CRAB | |
| 42 | CSSM_ALGID_RSA | |
| 43 | CSSM_ALGID_DSA | |
| 44 | CSSM_ALGID_MD5WithRSA | |
| 45 | CSSM_ALGID_MD2WithRSA | |
| 46 | CSSM_ALGID_ElGamal | |
| 47 | CSSM_ALGID_MD2Random | |
| 48 | CSSM_ALGID_MD5Random | |
| 49 | CSSM_ALGID_SHARandom | |
| 50 | CSSM_ALGID_DESRandom | |
| 51 | CSSM_ALGID_SHA1WithRSA | |
| 52 | CSSM_ALGID_CDMF | |
| 53 | CSSM_ALGID_CAST3 | |
| 54 | CSSM_ALGID_CAST5 | |
| 55 | CSSM_ALGID_GenericSecret | |
| 56 | CSSM_ALGID_ConcatBaseAndKey | |
| 57 | CSSM_ALGID_ConcatKeyAndBase | |
| 58 | CSSM_ALGID_ConcatBaseAndData | |
| 59 | CSSM_ALGID_ConcatDataAndBase | |
| 60 | CSSM_ALGID_XORBaseAndData | |
| 61 | CSSM_ALGID_ExtractFromKey | |
| 62 | CSSM_ALGID_SSL3PreMasterGen | |
| 63 | CSSM_ALGID_SSL3MasterDerive | |
| 64 | CSSM_ALGID_SSL3KeyAndMacDerive | |
| 65 | CSSM_ALGID_SSL3MD5_MAC | |
| 66 | CSSM_ALGID_SSL3SHA1_MAC | |
| 67 | CSSM_ALGID_PKCS5_PBKDF1_MD5 | |
| 68 | CSSM_ALGID_PKCS5_PBKDF1_MD2 | |
| 69 | CSSM_ALGID_PKCS5_PBKDF1_SHA1 | |
| 70 | CSSM_ALGID_WrapLynks | |
| 71 | CSSM_ALGID_WrapSET_OAEP | |
| 72 | CSSM_ALGID_BATON | |
| 73 | CSSM_ALGID_ECDSA | |
| 74 | CSSM_ALGID_MAYFLY | |
| 75 | CSSM_ALGID_JUNIPER | |
| 76 | CSSM_ALGID_FASTHASH | |
| 77 | CSSM_ALGID_3DES | |
| 78 | CSSM_ALGID_SSL3MD5 | |
| 79 | CSSM_ALGID_SSL3SHA1 | |
| 80 | CSSM_ALGID_FortezzaTimestamp | |
| 81 | CSSM_ALGID_SHA1WithDSA | |
| 82 | CSSM_ALGID_SHA1WithECDSA | |
| 83 | CSSM_ALGID_DSA_BSAFE | |
| 84 | CSSM_ALGID_ECDH | |
| 85 | CSSM_ALGID_ECMQV | |
| 86 | CSSM_ALGID_PKCS12_SHA1_PBE | |
| 87 | CSSM_ALGID_ECNRA | |
| 88 | CSSM_ALGID_SHA1WithECNRA | |
| 89 | CSSM_ALGID_ECES | |
| 90 | CSSM_ALGID_ECAES | |
| 91 | CSSM_ALGID_SHA1HMAC | |
| 92 | CSSM_ALGID_FIPS186Random | |
| 93 | CSSM_ALGID_ECC | |
| 94 | CSSM_ALGID_MQV | |
| 95 | CSSM_ALGID_NRA | |
| 96 | CSSM_ALGID_IntelPlatformRandom | |
| 97 | CSSM_ALGID_UTC | |
| 98 | CSSM_ALGID_HAVAL3 | |
| 99 | CSSM_ALGID_HAVAL4 | |
| 100 | CSSM_ALGID_HAVAL5 | |
| 101 | CSSM_ALGID_TIGER | |
| 102 | CSSM_ALGID_MD5HMAC | |
| 103 | CSSM_ALGID_PKCS5_PBKDF2 | |
| 104 | CSSM_ALGID_RUNNING_COUNTER | |
| | | |
| 0x80000000 | CSSM_ALGID_VENDOR_DEFINED | |
| 0x80000001 | CSSM_ALGID_AES | |

### Encryption padding types {#encryption_padding_types}

| Value | Identifier | Description  |
| --- | --- | --- |
| 0 | CSSM_PADDING_NONE | |
| 1 | CSSM_PADDING_CUSTOM | |
| 2 | CSSM_PADDING_ZERO | |
| 3 | CSSM_PADDING_ONE | |
| 4 | CSSM_PADDING_ALTERNATE | |
| 5 | CSSM_PADDING_FF | |
| 6 | CSSM_PADDING_PKCS5 | |
| 7 | CSSM_PADDING_PKCS7 | |
| 8 | CSSM_PADDING_CIPHERSTEALING | |
| 9 | CSSM_PADDING_RANDOM | |
| 10 | CSSM_PADDING_PKCS1 | |

### Encryption modes {#encryption_modes}

| Value | Identifier | Description  |
| --- | --- | --- |
| 0 | CSSM_ALGMODE_NONE | |
| 1 | CSSM_ALGMODE_CUSTOM | |
| 2 | CSSM_ALGMODE_ECB | |
| 3 | CSSM_ALGMODE_ECBPad | |
| 4 | CSSM_ALGMODE_CBC | |
| 5 | CSSM_ALGMODE_CBC_IV8 | |
| 6 | CSSM_ALGMODE_CBCPadIV8 | |
| 7 | CSSM_ALGMODE_CFB | |
| 8 | CSSM_ALGMODE_CFB_IV8 | |
| 9 | CSSM_ALGMODE_CFBPadIV8 | |
| 10 | CSSM_ALGMODE_OFB | |
| 11 | CSSM_ALGMODE_OFB_IV8 | |
| 12 | CSSM_ALGMODE_OFBPadIV8 | |
| 13 | CSSM_ALGMODE_COUNTER | |
| 14 | CSSM_ALGMODE_BC | |
| 15 | CSSM_ALGMODE_PCBC | |
| 16 | CSSM_ALGMODE_CBCC | |
| 17 | CSSM_ALGMODE_OFBNLF | |
| 18 | CSSM_ALGMODE_PBC | |
| 19 | CSSM_ALGMODE_PFB | |
| 20 | CSSM_ALGMODE_CBCPD | |
| 21 | CSSM_ALGMODE_PUBLIC_KEY | |
| 22 | CSSM_ALGMODE_PRIVATE_KEY | |
| 23 | CSSM_ALGMODE_SHUFFLE | |
| 24 | CSSM_ALGMODE_ECB64 | |
| 25 | CSSM_ALGMODE_CBC64 | |
| 26 | CSSM_ALGMODE_OFB64 | |
| 28 | CSSM_ALGMODE_CFB32 | |
| 29 | CSSM_ALGMODE_CFB16 | |
| 30 | CSSM_ALGMODE_CFB8 | |
| 31 | CSSM_ALGMODE_WRAP | |
| 32 | CSSM_ALGMODE_PRIVATE_WRAP | |
| 33 | CSSM_ALGMODE_RELAYX | |
| 34 | CSSM_ALGMODE_ECB128 | |
| 35 | CSSM_ALGMODE_ECB96 | |
| 36 | CSSM_ALGMODE_CBC128 | |
| 37 | CSSM_ALGMODE_OAEP_HASH | |
| 38 | CSSM_ALGMODE_PKCS1_EME_V15 | |
| 39 | CSSM_ALGMODE_PKCS1_EME_OAEP | |
| 40 | CSSM_ALGMODE_PKCS1_EMSA_V15 | |
| 41 | CSSM_ALGMODE_ISO_9796 | |
| 42 | CSSM_ALGMODE_X9_31 | |

## Format edge cases and corruption scenarios

### Non-sequential segment files

It is currently unknown if non-sequential segment files are supported.

### XML plist and resource fork both in use

The XML plist and resource fork could be used simultaneously, allowing for a single UDIF to
contain multiple images.

### XML plist and/or resource fork in non-first segment files

The XML plist and/or resource fork could be used in non-first segment files.
