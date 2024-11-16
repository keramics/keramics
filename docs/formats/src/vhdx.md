# Virtual Hard Disk version 2 (VHDX) image format

The Virtual Hard Disk version 2 (VHDX) format is used by Microsoft vitualization
products as one of its image formats. It is both used the store hard disk images
and snapshots.

## Overview

A VHDX image file consist of: 

* file (type) identifier
* 2x image headers
* 2x region tables
* log or metadata journal
* block allocation table (BAT) region
* metadata region
  * metadata table
  * metadata items
* image (content) data 

The elements are stored in 64 KiB (65536 bytes) aligned blocks

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | N/A
| Character strings | UCS-2 little-endian, which allows for unpaired Unicode surrogates, such as "U+d800" and "U+dc00"

The number of bytes per sector is 512 or 4096 depending on the logical sector size.

## File (type) identifier

The file (type) identifier is 64 KiB (65536 bytes) in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "vhdxfile" | Signature
| 8 | 512 | | Creator application and version, with contains an UCS-2 little-endian string with end-of-string character
| 520 | 65016 | | Unknown (reserved)

## Image header

The image header is 4 KiB (4096 bytes) in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "head" | Signature
| 4 | 4 | | Checksum
| 8 | 8 | | Sequence number
| 16 | 16 | | File write identifier, which contains a GUID
| 32 | 16 | | Data write identifier, which contains a GUID
| 48 | 16 | | Log identifier, which contains a GUID
| 64 | 2 | | Log format version
| 66 | 2 | 1 | Format version
| 68 | 4 | | Log size, which according to MS-VHDX this value must be a multitude of 1 MiB
| 72 | 8 | | Log offset, which according to MS-VHDX this value must be a multitude of 1 MiB and greater than or equal to 1 MiB
| 80 | 4016 | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0

The CRC32-C algorithm with the Castagnoli polynomial (0x1edc6f41) and initial
value of 0 is used to calculate the checksum.

The checksum is calculated over the 4 KiB bytes of data of the image header,
where the image header checkum value is considered to be 0 during calculation.

## Region table

The region table is stored in a block of 64 KiB (65536 bytes) and consists of:

* region table header
* 0 or more region table entries
* Unknown (reserved)

TODO: determine if 0 entries is actually supported

### Region table header

The region table header is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "regi" | Signature
| 4 | 4 | | Checksum
| 8 | 4 | | Number of table entries, which according to MS-VHDX this value must be less than or equal to 2047
| 12 | 4 | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0

The CRC32-C algorithm with the Castagnoli polynomial (0x1edc6f41) and initial
value of 0 is used to calculate the checksum.

The checksum is calculated over the 64 KiB bytes of data of the region table
where the image header checkum value is considered to be 0 during calculation.

### Region table entry

The region table entry is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | [Region type identifier](#region_type_identifiers), which contains a GUID
| 16 | 8 | | Region data offset, which contains an offset relative to the start of the file. According to MS-VHDX this value must be a multitude of 1 MiB and greater than or equal to 1 MiB
| 24 | 4 | | Region data size, which according to MS-VHDX this value must be a multitude of 1 MiB
| 28 | 4 | | Is required flag, which contains 1 to indicate the region type needs to be supported.

### <a name="region_type_identifiers"></a>Region type identifiers

| Value | Identifier | Description
| --- | --- | ---
| 2dc27766-f623-4200-9d64-115e9bfd4a08 | | Block allocation table (BAT) region
| 8b7ca206-4790-4b9a-b8fe-575f050f886e | | Metadata region

## Metadata region

The metadata region contains:

* metadata table
* metadata items

### Metadata table

The metadata table is stored in a block of 64 KiB (65536 bytes) and consists of:

* metadata table header
* 0 or more metadata table entries
* Unknown (reserved)

TODO: determine if 0 entries is actually supported

#### Metadata table header

The metadata table header is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "metadata" | Signature
| 8 | 2 | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0
| 10 | 2 | | Number of table entries, which according to MS-VHDX this value must be less than or equal to 2047
| 12 | 20 | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0

#### Metadata table entry

The metdata table entry is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | [Metadata item identifier](#metdata_item_identifiers), which contains a GUID
| 16 | 4 | | Metadata item offset, which contains an offset relative to the start of the metadata region. According to MS-VHDX this value must be greater than 64 KiB
| 20 | 4 | | Metadata item size
| 24 | 8 | | Unknown

TODO: describe last 8 bytes

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | IsUser |
| 0x00000002 | IsVirtualDisk |
| 0x00000004 | IsRequired |

### Metadata items

#### <a name="metdata_item_identifiers"></a>Metadata item identifiers

| Value | Identifier | Description
| --- | --- | ---
| 2fa54224-cd1b-4876-b211-5dbed83bf4b8 | | Virtual disk size
| 8141bf1d-a96f-4709-ba47-f233a8faab5f | | Logical sector size
| a8d35f2d-b30b-454d-abf7-d3d84834ab0c | | Parent locator
| beca12ab-b2e6-4523-93ef-c309e000c746 | | Virtual disk identifier
| caa16737-fa36-4d43-b3b6-33f0aa44e76b | | File parameters
| cda348c7-445d-4471-9cc9-e9885251c556 | | Physical sector size

#### File parameters metadata item

The file parameters metadata item is 8 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Block size, which according to MS-VHDX this value must be a power of 2 and greater than or equal to 1 MiB and not greater than 256 MiB
| 4.0 | 1 bit | | Blocks remain allocated flag, which is used to indicate the file is a fixed-size image
| 4.1 | 1 bit | | Has parent flag, which indicates if the VHDX file contains a differential image that has a parent image
| 4.2 | 30 bits | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0

#### Logical sector size metadata item

The logical sector size metadata item is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Logical sector size, which according to MS-VHDX this value must be either 512 or 4096

#### Parent locator metadata item

The parent locator metadata item is of variable size and consits of:

* parent locator header
* 0 or more parent locator entry
* parent locator key and value data

TODO: determine if 0 entries is actually supported

##### Parent locator header

The parent locator header is 20 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Parent locator type indicator, which contains the GUID: b04aefb7-d19e-4a81-b789-25b8e9445913
| 16 | 2 | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0
| 18 | 2 | | Number of entries (or key-value pairs)

##### Parent locator entry

The parent locator entry is 12 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Key data offset, which contains the offset relative from the start of the parent locator header
| 4 | 4 | | Value data offset, which contains the offset relative from the start of the parent locator header
| 8 | 2 | | Key data size
| 10 | 2 | | Value data size

##### Parent locator key and value data

A parent locator key or value is stored as UCS-2 little-endian string without
end-of-string character.

Known keys are:

| Value | Description
| --- | ---
| absolute_win32_path | The value contains an absolute drive Windows path "\\?\c:\file.vhdx"
| parent_linkage | The value contains a string of a GUID. This GUID should correspond to the data write identifier of the parent image
| parent_linkage2 | The value contains a string of a GUID
| relative_path | The value contains a relative Windows path "..\file.vhdx"
| volume_path | The value contains an absolute volume Windows path with "\\?\Volume{%GUID%}\file.vhdx"

#### Physical sector size metadata item

The physical sector size metadata item is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Physical sector size, which according to MS-VHDX this value must be either 512 or 4096

#### Virtual disk identifier metadata item

The virtual disk identifier metadata item is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Virtual disk identifier, which contains a GUID

> Note that in contrast to VHD (version 1) the virtual disk identifier does
> not change between a differential image and its parent. The data write
> identifier seems to be used instead.

#### Virtual disk size metadata item

The virtual disk size metadata item is 8 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Virtual disk size

## Block allocation table (BAT) region

The block allocation table (BAT) region contains the block allocation table.
The entries of this table describe the location of either blocks containing
image content data (or payload blocks) or blocks containing a sector bitmap.

The size of an individual sector bitmap block is 1 MiB which allows for `2^23`
sectors to be represented by the bitmap.

Block allocation table (BAT) entries are grouped in chunks. The size of a chunk
can be calculated as following:

```
number of entries per chunk = ( 2^23 * logical sector size ) / block size
```

The block allocation table (BAT) consists of:

* one or more chunks containing:
  * number of entries per chunk x BAT entry describing image content data
  * 1 x BAT entry describing the a sector bitmap

Unused BAT entries are filled with 0-byte values.

The block allocation table (BAT) of:

* a fixed-size image does not contain sector bitmap entries;
* a dynamic-size image does contain sector bitmap entries, although according to MS-VHDX are not used;
* a differential image does contain sector bitmap entries.

### Block allocation table (BAT) entry

The block allocation table (BAT) entry is 64 bits in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 3 bits | | Block state
| 0.3 | 17 bits | 0 | Unknown (reserved), which according to MS-VHDX this value must be set to 0
| 2.4 | 44 bits | | Block offset, which contains the offset relative from the start of the file as a multitude of 1 MiB

### Block states

#### Payload block states

| Value | Identifier | Description
| --- | --- | ---
| 0 | PAYLOAD_BLOCK_NOT_PRESENT | Block is new and therefore not (yet) stored in the file
| 1 | PAYLOAD_BLOCK_UNDEFINED | Block is not stored in the file
| 2 | PAYLOAD_BLOCK_ZERO | Block is sparse and therefore filled with 0-byte values
| 3 | PAYLOAD_BLOCK_UNMAPPED | Block has been unmapped
| | |
| 6 | PAYLOAD_BLOCK_FULLY_PRESENT | Block is stored in the file
| 7 | PAYLOAD_BLOCK_PARTIALLY_PRESENT | Block is stored in the parent

#### Sector bitmap block states

| Value | Identifier | Description
| --- | --- | ---
| 0 | SB_BLOCK_NOT_PRESENT | Block is new and therefore not (yet) stored in the file
| | |
| 6 | SB_BLOCK_PRESENT | Block is stored in the file

### Sector bitmap

In differential disk images the sector bitmap indicates which sectors are stored
within the image (bit set to 1) or in the parent (bit set to 0).

The bitmap is stored in a 1 MiB block.

The bitmap is stored on a per-byte basis with the LSB represents the first bit
in the bitmap.

## Log (metadata journal)

TODO: complete section

The log serves as metadata journal is of variable size and consist of contiguous
circular (ring) buffer that contains log entries.

### Log entry

TODO: complete section

4 KiB (4096 bytes) in size

#### Log entry header

TODO: complete section

#### Zero descriptor

TODO: complete section

#### Data descriptor

TODO: complete section

#### Data sector

TODO: complete section

## References

* [MS-VHDX: Virtual Hard Disk v2 (VHDX) File Format](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-vhdx/83e061f8-f6e2-4de1-91bd-5d518a43d477), by Microsoft
