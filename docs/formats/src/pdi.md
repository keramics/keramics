# Parallels Disk Image (PDI) format

The Parallels Disk Image format used in Parallels virtualization products as
one of its image formats. It is both used the store hard disk images and
snapshots.

## Overview

A Parallels Disk Image consists of a directory, typically named "{NAME}.hdd"
containing:

* Descriptor file (DiskDescriptor.xml) and backup (DiskDescriptor.xml.Backup)
* {NAME}.hdd file
* Storage data file ({NAME}.hdd.0.{GUID}.hds)
* {NAME}.hdd.drh

Where {NAME} is an arbitrary name and {GUID} is a unique identifier.

### Disk types

The Parallels Disk Image format support multiple disk types:

| Identifier | Description
| --- | ---
| Expanding | Disk that consists of a single (dynamic size) sparse storage data file
| Plain | Disk that consists of a single single (fixed size) raw storage data file
| Split | Disk that consists of a one or more split storage data files, either expanding or plain, holding upto 2G of data

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Character strings | UTF-8 by default, the encoding is defined in the disk descriptor XML file.

The number of bytes per sector is 512.

## Descriptor file

The DiskDescriptor.xml and its backup (DiskDescriptor.xml.Backup) contain
the "Parallels_disk_image" XML element tha consists of the following values:

| Identifier | Description
| --- | ---
| Disk_Parameters | The disk parameters
| StorageData | Information about the storage data files
| Snapshots | Information about snapshots

```xml
<?xml version='1.0' encoding='UTF-8'?>
<Parallels_disk_image Version="1.0">
    <Disk_Parameters>
        <Disk_size>134217728</Disk_size>
        <Cylinders>262144</Cylinders>
        <PhysicalSectorSize>4096</PhysicalSectorSize>
        <LogicSectorSize>512</LogicSectorSize>
        <Heads>16</Heads>
        <Sectors>32</Sectors>
        <Padding>0</Padding>
        <Encryption>
            <Engine>{00000000-0000-0000-0000-000000000000}</Engine>
            <Data></Data>
        </Encryption>
        <UID>{GUID}</UID>
        <Name>{NAME}</Name>
        <Miscellaneous>
            <CompatLevel>level2</CompatLevel>
            <Bootable>1</Bootable>
            <ChangeState>0</ChangeState>
            <SuspendState>0</SuspendState>
        </Miscellaneous>
    </Disk_Parameters>
    <StorageData>
        <Storage>
            <Start>0</Start>
            <End>134217728</End>
            <Blocksize>2048</Blocksize>
            <Image>
                <GUID>{GUID}</GUID>
                <Type>Compressed</Type>
                <File>{NAME}.hdd.0.{GUID}.hds</File>
            </Image>
            ...
        </Storage>
        ...
    </StorageData>
    <Snapshots>
        <Shot>
            <GUID>{GUID}</GUID>
            <ParentGUID>{GUID}</ParentGUID>
        </Shot>
        ...
    </Snapshots>
</Parallels_disk_image>
```

### Disk parameters

The disk parameters are stored in the "Disk_Parameters" XML element and
contains the following values.

| Identifier | Description
| --- | ---
| Cylinders | Number of cylinders
| Disk_size | Disk size, in number of sectors
| Encryption | "Encryption" sub XML element
| Heads | Number of heads
| Miscellaneous | "Miscellaneous" sub XML element
| Name | Name of the disk
| LogicSectorSize | Logical sector size
| Padding | Unknown (padding)
| PhysicalSectorSize | Physical sector size
| Sectors | Number of sectors per cylinder
| UID | Unknown (identifier)

#### Encryption

```xml
<Encryption>
    <Engine>{00000000-0000-0000-0000-000000000000}</Engine>
    <Data></Data>
    <Salt></Salt>
</Encryption>
```

#### Miscellaneous

```xml
<Miscellaneous>
    <CompatLevel>level2</CompatLevel>
    <Bootable>1</Bootable>
    <ChangeState>0</ChangeState>
    <SuspendState>0</SuspendState>
    <DupBlocksCnt>0</DupBlocksCnt>
    <CorruptBlocksCnt>0</CorruptBlocksCnt>
    <UnrefBlocksCnt>0</UnrefBlocksCnt>
    <OutOfDiskBlocksCnt>0</OutOfDiskBlocksCnt>
    <BatOverlapBlocksCnt>0</BatOverlapBlocksCnt>
    <BlocksCnt>0</BlocksCnt>
    <TruncatedBlocksCnt>0</TruncatedBlocksCnt>
    <ReferencedBlocksCnt>0</ReferencedBlocksCnt>
    <ShutdownState>0</ShutdownState>
    <GuestToolsVersion>17.1.1-51537</GuestToolsVersion>
</Miscellaneous>
```

##### CompatLevel

Seen: level0 and level2

### Storage data

The "StorageData" XML element contains the following values.

| Identifier | Description
| --- | ---
| Storage | One or more "Storage" XML sub elements

> Note that a split disks contains multiple "Storage" XML sub elements.

#### Storage

The "Storage" XML element contains the following values.

| Identifier | Description
| --- | ---
| Start | Start sector number of the segment stored in the storage data file
| End | End sector number of the segment stored in the storage data file
| Blocksize | Block size, in number of sectors
| Image | One or more "Image" sub XML elements

##### Image

The "Image" XML element contains the following values.

| Identifier | Description
| --- | ---
| GUID | Identifier of snapshot (or layer)
| Type | [Storage data file type](#storage_data_file_types)
| File | Name (or path) of the storage data file

### Snapshots data

The "Snapshots" XML element contains the following values.

| Identifier | Description
| --- | ---
| Shot | One or more "Shot" sub XML elements

#### Shot

The "Shot" XML element contains the following values.

| Identifier | Description
| --- | ---
| GUID | Identifier of snapshot (or layer)
| ParentGUID | Identifier of parent snapshot (or layer), which contains "{00000000-0000-0000-0000-000000000000}" if not set

## Storage data file

### Storage data file types {#storage_data_file_types}

| Value | Description
| --- | ---
| "Compressed" | Sparse storage data file
| "Plain" | Raw storage data file

## Raw storage data file

The raw (or plain) storage data file contains the disk image data including
free space.

### Sparse storage data file

The sparse storage data file contains the actual disk image data without free
space.

A sparse storage data file consists of:

* file header
* block allocation table (BAT)
* data blocks

#### Sparse storage data file header

The sparse storage data file header is 64 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | "WithoutFreeSpace" or "WithouFreSpacExt" | Signature
| 16 | 4 | 2 | Format version
| 20 | 4 | | Number of heads
| 24 | 4 | | Number of cylinders
| 28 | 4 | | Block size (or number of tracks) in number of sectors
| 32 | 4 | | Number of blocks, which is equivalent to the number of block allocation table entries
| 36 | 8 | | Number of sectors
| 44 | 4 | | Unknown (Creator?), seen: "\x00\x00\x00\x00", "pd17", "pd22"
| 48 | 4 | | Data start sector number, which is relative to the start of the sparse storage data file
| 52 | 4 | | Unknown (Flags?)
| 56 | 8 | | Unknown (Features start sector?)

#### Block allocation table (BAT)

The block allocation table consists of 32-bit entries. An entry contains the
sector number where the data block starts is set to 0 if the block is sparse or
stored in the parent disk image.

For example block allocation table entry 0 corresponds to disk image offset 0.
If contains a value of 0x800 the corresponding data block is stored at file
offset 0x100000 (0x800 x 512).
