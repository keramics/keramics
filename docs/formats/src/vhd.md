# Virtual Hard Disk (VHD) image format

The Virtual Hard Disk (VHD) format is used by Microsoft vitualization products
as one of its image format. It is both used the store hard disk images and
snapshots.

## Overview

There are multiple types of VHD images, namely:

* Fixed-size VHD image
* Dynamic-size (or sparse) VHD image
* Differential (or differencing) VHD image

### Fixed-size hard disk image

A fixed-size VHD image consists of:

* data
* file footer

> Note that a fixed-size VHD image is equivalent to a raw storage media image
> with an additional footer. 

### Dynamic-size (or sparse) hard disk image

A dynamic-size (or sparse) VHD image consists of:

* copy of file footer
* dynamic disk header
* block allocation table
* data in blocks
* file footer

### Differential hard disk image

A differential (or differencing) VHD image consists of:

* copy of file footer
* dynamic disk header
* block allocation table
* data in blocks
* file footer

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | big-endian
| Date and time values | Number of seconds since January 1, 2000 00:00:00 UTC
| Character strings | UCS-2 big-endian, which allows for unpaired Unicode surrogates, such as "U+d800" and "U+dc00"

The number of bytes per sector is 512.

### Undo disk image

Virtual PC has a feature to create "Undo Disks". This undo disk feature stores
a differential hard disk image in files named something similar like:

```
VirtualPCUndo_<name>_0_0_hhmmssMMDDYYYY.vud
```

Where the date and time seems to be stored in UTC and &lt;name&gt; represents the
name of the parent image.

## File footer

The file footer is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "conectix" | Signature (also referred to as cookie)
| 8 | 4 | | Features
| 12 | 4 | 0x00010000 | Format version, where the upper 16-bit are the major version and the lower 16-bit the minor version
| 16 | 8 | | Next offset, which contains the offset to the next (metadata) structure. The offset is relative from the start of the file. It should only be set in dynamic and differential disk images. In fixed disk images it should be set to 0xffffffffffffffff (-1).
| 24 | 4 | | Modification time, which contains the number of seconds since January 1, 2000 00:00:00 UTC
| 28 | 4 | | Creator application
| 32 | 4 | | Creator version, where the upper 16-bit are the major version and the lower 16-bit the minor version
| 36 | 4 | | Creator (host) operating system
| 40 | 8 | | Disk size, which contains the size of the disk in bytes
| 48 | 8 | | Data size, which contains the size of the data in bytes
| 56 | 4 | | Disk geometry
| 60 | 4 | | Disk type
| 64 | 4 | | Checksum
| 68 | 16 | | Identifier, which contains a big-endian UUID
| 84 | 1 | | Saved state, which contains a flag to indicate the image is in saved state.
| 85 | 427 | 0 | Unknown (Reserved should contain 0-byte values)

> Note that the checksum is a one's complement of the sum of all the bytes in
> the file footer without the checksum field.

### Features

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 1 bit | | Is temporary disk, which indicates that this disk is a candidate for deletion on shutdown
| 0.1 | 1 bit | | Unknown (Reserved, must be set to 1)
| 0.2 | 30 bits | | Unknown (Reserved, must be set to 0)

A value of 0 represents no features are enabled.

### Creator application

| Value | Identifier | Description
| --- | --- | ---
| "d2v\x00" | | Disk2vhd
| "qemu" | | Qemu
| "vpc\x20" | | Virtual PC
| "vs\x20\x20" | | Virtual Server
| "win\x20" | | Windows (Disk Management)

### Creator host operating system

| Value | Identifier | Description
| --- | --- | ---
| "Mac\x20" | | Macintosh
| "Wi2k" | | Windows

### Disk geometry

The disk geometry is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Number of cylinders
| 2 | 1 | | Number of heads
| 3 | 1 | | Number of sectors per track (cylinder)

### Disk type

| Value | Identifier | Description
| --- | --- | ---
| 0 | | None
| 1 | | Unknown (Deprecated)
| 2 | | Fixed hard disk
| 3 | | Dynamic hard disk
| 4 | | Differential hard disk
| 5 | | Unknown (Deprecated)
| 6 | | Unknown (Deprecated)

## Dynamic disk header

The dynamic disk header is 1024 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "cxsparse" | Signature (Cookie)
| 8 | 8 | | Next offset, which contains the offset to the next (metadata) structure. The offset is relative from the start of the file. Currently this is unused and should be set to 0xffffffffffffffff (-1).
| 16 | 8 | | Block allocation table offset, whic contains the offset to the block allocation table structure. The offset is relative from the start of the file.
| 24 | 4 | 0x00010000 | Format version, where the upper 16-bit are the major version and the lower 16-bit the minor version
| 28 | 4 | | Number of blocks, which contains the maximum number of block allocation table entries
| 32 | 4 | | Block size. The block size must be a power-of-two multitude of the sector size and does not include the size of the sector bitmap. The default block size is 4096 x 512-byte sectors (2 MB).
| 36 | 4 | | Checksum
| 40 | 16 | | Parent identifier, which contains a big-endian UUID that identifies the parent image. Only used by differential hard disk images.
| 56 | 4 | | Parent last modification time, which contains the number of seconds since January 1, 2000 00:00:00 UTC. Only used by differential hard disk images.
| 60 | 4 | 0 | Unknown (Reserved should contain 0-byte values)
| 64 | 512 | | Parent name, which contains an UCS-2 big-endian string. Only used by differential hard disk images.
| 576 | 8 x 24 = 192 | | Array of parent locator entries. Only used by differential hard disk images.
| 768 | 256 | 0 | Unknown (Reserved should contain 0-byte values)

The maximum number of block allocation table entries should match the maximum
possible number of blocks in the disk.

> Note that the parent name can also contain a full path, e.g. in .avhd files.
> The part segments are separated by the \ character.

> Note that the checksum is a one's complement of the sum of all the bytes in
> the file dynamic disk header without the checksum field.

### Parent locator entry

The parent locator entry is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Locator platform code
| 4 | 4 | | Platform data space, which contains the number of 512-byte sectors needed to store the parent hard disk locator
| 8 | 4 | | Locator data size
| 12 | 4 | 0 | Unknown (Reserved should contain 0-byte values)
| 16 | 8 | | Locator data offset, which contains the offset to the locator data. The offset is relative from the start of the file.

#### Locator platform code

| Value | Identifier | Description
| --- | --- | ---
| 0 | | None
| | |
| "Mac\x20" | | Mac OS alias stored as a blob
| "MacX" | | File URL with UTF-8 encoding conforming to RFC 2396
| | |
| "W2ku" | | Absolute Windows path, which contains an UCS-2 big-endian string
| "W2ru" | | Windows path relative to the differential image, which contains an UCS-2 big-endian string
| "Wi2k" | | Unknown (Deprecated)
| "Wi2r" | | Unknown (Deprecated)

## Block allocation table

The block allocation table is only used in dynamic and differential disk images.

The block allocation table consists of 32-bit entries. The entries represent
the sector number where the data block starts or unused when set to
0xffffffff (-1).

```
if block allocation table entry == 0xffffffff (-1):
        block is sparse or stored in parent
else:
        file offset = ( block allocation table entry * 512 ) + sector bitmap size
```

Unused block in a dynamic disk are sparse and should be filled with zero byte
values. In a differential disk the block is stored in the parent disk image.

## Data blocks

Data blocks are only used in dynamic and differential disk images.

A data block consists of:

* sector bitmap
* sector data

```
size of bitmap (in bytes) = block size / ( 512 * 8 )
```

The size of the bitmap is rounded up to the next multitude of the sector size.

### Sector bitmap

In dynamic disk images the sector bitmap indicates which sectors contain data
(bit set to 1) or are sparse (bit set to 0).

In differential disk images the sector bitmap indicates which sectors are stored
within the image (bit set to 1) or in the parent (bit set to 0).

The bitmap is padded to a 512-byte sector boundary.

The bitmap is stored on a per-byte basis with the MSB represents the first bit
in the bitmap.

## References

* [VHD Specifications](https://www.microsoft.com/en-us/download/details.aspx?id=23850), by Microsoft
