# File Allocation Table (FAT) file system format

The File Allocation Table (FAT) is widely used a file sytem.

## Overview

The File Allocation Table (FAT) is widely used a file sytem. There are multiple
known variants or derivatives of FAT, such as:

* (original) 8-bit FAT
* FAT-12
* FAT-16
* FAT-32
* [exFAT](exfat.md)

A FAT file system consists of:

* One or more reserved sectors
  * a boot record (or boot sector)
  * file system informartion for FAT-32
* One or more cluster block allocation tables
* Root directory data for FAT-12 and FAT-16
* File and directory data

> Note that FAT-32 stores the root directory as part of the file and directory
> data.

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | little-endian
| Date and time values | FAT date and time
| Character strings | A narrow character Single Byte Character (SBC) ASCII string.

### Terminology

| Term | Description
| --- | ---
| Hidden sectors | The sectors stored before the FAT volume, such as those used to store a parition table.

### Determing the FAT format version

To distinguish between FAT-12, FAT-16 and FAT-32, compute the number of clusters
in the data area:

```
data area size = total number of sectors - ( number of reserved sectors s tables + size of root directory )
```

```
number of clusters = round down ( data area size / sectors per cluster )
```

* FAT-12 is used if the number of clusters is less than 4085
* FAT-16 is used if the number of clusters is less than 65525
* FAT-32 is used otherwise

## Boot record

The boot record is stored in the first sector of the volume.

### FAT-12 and FAT-16 boot record

The FAT-12 and FAT-16 boot record is at least 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 3 | "\xeb\x3c\x90" | Boot entry point (JMP +62, NOP)
| 3 | 8 | | File system signature (or OEM name)
| 11 | 2 | | Bytes per sector, which must be 512, 1024, 2048 or 4096
| 13 | 1 | | Sectors per cluster block, which must be 1, 2, 4, 8, 16, 32, 64 or 128
| 14 | 2 | | Number of reserved sectors (reserved region), which starts at the first sector of the volume (sector 0) and must be 1 or more (typically 1 or 32).
| 16 | 1 | | Number of cluster block allocation tables, which must be 1 or more (typically 2).
| 17 | 2 | | Number of root directory entries
| 19 | 2 | | Total number of sectors (16-bit)
| 21 | 1 | | [Media descriptor](#media_descriptors)
| 22 | 2 | | Cluster block allocation table size (16-bit) in number of sectors
| 24 | 2 | | Number of sectors per track
| 26 | 2 | | Number of heads
| 28 | 4 | | Number of hidden sectors
| 32 | 4 | | Total number of sectors (32-bit)
| 36 | 1 | | Drive number
| 37 | 1 | 0 | Unknown (reserved for Windows NT)
| 38 | 1 | | Extended boot signature
| <td colspan="4"> *If extended boot signature == 0x29*
| 39 | 4 | | Volume serial number
| 43 | 11 | | Volume label, which contains a narrow character string or "NO\x20NAME\x20\x20\x20\x20" if not set
| 54 | 8 | "FAT12\x20\x20\x20" or "FAT16\x20\x20\x20" | File system hint
| <td colspan="4"> *If extended boot signature != 0x29*
| 39 | 23 | | Unknown
| <td colspan="4"> *Common*
| 62 | 448 | | Used for boot code
| 510 | 2 | "\x55\xaa" | Sector signature

> Note that the sector signature must be set at offset 512 but can, in
> addition, it can be set in the last 2 bytes of the sector.

> Note that the volume serial number can be derived from the system current
> date and time.

> Note that the file system hint value is purely informational and not enforced
> by the format.

### FAT-32 boot record

The FAT-32 boot record is at least 512 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 3 | "\xeb\x58\x90" | Boot entry point (JMP +90, NOP)
| 3 | 8 | | File system signature (or OEM name)
| 11 | 2 | | Bytes per sector, which must be 512, 1024, 2048 or 4096
| 13 | 1 | | Sectors per cluster block, which must be 1, 2, 4, 8, 16, 32, 64 or 128
| 14 | 2 | | Number of reserved sectors (reserved region), which starts at the first sector of the volume (sector 0) and must be 1 or more (typically 1 or 32).
| 16 | 1 | | Number of cluster block allocation tables, which must be 1 or more (typically 2).
| 17 | 2 | 0 | Number of root directory entries, which must be 0 for FAT-32
| 19 | 2 | 0 | Total number of sectors (16-bit), which must be 0 for FAT-32
| 21 | 1 | | [Media descriptor](#media_descriptors)
| 22 | 2 | 0 | Cluster block allocation table size (16-bit) in number of sectors, which must be 0 for FAT-32
| 24 | 2 | | Number of sectors per track
| 26 | 2 | | Number of heads
| 28 | 4 | | Number of hidden sectors
| 32 | 4 | | Total number of sectors (32-bit)
| 36 | 4 | | Cluster block allocation table size (32-bit) in number of sectors, which must be non 0 for FAT-32
| 40 | 2 | | Extended flags
| 42 | 1 | 0 | Format revision minor number
| 43 | 1 | 0 | Format revision major number
| 44 | 4 | | Root directory start cluster
| 48 | 2 | | File system information (FSINFO) sector number
| 50 | 2 | | Boot record sector number
| 52 | 12 | 0 | Unknown (reserved)
| 64 | 1 | | Drive number
| 65 | 1 | 0 | Unknown (reserved for Windows NT)
| 66 | 1 | | Extended boot signature
| <td colspan="4"> *If extended boot signature == 0x29*
| 67 | 4 | | Volume serial number
| 71 | 11 | | Volume label, which contains a narrow character string or "NO\x20NAME\x20\x20\x20\x20" if not set
| 82 | 8 | "FAT32\x20\x20\x20" | File system hint
| <td colspan="4"> *If extended boot signature != 0x29*
| 67 | 23 | | Unknown
| <td colspan="4"> *Common*
| 90 | 420 | | Used for boot code
| 510 | 2 | "\x55\xaa" | Sector signature

> Note that the sector signature must be set at offset 512 but can, in
> addition, it can be set in the last 2 bytes of the sector.

> Note that the volume serial number can be derived from the system current
> date and time.

> Note that the file system hint value is purely informational and not enforced
> by the format.

### OEM names

| Value | Description
| --- | ---
| "MSWIN4.1" |
| "MSDOS 5.0" |

### <a name="media_descriptors"></a>Media descriptors

| Value | Identifier | Description
| --- | --- | ---
| 0xf0 | | removable media
| 0xf8 | | fixed (non-removable) media
| 0xf9 | |
| 0xfa | |
| 0xfb | |
| 0xfc | |
| 0xfd | |
| 0xfe | |
| 0xff | |

## Cluster block allocation table

A cluster block allocation table consists of:

* One ore more cluster block allocation table entries

### FAT 12 cluster block allocation table entry

A FAT 12 cluster block allocation table entry is 12 bits in size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 12 bits | | Data cluster number

Where the data cluster number has the following meanings:

| Value(s) | Description
| --- | ---
| 0x000 | Unused (free) cluster
| 0x001 | Unknown (invalid)
| 0x002 - 0xfef | Used cluster
| 0xff0 - 0xff6 | Reserved
| 0xff7 | Bad cluster
| 0xff8 - 0xfff | End of cluster chain

### FAT 16 cluster block allocation table entry

A FAT 16 cluster block allocation table entry is 16 bits in size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 bits | | Data cluster number

Where the data cluster number has the following meanings:

| Value(s) | Description
| --- | ---
| 0x0000 | Unused (free) cluster
| 0x0001 | Unknown (invalid)
| 0x0002 - 0xffef | Used cluster
| 0xfff0 - 0xfff6 | Reserved
| 0xfff7 | Bad cluster
| 0xfff8 - 0xffff | End of cluster chain

### FAT 32 cluster block allocation table entry

A FAT 32 cluster block allocation table entry is 32 bits in size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 32 bits | | Data cluster number

> Note that only the lower 28-bits are used

Where the data cluster number has the following meanings:

| Value(s) | Description
| --- | ---
| 0x00000000 | Unused (free) cluster
| 0x00000001 | Unknown (invalid)
| 0x00000002 - 0x0fffffef | Used cluster
| 0x0ffffff0 - 0x0ffffff6 | Reserved
| 0x0ffffff7 | Bad cluster
| 0x0ffffff8 - 0x0fffffff | End of cluster chain
| 0x10000000 - 0xffffffff | Unknown

## Directory

A directory consists of:

* self (".") directory entry (not used in root directory)
* parent ("..") directory entry (not used in root directory)
* Zero or more directory entries
* Terminator directory entry

### Directory entry

### Determining the root directory location

```
first allocation table offset = number of reserved sectors * bytes per sector
```

#### FAT-12 and FAT-16 root directory

```
root directory start offset = first allocation table offset + ( number of allocation tables * allocation table size * bytes per sector )
```

```
first cluster offset = directory start sector + ( number of root directory entries * 32 )
```

#### FAT-32 root directory

```
first cluster offset = first allocation table sector + ( number of allocation tables * allocation table size * bytes per sector )
```

```
root directory start offset = first cluster sector + ( ( root directory cluster - 2 ) * number of sectors per cluster )
```

#### FAT-12, FAT-16 and FAT-32 directory entry

A FAT-12, FAT-16 and FAT-32 directory entry is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Name, which is padded with spaces and the first character can have a special meaning
| 8 | 3 | | Extension, which is padded with spaces
| 11 | 1 | | [File attribute flags](#file_attribute_flags)
| 12 | 1 | | [Flags](#short_file_name_flags)
| 13 | 1 | | Creation time fraction of seconds, which contains fraction of 2-seconds in 10 ms intervals
| 14 | 2 | | Creation time
| 16 | 2 | | Creation date
| 18 | 2 | | Last access date
| 20 | 2 | | Unknown (OS/2 extended attribute), which is not used by FAT-12
| 22 | 2 | | Last modification time
| 24 | 2 | | Last modification date
| 26 | 2 | | Data stream start cluster
| 28 | 4 | | Data stream data size

### Short (or 8.3) file name

A FAT short (or 8.3) file name is stored in an OEM character set (codepage). The
[first character](#short_name_first_character) can have a special meaning.

Valid FAT short file name characters are:

| Value | Description
| --- | ---
| 'A-Z' | Upper case character
| '0-9' | Numeric character
| ' ' | Space, where trailing spaces are considered padding and therefore ignored.
| '.' | Dot, with the exception of "." and  "..". Trailing dot characters are ignored.
| '!' |
| '#' |
| '$' |
| '%' |
| '&' |
| '\'' | 
| '(' |
| ')' |
| '-' |
| '@' |
| '^' |
| '_' |
| '`' | 
| '{' |
| '}' |
| '~' |
| 0x80 - 0xff | Extended ASCII character, which are codepage dependent.

#### <a name="short_name_first_character"></a>First character

| Value | Description
| --- | ---
| 0x00 | Last (or terminator) directory entry
| 0x01 - 0x13 | VFAT long file name directory entry
| 0x05 | Directory entry pending deallocation (deprecated since DOS 3.0) or substitution of a 0xe5 extended ASCII character value
| 0x41 - 0x54 | Last VFAT long file name directory entry
| 0xe5 | Unallocated directory entry

### <a name="file_attribute_flags"></a>File attribute flags

| Value | Description
| --- | ---
| 0x01 | Read-only
| 0x02 | Hidden
| 0x04 | System
| 0x08 | Is volume label
| 0x10 | Is directory
| 0x20 | Archive
| 0x40 | Is device
| 0x80 | Unused (reserved)

### <a name="short_file_name_flags"></a>Flags

| Value | Description
| --- | ---
| 0x01 | Data is EFS encrypted
| 0x02 | Data contains large EFS header
| <td colspan="3"> &nbsp;
| 0x08 | Name should be represented in lower case
| 0x10 | Extension should be represented in lower case

### VFAT long file name entry

VFAT long file names entries are stored in directory entries. Multiple VFAT
long file name entries can be used to store a single long file name, where
the highest (last) sequence number is stored first. A maximum of 20 VFAT long
file name entries can be used to store a long file name of 255 UCS-2 characters.

VFAT long file names are stored using UCS-2 little-endian, which allows for
unpaired Unicode surrogates such as "U+d800" and "U+dc00"

VFAT long file name entries are stored before the directory entry containing
the short file name and additional file entry information.

A VFAT long file name entry is 32 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | | Sequence number
| 1 | 10 | | First name segment string, which contains 5 UCS-2 string characters
| 11 | 1 | 0x0f | Unknown (attributes)
| 12 | 1 | 0x00 | Unknown (type)
| 13 | 1 | | Checksum of the short (8.3) file name
| 14 | 12 | | Second name segment string, which contains 6 UCS-2 string characters
| 26 | 2 | 0 | Unknown (first cluster)
| 28 | 4 | | Third name segment string, which contains 2 UCS-2 string characters

> Note that unused characters in the VFAT long file segment strings after the
> end-of-string character (0x0000) are padded with 0xffff.

#### VFAT long file name sequence number

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 5 bits | | Number
| 0.5 | 1 bit | 0 | Unknown (reserved)
| 0.6 | 1 bit | 0 | Unknown (last logical, first physical LFN entry)
| 0.7 | 1 bit | 0 | Unknown

## References

* [Microsoft Extensible Firmware Initiative FAT32 File System Specification](http://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/fatgen103.doc), by Microsoft
* [Design of the FAT file system](https://en.wikipedia.org/wiki/Design_of_the_FAT_file_system), by Wikipedia
* [File Allocation Table](https://en.wikipedia.org/wiki/File_Allocation_Table), by Wikipedia
