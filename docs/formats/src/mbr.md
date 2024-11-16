# Master Boot Record (MBR) partition table format

The Master Boot Record (MBR) partition table is mainly used on the family of
Intel x86 based computers.

## Overview

A MBR partition table consists of:

* Master Boot Record (MBR)
* Extended Partition Records (EPRs)

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | N/A
| Character strings | N/A

### Terminology

| Term | Description
| --- | ---
| Physical block | A fixed location on the storage media defined by the storage media.
| Logical block | An abstract location on the storage media defined by software.

### Sector size(s)

Traditionally the size of sector is 512 bytes, but modern hard disk drives use
4096 bytes. The linux fdisk utility supports sector sizes of: 512, 1024, 2048
and 4096.

The location of of the "boot signature" of the MBR does not indicate the sector
size. Methods to derive the sector size from the data:

* check the "boot signature" of the first EPR, if present
* check the content of well known partition types

## Cylinder Head Sector (CHS) address

The Cylinder Head Sector (CHS) address is 24 bits in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0  | 8 bits | | Head
| 1.0  | 6 bits | | Sector
| 1.5 | 10 bits | | Cylinder

The logical block address (LBA) can be determined from the CHS with the
following calculation:

```
LBA = ( ( ( cylinder * heads per cylinder ) + head ) * sectors per track ) + sector - 1
```

## The Master Boot Record (MBR)

The Master Boot Record (MBR) is a data structure that describes the properties
of the storage medium and its partitions.

The classical MBR can only contain 4 partition table entries. Additional
partition entries must be stored using extended partition records (EPR). The
classical MBR has evolved into different variants like:

* The modern MBR
* The Advanced Active Partitions (AAP) MBR
* The NEWLDR MBR
* The AST/NEC MS-DOS and SpeedStor MBR
* The Disk Manager MBR

### The classical MBR

The classical MBR is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 446 | | The boot (loader) code
| 446 | 16 | | Partition table entry 1
| 462 | 16 | | Partition table entry 2
| 478 | 16 | | Partition table entry 3
| 494 | 16 | | Partition table entry 4
| 510 | 2 | "\x55\xaa" | The (boot) signature

### The modern MBR

The modern MBR is 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 218 | | The first part of the boot (loader) code
| <td colspan="4"> *Disk timestamp* used by Microsoft Windows 95, 98 and ME
| **218** | **2** | **0x0000** | Unknown (Reserved)
| **220** | **1** | | Unknown (Original physical drive), which contains a value that ranges from 0x80 to 0xff, where 0x80 is the first drive, 0x81 the second, etc.
| **221**| **1** | | **Seconds**, which contains a value that ranges from 0 to 59
| **222**| **1** | | **Minutes**, which contains a value that ranges from 0 to 59
| **223**| **1** | | **Hours**, which contains a value that ranges from 0 to 23
| <td colspan="4"> *Without disk identity*
| 224 | 222 | | The second part of the boot (loader) code
| <td colspan="4"> *With disk identity*, used by UEFI, Microsoft Windows NT or later
| 224 | 216 | | The second part of the boot (loader) code
| **440** | **4** | | **Disk identity (signature)**
| **444** | **2** | **0x0000** or **0x5a5a** | **copy-protection marker**
| <td colspan="4"> *Common*
| 446 | 16 | | Partition table entry 1
| 462 | 16 | | Partition table entry 2
| 478 | 16 | | Partition table entry 3
| 494 | 16 | | Partition table entry 4
| 510 | 2 | "\x55\xaa" | The (boot) signature

## The extended partition record

The extended partition record (EPR) (also referred to as extended boot record
(EBR)) starts with a 64 byte (extended) partition record (EPR) like the MBR.
This partition table contains information about the logical partition (volume)
and additional extended partition tables.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 446 | 0x00 | Unknown (Unused), which should contain zero bytes
| 446 | 16 | | Partition table entry 1
| 462 | 16 | | Partition table entry 2, which should contain an extended partition
| 478 | 16 | 0x00 | Partition table entry 3, which should be unused and contain zero bytes
| 494 | 16 | 0x00 | Partition table entry 4, which should be unused and contain zero bytes
| 510 | 2 | "\x55\xaa" | Signature

The second partition entry contains an extended partition which points to the
next EPR. The LBA addresses in the EPR are relative to the start of the first
EPR.

The first EPR typically has a [partition type](#partition_types) of 0x05 but
certain version of Windows are known to use a partition type 0x0f, such as
Windows 98.

## The partition table entry

The partition table entry is 16 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | | [Partition flags](#partition_flags)
| 1 | 3 | | The partition start address, which contains a CHS relative from the start of the harddisk
| 4 | 1 | | [Partition type](#partition_types)
| 5 | 3 | | The partition end address, which contains a CHS relative from the start of the harddisk
| 8 | 4 | | The partition start address, which contains a LBA (sectors) relative from the start of the harddisk
| 12 | 4 | | Size of the partition in number of sectors

### <a name="partition_flags"></a>Partition flags

The partition flags consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| 0x80 | | Partition is boot-able

### <a name="partition_types"></a>Partition types

The partition types consist of the following values:

| Value | Identifier | Description
| --- | --- | ---
| 0x00 | | Empty
| 0x01 | | FAT12 (CHS)
| 0x02 | | XENIX root
| 0x02 | | XENIX user
| 0x04 | | FAT16 (16 MiB -32 MiB CHS)
| 0x05 | | Extended (CHS)
| 0x06 | | FAT16 (32 MiB - 2 GiB CHS)
| 0x07 | | HPFS/NTFS
| 0x08 | | AIX
| 0x09 | | AIX bootable
| 0x0a | | OS/2 Boot Manager
| 0x0b | | FAT32 (CHS)
| 0x0c | | FAT32 (LBA)
| | |
| 0x0e | | FAT16 (32 MiB - 2 GiB LBA)
| 0x0f | | Extended (LBA)
| 0x10 | | OPUS
| 0x11 | | Hidden FAT12 (CHS)
| 0x12 | | Compaq diagnostics
| | |
| 0x14 | | Hidden FAT16 (16 MiB - 32 MiB CHS)
| | |
| 0x16 | | Hidden FAT16 (32 MiB - 2 GiB CHS)
| 0x17 | | Hidden HPFS/NTFS
| 0x18 | | AST SmartSleep
| | |
| 0x1b | | Hidden FAT32 (CHS)
| 0x1c | | Hidden FAT32 (LBA)
| | |
| 0x1e | | Hidden FAT16 (32 MiB - 2 GiB LBA)
| | |
| 0x24 | | NEC DOS
| | |
| 0x27 | | Unknown (PackardBell recovery/installation partition)
| | |
| 0x39 | | Plan 9
| | |
| 0x3c | | PartitionMagic recovery
| | |
| 0x40 | | Venix 80286
| 0x41 | | PPC PReP Boot
| 0x42 | | SFS or LDM: Microsoft MBR (Dynamic Disk)
| | |
| 0x4d | | QNX4.x
| 0x4e | | QNX4.x 2nd part
| 0x4f | | QNX4.x 3rd part
| 0x50 | | OnTrack DM
| 0x51 | | OnTrack DM6 Aux1
| 0x52 | | CP/M
| 0x53 | | OnTrack DM6 Aux3
| 0x54 | | OnTrackDM6
| 0x55 | | EZ-Drive
| 0x56 | | Golden Bow
| | |
| 0x5c | | Priam Edisk
| | |
| 0x61 | | SpeedStor
| | |
| 0x63 | | GNU HURD or SysV
| 0x64 | | Novell Netware 286
| 0x65 | | Novell Netware 386
| | |
| 0x70 | | DiskSecure Multi-Boot
| | |
| 0x75 | | PC/IX
| | |
| 0x78 | | XOSL
| | |
| 0x80 | | Old Minix
| 0x81 | | Minix / old Linux
| 0x82 | | Solaris x86 or Linux swap
| 0x83 | | Linux
| 0x84 | | Hibernation or OS/2 hidden C: drive
| 0x85 | | Linux extended
| 0x86 | | NTFS volume set
| 0x87 | | NTFS volume set
| | |
| 0x8e | | Linux LVM
| | |
| 0x93 | | Amoeba
| 0x94 | | Amoeba BBT
| | |
| 0x9f | | BSD/OS
| 0xa0 | | IBM Thinkpad hibernation
| 0xa1 | | Hibernation
| | |
| 0xa5 | | FreeBSD
| 0xa6 | | OpenBSD
| 0xa7 | | NeXTSTEP
| 0xa8 | | Mac OS X
| 0xa9 | | NetBSD
| | |
| 0xab | | Mac OS X Boot
| | |
| 0xaf | | Mac OS X
| | |
| 0xb7 | | BSDI
| 0xb8 | | BSDI swap
| | |
| 0xbb | | Boot Wizard hidden
| | |
| 0xc1 | | DRDOS/sec (FAT-12)
| | |
| 0xc4 | | DRDOS/sec (FAT-16 < 32M)
| | |
| 0xc6 | | DRDOS/sec (FAT-16)
| 0xc7 | | Syrinx
| | |
| 0xda | | Non-FS data
| 0xdb | | CP/M / CTOS / ...
| | |
| 0xde | | Dell Utility
| 0xdf | | BootIt
| | |
| 0xe1 | | DOS access
| | |
| 0xe3 | | DOS R/O
| 0xe4 | | SpeedStor
| | |
| 0xeb | | BeOS
| | |
| 0xee | | EFI GPT protective partition
| 0xef | | EFI system partition (FAT)
| 0xf0 | | Linux/PA-RISC boot
| 0xf1 | | SpeedStor
| 0xf2 | | DOS secondary
| | |
| 0xf4 | | SpeedStor
| | |
| 0xfb | | VMWare file system
| 0xfc | | VMWare swap
| 0xfd | | Linux RAID auto-detect
| 0xfe | | LANstep
| 0xff | | BBT
