# Apple Partition Map (APM) format

The Apple Partition Map (APM) format is used on Motorola based Macintosh
computers. On Intel based Macintosh computers the [GUID Partition Table (GPT) format](gpt.md)
is used.

## Overview

An Apple Partition Map (APM) consists of:

* a drive descriptor
* partition map entry of type "Apple_partition_map"
* zero or more partition map entries

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | big-endian
| Date and time values | N/A
| Character strings | ASCII

### Terminology

| Term | Description
| --- | ---
| Physical block | A fixed location on the storage media defined by the storage media.
| Logical block | An abstract location on the storage media defined by software.

## The drive descriptor

The driver descriptor identifies the device drivers installed on a storage
medium. The driver descriptor can contain refer to multiple device drivers.
Every device driver is stored in a separate partition.

The drive descriptor is situated in the first block of the storage medium. This
block is referred to as the device driver block. The driver descriptor block is
not considered part of any partition.

The drive descriptor is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | "\x45\x52" or "ER" | Signature
| 2 | 2 | | The block size of the device in bytes
| 4 | 4 | | The number of blocks on the device
| 8 | 2 | | Device type (Reserved)
| 10 | 2 | | Device identifier (Reserved)
| 12 | 4 | | Device data (Reserved)
| 16 | 2 | | The number of driver descriptors
| 18 | 8 | | The first device driver descriptor
| 26 | 484 | | Additional driver descriptors, where unused entries are 16-bit integer values filled with 0

### The device driver descriptor

The device driver descriptor is 8 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Start block of the device driver
| 4 | 2 | | Device driver number of blocks
| 6 | 2 | | Operating system type, where is 1 represents "Mac OS"

## The partition map

The partition map is stored after the drive descriptor. The partition map
consists of multiple entries that must be stored continuously. The partition
map itself is considered a partition therefore the first entry in the partition
map describes the partition map itself.

### The partition map entry

A partition map entry is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | "\x50\x4d" or "PM" | Signature
| 2 | 2 | 0x00 | Unknown (Reserved)
| 4 | 4 | | Total number of entries in the partition map
| 8 | 4 | | Partition start sector
| 12 | 4 | | Partition number of sectors
| 16 | 32 | | Partition name, which contains an ASCII string
| 48 | 32 | | [Partition type](#partition_types), which contains an ASCII string
| 80 | 4 | | Data area start sector
| 84 | 4 | | Data area number of sectors
| 88 | 4 | | [Status flags](#status_flags)
| 92 | 4 | | Boot code start sector
| 96 | 4 | | Boot code number of sectors
| 100 | 4 | | Boot code address
| 104 | 4 | | Unknown (Reserved)
| 108 | 4 | | Boot code entry point
| 112 | 4 | | Unknown (Reserved)
| 116 | 4 | | Boot code checksum
| 120 | 16 | | Processor type
| 136 | ( 188 x 2 ) = 376 | 0x00 | Unknown (Reserved)

> Note that the partition name can be empty.

### <a name="partition_types"></a>Partition types

The partition types consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| "Apple_Boot" | |
| "Apple_Boot_RAID" | |
| "Apple_Bootstrap" | |
| "Apple_Driver" | |
| "Apple_Driver43" | |
| "Apple_Driver43_CD" | |
| "Apple_Driver_ATA" | |
| "Apple_Driver_ATAPI" | |
| "Apple_Driver_IOKit" | |
| "Apple_Driver_OpenFirmware" | |
| "Apple_Extra" | |
| "Apple_Free" | |
| "Apple_FWDriver" | |
| "Apple_HFS" | |
| "Apple_HFSX" | |
| "Apple_Loader" | |
| "Apple_MDFW" | |
| "Apple_MFS" | |
| "Apple_partition_map" | |
| "Apple_Patches" | |
| "Apple_PRODOS" | |
| "Apple_RAID" | |
| "Apple_Rhapsody_UFS" | |
| "Apple_Scratch" | |
| "Apple_Second" | |
| "Apple_UFS" | |
| "Apple_UNIX_SVR2" | |
| "Apple_Void" | |
| "Be_BFS" | |
| "MFS" | |

### <a name="status_flags"></a>Status flags

The partition status flags consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | | Is valid
| 0x00000002 | | Is allocated
| 0x00000004 | | Is in use
| 0x00000008 | | Contains boot information
| 0x00000010 | | Is readable
| 0x00000020 | | Is writable
| 0x00000040 | | Boot code is position independent
| | |
| 0x00000100 | | Contains a chain-compatible driver
| 0x00000200 | | Contains a real driver
| 0x00000400 | | Contains a chain driver
| | |
| 0x40000000 | | Automatic mount at startup
| 0x80000000 | | Is startup partition

> Note that the "is in use" status flags does not appear to be used consistently.
