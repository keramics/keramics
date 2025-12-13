# VMware Virtual Disk (VMDK) format

The VMware Virtual Disk (VMDK) format is used by VMware virtualization products
as one of its image format.

## Overview

A VMDK disk image can consist of multiple files, such as:

* descriptor file
* extent data files
* raw extent data file
* VMDK sparse extent data file
* COWD sparse extent data file

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values |
| Character strings | narrow character (Single Byte Character (SBC) or Multi Byte Character (MBC)) stored using a codepage defined in the descriptor file.

The number of bytes per sector is 512.

### Disk types

There are multiple types of VMKD images, namely:

The 2GbMaxExtentFlat (or twoGbMaxExtentFlat) disk image, which consists of:

* a descriptor file (&lt;name&gt;.vmdk)
* raw data extent files (&lt;name&gt;-f###.vmdk), where ### is contains a decimal value starting with 1.

The 2GbMaxExtentSparse (or twoGbMaxExtentSparse) disk image, which consists of:

* a descriptor file (&lt;name&gt;.vmdk)
* VMDK sparse data extent files (&lt;name&gt;-s###.vmdk), where ### is contains a decimal value starting with 1.

The monolithicFlat disk image, which consists of:

* a descriptor file (&lt;name&gt;.vmdk)
* raw data extent file (&lt;name&gt;-f001.vmdk)

The monolithicSparse disk image, which consists of:

* VMDK sparse data extent file (&lt;name&gt;.vmdk) also contains the descriptor file data.

The vmfs disk image, which consists of:

* a descriptor file (&lt;name&gt;.vmdk)
* raw data extent file (&lt;name&gt;-flat.vmdk)

The vmfsSparse differential disk image, which consists of:

* a descriptor file (&lt;name&gt;.vmdk)
* COWD sparse data extent files (&lt;name&gt;-delta.vmdk)

TODO: describe more disk types

### Delta links

A delta link is similar to a differential image where the image contains the
changes (or delta) in comparison of a parent image. According to the Virtual
Disk Format 5.0 specification one delta image can chain to another delta image.

TODO: Name &lt;name&gt;-delta.vmdk

## Descriptor file {#descriptor_file}

The descriptor file is a case-insensitive text based file that contains the
following information:

* optional comment and empty lines
* header
* extent descriptions
* optional change tracking file
* disk data base (DDB)

> Note that the descriptor file can contains leading and trailing whitespace.
> Lines are separated by a line feed character (0x0a). And leading comment
> (starting with #) and empty lines.

### Header

The header of a descriptor file looks similar to the data below.

```
# Disk DescriptorFile
version=1
CID=12345678
parentCID=ffffffff
createType="twoGbMaxExtentSparse"
```

The header consists of the following values:

| Value | Description
| --- | ---
| "# Disk DescriptorFile" | Section header (or file signature)
| version | [Format version](#descriptor_file_format_versions)
| encoding | [Encoding](#descriptor_file_encodings)
| CID | Content identifier, which contains a random 32-bit value updated the first time the content of the virtual disk is modified after the virtual disk is opened.
| parentCID | The content identifier of the parent, which contains a 32-bit value identifying the parent content, where a value of 'ffffffff' (-1) represents no parent content.
| isNativeSnapshot | TODO: add description. A value of "no" has been observed in a VMWare Player 9 descriptor file.
| createType | [Disk type](#descriptor_file_disk_types)
| parentFileNameHint | Contains the path to the parent image, which is only present if the image is a differential image (delta link).

TODO: confirm if a content identifier of 'fffffffe' (-2) represents that the long content identifier should be used

#### Format versions {#descriptor_file_format_versions}

| Value | Description
| --- | ---
| 1 | TODO: add description
| 2 | TODO: add description
| 3 | TODO: add description

#### Encodings {#descriptor_file_encodings}

> Note that it is currently unknown which encodings are supported, currently it
> is assumed that at least the Windows codepages are supported and that the
> default is UTF-8.

| Value | Description
| --- | ---
| Big5 | Big5 assumed to be equivalent to Windows codepage 950
| GBK | GBK assumed to be equivalent to Windows codepage 936, which was observed in VMWare Workstation for Windows, Chinese edition
| Shift_JIS | Shift_JIS assumed to be equivalent to Windows codepage 932, which was observed in VMWare Workstation for Windows, Japanese edition
| UTF-8 | UTF-8
| |
| windows-949-2000 | Windows codepage 949, 2000 version
| windows-1252 | Windows codepage 1252, which was observed in VMWare Player 9 descriptor file

#### Disk types {#descriptor_file_disk_types}

| Value | Description
| --- | ---
| 2GbMaxExtentFlat, twoGbMaxExtentFlat | The disk is split into fixed-size extents of maximum 2 GB, which consists of raw extent data files.
| 2GbMaxExtentSparse, twoGbMaxExtentSparse | The disk is split into sparse (dynamic-size) extents of maximum 2 GB, which consists of VMDK sparse extent data files.
| custom | TODO: add description. Descriptor file with arbitrary extents, used to mount v2i-format.
| fullDevice | The disk uses a full physical disk device.
| monolithicFlat | The disk is a single raw extent data file.
| monolithicSparse | The disk is a single VMDK sparse extent data file.
| partitionedDevice | The disk uses a full physical disk device, using access per partition.
| streamOptimized | The disk is a single compressed VMDK sparse extent data file.
| vmfs | The disk is a single raw extent data file, which is similar to the "monolithicFlat".
| vmfsEagerZeroedThick | The disk is a single raw extent data file.
| vmfsPreallocated | The disk is a single raw extent data file.
| vmfsRaw | The disk uses a full physical disk device.
| vmfsRDM, vmfsRawDeviceMap | The disk uses a full physical disk device, which is also referred to as Raw Device Map (RDM)
| vmfsRDMP, vmfsPassthroughRawDeviceMap | The disk uses a full physical disk device, which is similar to the Raw Device Map (RDM), but sends SCSI commands to underlying hardware.
| vmfsSparse | The disk is split into COWD sparse (dynamic-size) extents.
| vmfsThin | The disk is split into COWD sparse (dynamic-size) extents.

### Extent descriptions

The extent descriptions of a descriptor file looks similar to the data below.

```
# Extent description
RW 4192256 SPARSE "test-s001.vmdk"
```

```
# Extent description
RW 1048576 FLAT "test-f001.vmdk" 0
```

The extent descriptions consists of the following values:

| Value | Description
| --- | ---
| "# Extent description" | Section header
| | Extent descriptors

#### Extent descriptor

The extent descriptor consists of the following values:

| Value | Description
| --- | ---
| 1st | [Access mode](#extent_access_mode)
| 2nd | The number of sectors
| 3rd | [Extent type](#extent_types)
| <td colspan="2">*If extent type is not ZERO*</td>
| 4th | Path of the VMDK extent data file, relative to the location of the VMDK descriptor file
| <td colspan="2">*Optional*</td>
| 5th | The extent start sector
| <td colspan="2">*Seen in VMWare Player 9 in combination with a physical device extent on Windows*</td>
| 6th and 7th | "partitionUUID" followed by a device identifier

The extent offset is specified only for flat extents and corresponds to the
offset in the file or device where the extent data is located. For
device-backed virtual disks (physical or raw disks) the extent offset can be
non-zero. For raw extent data files the extent offset should be zero.

#### Extent access mode {#extent_access_mode}

The extent access mode consists of the following values:

| Value | Description
| --- | ---
| NOACCESS | No access
| RDONLY | Read only
| RW | Read write

#### Extent types {#extent_types}

The extent type consists of the following values:

| Value | Description
| --- | ---
| FLAT | raw extent data file
| SPARSE | VMDK sparse extent data file
| ZERO | Sparse extent that consists of 0-byte values
| VMFS | raw extent data file
| VMFSSPARSE | COWD sparse extent data file
| VMFSRDM | Unknown (Physical disk device that uses RDM?)
| VMFSRAW | Unknown (Physical disk device?)

> Note that VMWare Player 9 has been observed to use "FLAT" for Windows devices

### Change tracking file section

The change tracking file section was introduced in version 3 and looks similar to:

```
# Change Tracking File
changeTrackPath="test-flat.vmdk"
```

The change tracking file section consists of the following values:

| Value | Description
| --- | ---
| "# Change Tracking File" | Section header
| changeTrackPath | Unknown (The path to the change tracking file?)

### Disk database

The disk data base of a descriptor file looks similar to the data below.

```
# The Disk Data Base
#DDB

ddb.virtualHWVersion = "4"
ddb.geometry.cylinders = "16383"
ddb.geometry.heads = "16"
ddb.geometry.sectors = "63"
ddb.adapterType = "ide"
ddb.toolsVersion = "0"
```

The disk data base consists of the following values:

| Value | Description
| --- | ---
| "# The Disk Data Base" | Section header
| "#DDB" | Currently assumed to be part of the section header
| ddb.deletable | Unknown (seen: "true")
| ddb.virtualHWVersion | The virtual hardware version. For VMWare Player and Workstation this seems to correspond with the application version
| ddb.longContentID | The long content identifier, which contains a 128-bit base16 encoded value, without spaces
| ddb.uuid | UUIDm which contains a 128-bit base16 encoded value, with spaces between bytes
| ddb.geometry.cylinders | The number of cylinders
| ddb.geometry.heads | The number of heads
| ddb.geometry.sectors | The number of sectors
| ddb.geometry.biosCylinders | The number of cylinders as reported by the BIOS.
| ddb.geometry.biosHeads | The number of heads as reported by the BIOS.
| ddb.geometry.biosSectors | The number of sectors as reported by the BIOS.
| ddb.adapterType | [Disk adapter type](#disk_adapter_types)
| ddb.toolsVersion | String containing the version of the installed VMWare tools version
| ddb.thinProvisioned | Unknown (seen: "1")

VirtualBox has been observed to use a different case for "disk" in the section header:

```
# The disk Data Base
```

#### Virtual hardware version

| Value | Description
| --- | ---
| 4 | TODO: add description
| <td colspan="2">&nbsp;</td>
| 6 | TODO: add description
| 7 | TODO: add description
| <td colspan="2">&nbsp;</td>
| 9 | VMWare Player/Workstation 9.0

#### Disk adapter types {#disk_adapter_types}

| Value | Description
| --- | ---
| ide | TODO: add description
| buslogic | TODO: add description
| lsilogic | TODO: add description
| legacyESX | TODO: add description

The buslogic and lsilogic values are for SCSI disks and show which virtual SCSI
adapter is configured for the virtual machine. The legacyESX value is for older
ESX Server virtual machines when the adapter type used in creating the virtual
machine is not known.

## The raw extent data file

The raw extent data file contains the actual disk data. The raw extent data
file can be a file or a device.

This type of extent data file is also known as "Simple" or "Flat Extent".

## The VMDK sparse extent data file

The VMDK sparse extent data file contains the actual disk data. A VMDK sparse
extent data file consists of:

* file header
* optional embedded descriptor file
* optional secondary grain directory
  * optional secondary grain tables
* (primary) grain directory
  * (primary) grain tables
* grains
* optional backup file header

This type of extent data file is also known as "Hosted Sparse Extent" or
"Stream-Optimized Compressed Sparse Extent" when markers are used.

> Note that the actual layout can vary per file, Stream-Optimized Compressed
> Sparse Extent have been observed to use secondary file headers.

Changes in format version 2:

* added encrypted disk support (though this feature never seem to never have been implemented).

Changes in format version 3:

* the size of extent files is no longer limited to 2 GiB;
* added support for persistent changed block tracking (CBT).

> Note that "CBT", the changeTrackPath value in the descriptor file references
> a file that describes changed areas on the virtual disk.

### File header

The file header is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "KDMV" | Signature
| 4 | 4 | 1, 2 or 3 | Format version
| 8 | 4 | | [Flags](#vmdk_extent_file_flags)
| 12 | 8 | | Maximum data number of sectors (capacity)
| 20 | 8 | | Sectors per grain, which must be a power of 2 and > 8
| 28 | 8 | | [Embedded descriptor file](#descriptor_file) start sector, which is relative from the start of the file or 0 if not set.
| 36 | 8 | | [Embedded descriptor file](#descriptor_file) size in sectors
| 44 | 4 | 512 | The number of grains table entries
| 48 | 8 | | Secondary grain directory start sector, which is relative from the start of the file or 0 if not set.
| 56 | 8 | | Primary grain directory start sector, which is relative from the start of the file, 0 if not set or 0xffffffffffffffff (GD_AT_END) if relative from the end of the file.
| 64 | 8 | | Metadata size in sectors
| 72 | 1 | | Value to determine if the extent data file was cleanly closed (or dirty flag)
| 73 | 1 | '\n' | Single end of line character
| 74 | 1 | ' ' | Non end of line character
| 75 | 1 | '\r' | First double end of line character
| 76 | 1 | '\n' | Second double end of line character
| 77 | 2 | | [Compression method](#vmdk_compression_method)
| 79 | 433 | 0 | Unknown (Padding)

The end of line characters are used to detect corruption due to file transfers
that alter line end characters.

> According to Virtual Disk Format 5.0 specification the maximum data number of 
> sectors (capacity) should be a multitude of the sectors per grain. Note that
> it has been observed that this is not always the case.

If the primary grain directory start sector is 0xffffffffffffffff (GD_AT_END)
in a Stream-Optimized Compressed Sparse Extent there should be a secondary
file header stored at offset -1024 relative from the end of the file (stream)
that contains the correct grain directory start sector.

#### Flags {#vmdk_extent_file_flags}

The flags consist of the following values:

| Value | Identifier | Description
| --- | --- | --- 
| 0x00000001 | | Valid new line detection test
| 0x00000002 | | Use secondary grain directory. The secondary (redundant) grain directory should be used instead of the primary grain directory.
| <td colspan="3">*As of format version 2*</td>
| 0x00000004 | | Use zeroed-grain table entry. The zeroed-grain table entry overloads grain data sector number 1 to indicate the grain is sparse.
| <td colspan="3">*Common*</td>
| 0x00010000 | | Has compressed grain data
| 0x00020000 | | Contains metadata, where the file contains markers to identify metadata or data blocks.

#### Compression method {#vmdk_compression_method}

The compression method consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| 0x00000000 | COMPRESSION_NONE | No compression
| 0x00000001 | COMPRESSION_DEFLATE | Compression using Deflate (RFC1951)

### Markers

The markers are used in Stream-Optimized Compressed Sparse Extents. The
corresponding flag must be set for markers to be present. An example of the
layout of a Stream-Optimized Compressed Sparse Extent that uses markers is:

* file header
* embedded descriptor
* compressed grain markers
* grain table marker
* grain table
* grain directory marker
* grain directory
* footer marker
* secondary file header
* end-of-stream marker

### The marker

The marker is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Value
| 8 | 4 | | Marker data size
| <td colspan="4">*If marker data size equals 0*</td>
| 12 | 4 | | [Marker type](#vmdk_extent_file_marker_types)
| 16 | 496 | 0 | Unknown (Padding)
| <td colspan="4">*If marker data size > 0*</td>
| 12 | ... | | Compressed grain data

If the marker data size > 0 the marker is a compressed grain marker.

#### Marker types {#vmdk_extent_file_marker_types}

| Value | Identifier | Description
| --- | --- | ---
| 0x00000000 | MARKER_EOS | End-of-stream marker
| 0x00000001 | MARKER_GT | Grain table (metadata) marker
| 0x00000002 | MARKER_GD | Grain directory (metadata) marker
| 0x00000003 | MARKER_FOOTER | Footer (metadata) marker

#### Compressed grain marker

The compressed grain marker indicates that compressed data follows.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4">*Compressed grain header*</td>
| 0 | 8 | 0 | Logical sector number
| 8 | 4 | | Compressed data size
| <td colspan="4">&nbsp;</td>
| 12 | ... | | Compressed data, which contains [Deflate compressed data](zlib.md#deflate_compressed_data)

> Note that the compressed grain data can be larger than the grain data size.

#### End of stream marker

The end-of-stream marker indicates the end of the virtual disk. Basically the
end-of-stream marker is an empty sector block.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | 0 | Value
| 8 | 4 | 0 | Marker data size
| 12 | 4 | MARKER_EOS | [Marker type](#vmdk_extent_file_marker_types)
| 16 | 496 | 0 | Unknown (Padding)

#### Grain table marker

The grain table marker indicates that a grain table follows the marker sector block.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | 0 | Value
| 8 | 4 | 0 | Marker data size
| 12 | 4 | MARKER_GT | [Marker type](#vmdk_extent_file_marker_types)
| 16 | 496 | 0 | Unknown (Padding)
| 512 | ... | | [Grain table](#vmdk_extent_file_grain_table)

#### Grain directory marker

The grain directory marker indicates that a grain directory follows the marker
sector block.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | 0 | Value
| 8 | 4 | 0 | Marker data size
| 12 | 4 | MARKER_GD | [Marker type](#vmdk_extent_file_marker_types)
| 16 | 496 | 0 | Unknown (Padding)
| 512 | ... | | [Grain directory](#vmdk_extent_file_grain_directory)

#### Footer marker

The footer marker indicates that a footer follows the marker sector block.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | 0 | Value
| 8 | 4 | 0 | Marker data size
| 12 | 4 | MARKER_FOOTER | [Marker type](#vmdk_extent_file_marker_types)
| 16 | 496 | 0 | Unknown (Padding)
| 512 | ... | | [Footer](#vmdk_extent_file_footer)

### Grain directory {#vmdk_extent_file_grain_directory}

The grain directory is also referred to as level 0 metadata.

The size of the grain directory is dependent on the number of grains in the
extent data file. The number of entries in the grain directory can be
determined as following:

```
grain table size = number of grain table entries x grain size
number of grain directory entries = maximum data size / grain table size

if( maximum data size % grain table size > 0 )
{
	number of entries += 1
}
```

The grain directory consists of 32-bit grain table offsets:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Grain table start sector, which is relative from the start of the file or 0 if sparse or the sector is stored in the parent image.

The grain directory is stored in a multitude of 512 byte sized blocks.

> Note that as of VMDK sparse extent data file version 2 if the "use
> zeroed-grain table entry" flag is set, a start sector of 1 indicates the
> grain table is sparse.

### Grain table {#vmdk_extent_file_grain_table}

The grain table is also referred to as level 1 metadata.

The size of the grain table is of variable size. The number of entries in the
grain table is stored in the file header. Note that the number of entries in
the last grain table is dependent on the maximum data size and not necessarily
the same as the value stored in the file header.

The grain directory consists of 32-bit grain table offsets:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Grain data sector number, which is relative from the start of the file or 0 if sparse or the sector is stored in the parent image.

The number of entries in a grain table and should be 512, therefore the size of
the grain table is 512 x 4 = 2048 bytes.

The grain table is stored in a multitude of 512 byte sized blocks.

> Note that as of VMDK sparse extent data file version 2 if the "use
> zeroed-grain table entry" flag is set, a sector number of 1 indicates the
> grain table is sparse.

### Grain data

In an uncompressed sparse extent data file the data is stored at the grain data
sector number.

In a compressed sparse extent data file every non-sparse grain is assumed to be
stored compressed.

#### Compressed grain data

The compressed grain data is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| <td colspan="4">*Compressed grain header*</td>
| 0 | 8 | 0 | Logical sector number
| 8 | 4 | | Compressed data size
| <td colspan="4">&nbsp;</td>
| 12 | ... | | Compressed data, which contains [zlib compressed data](zlib.md)
| ... | ... | | Unknown (Padding)

The uncompressed data size should be the grain size or less for the last grain.

### Footer {#vmdk_extent_file_footer}

The footer is only used in Stream-Optimized Compressed Sparse Extents. The
footer is the same as the file header. The footer should be the last block of
the disk and immediately followed by the end-of-stream marker so that they
together make up the last two sectors of the disk.

The header and footer differ in that the grain directory offset value in the
header is set to 0xffffffffffffffff (GD_AT_END) and in the footer to the correct
value.

### Changed block tracking (CBT)

TODO: complete section

## The COWD sparse extent data file

The copy-on-write disk (COWD) sparse extent data file contains the actual disk
data. The COW sparse extent data file consists of:

* file header
* grain directory
* grain tables
* grains

This type of extent data file is also known as ESX Server Sparse Extent.

### File header

The file header is 2048 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "COWD" | Signature
| 4 | 4 | 1 | Format version
| 8 | 4 | 0x00000003 | Unknown (Flags)
| 12 | 4 | | Maximum data number of sectors (capacity)
| 16 | 4 | | Sectors per grain
| 20 | 4 | 4 | Grain directory start sector, which is relative from the start of the file or 0 if not set.
| 24 | 4 | | Number of grain directory entries
| 28 | 4 | | The next free sector
| <td colspan="4">*In root extent data file*</td>
| 32 | 4 | | The number of cylinders
| 36 | 4 | | The number of heads
| 40 | 4 | | The number of sectors
| 44 | 1016 | | Unknown (Empty values)
| <td colspan="4">*In child extent data files*</td>
| 32 | 1024 | | Parent filename
| 1056 | 4 | | Parent generation
| <td colspan="4">*Common*</td>
| 1060 | 4 | | Generation
| 1064 | 60 | | Name
| 1124 | 512 | | Description
| 1636 | 4 | | Saved generation
| 1640 | 8 | | Unknown (Reserved)
| 1648 | 4 | | Value to determine if the extent data file was cleanly closed (or dirty flag)
| 1652 | 396 | | Unknown (Padding)

> Note that the parent filename seems not to be set in recent delta sparse
> extent files.

### Grain directory

The grain directory is also referred to as level 0 metadata.

The size of the grain directory is dependent on the number of grains in the
extent data file. The number of entries in the grain directory is stored in the
file header.

The grain directory consists of 32-bit grain table offsets:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Grain table start sector, which is relative from the start of the file or 0 if not set.

The grain directory is stored in a multitude of 512 byte sized blocks. Unused
bytes are set to 0.

### Grain table

The grain table is also referred to as level 1 metadata.

The size of the grain table is of variable size. The number of entries in a
grain table is the fixed value of 4096.

The grain directory consists of 32-bit grain table offsets:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Grain sector number, which is relative from the start of the file or 0 if not set.

The grain table is stored in a multitude of 512 byte sized blocks. Unused bytes
are set to 0.

## Change tracking file

TODO: complete section

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "\xa2\x72\x19\xf6" | Unknown (signature?)
| 4 | 4 | 1 | Unknown (version?)
| 8 | 4 | | Unknown (empty values)
| 12 | 4 | 0x200 | Unknown
| 16 | 8 | | Unknown
| 24 | 8 | | Unknown
| 32 | 4 | | Unknown
| 36 | 4 | | Unknown
| 40 | 4 | | Unknown
| 44 | 16 | | Unknown (UUID?)
| 60 | ... | | Unknown (empty values?)

## Corruption scenarios

The total size specified by the number of grain table entries is lager than
size specified by the maximum number of sectors. Seen in VMDK images generated
by qemu-img.

## Notes

The markers can be used to scan for the individual parts of the VMDK sparse
extent data file if the stream has been truncated, but not that this can be
very expensive process IO-wise.

## References

* [Virtual Disk Format 5.0](https://web.archive.org/web/20120302211605/http://www.vmware.com/support/developer/vddk/vmdk_50_technote.pdf), by VMWare
