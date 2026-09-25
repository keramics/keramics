# EWF 2 specification

In EnCase 7 Guidance Software introduced a version 2 of the
[Expert Witness Compression Format (EWF)](ewf.md). At the conceptual level both version 1 (EWF1)
and 2 (EWF2) are comparable. The data format is different.

This document will use EWF2 as the name of the format, although Guidance named the format "EnCase
Evidence File Format Version 2". The first observed version of EWF2 is 2.1.

## Overview

There are 2 different versions of EWF2:

* EWF2-Ex01; to store disk, volume and memory images.
* EWF2-Lx01; logical evidence file to store files and directories.

In EWF2 the data is either compressed or non-compressed, EWF2 no longer distinguishes between
multiple compression levels. Also support was added to encrypt the data and relevant metadata.

## Segment files

EWF2 stores data in one or more segment files (or segments). Each segment file consists of:

* A [file header](#file_header)
* One or more sections; which "EnCase Evidence File Format Version 2" refers to as link records and
  data.

EnCase 7 allows for the segment file size to be set at 30 MB at minimum and about 8.8 TB at maximum.

### File header {#file_header}

Each segment file starts with file header, the file header differs for EWF2-Ex01 and EWF2-Lx01.

#### EWF2-Ex01

The file header is 32 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "EVF2\r\n\x81\x00" | Signature |
| 8 | 1 | 2 | Major version |
| 9 | 1 | 1 | Minor version |
| 10 | 2 | | [Compression method](#compression_methods) |
| 12 | 4 | | Segment file number (Series) |
| 16 | 16 | | Segment file set identifier, which contains a little-endian version 4 GUID |

#### EWF2-Lx01

The file header is 32 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "LEF2\r\n\x81\x00" | Signature |
| 8 | 1 | 2 | Major version |
| 9 | 1 | 1 | Minor version |
| 10 | 2 | | [Compression method](#compression_methods) |
| 12 | 4 | | Segment file number (Series) |
| 16 | 16 | | Segment file set identifier, which contains a little-endian version 4 GUID |

#### Compression methods {#compression_methods}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | COMPRESSION_NONE | No compression |
| 1 | COMPRESSION_LZ | [zlib compression](zlib.md) ) |
| 2 | COMPRESSION_BZIP2 | BZip2 compression |

"EnCase Evidence File Format Version 2" states that "COMPRESSION_NONE will be never used", even so
EnCase 7 does not even seem to supports this compression method and indicates the file header is
corrupt.

> Note that EnCase 7 does not appear to provide an option to set the compression method to bzip2.

### Segment file extensions

#### EWF2-Ex01

* The first segment file has the extension '.Ex01'.
* The next segment file has the extension '.Ex02.
* This will continue up to '.Ex99'.
* After which the next segment file has the extension '.ExAA'.
  * The next segment file has the extension '.ExAA'.
  * This will continue up to '.ExAZ'.
  * The next segment file has the extension <!-- typos:disable -->'.ExBA'<!-- typos:enable -->.
  * This will continue up to '.ExZZ'.
  * The next segment file has the extension '.EyAA '.
  * This will continue up to '.EzZZ'.

Keramics supports extensions up to .EzZZ

TODO: determine what comes after .EzZZ

#### EWF2-Lx01

* The first segment file has the extension '.Lx01'.
* The next segment file has the extension '.Lx02.
* This will continue up to '.Lx99'.
* After which the next segment file has the extension '.LxAA'.
  * The next segment file has the extension '.LxAA'.
  * This will continue up to '.LxAZ'.
  * The next segment file has the extension <!-- typos:disable -->'.LxBA'<!-- typos:enable -->.
  * This will continue up to '.LxZZ'.
  * The next segment file has the extension '.LyAA '.
  * This will continue up to '.LzZZ'.

Keramics supports extensions up to .LzZZ

TODO: determine what comes after .LzZZ

## The sections

The remainder of the segment file consists of sections. Every section ends with a section
descriptor. In contrast to EWF1 the section descriptor is at the end of the section and the
section descriptor points to its previous section so the sections need to be read from
back-to-front.

### Section descriptor

The section descriptor is 64 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Section type](#section_types) |
| 4 | 4 | | [Data flags](#data_flags) |
| 8 | 8 | | Previous section offset, which is relative from the start of the segment file or 0 if there is no previous section |
| 16 | 8 | | Data size |
| 24 | 4 | | Section descriptor size |
| 28 | 4 | | 16-byte alignment padding size, in number of bytes |
| 32 | 16 | | Data integrity hash, which contains an MD5 of the data including padding. If the data is encrypted the integrity hash is calculated of the encrypted data |
| 48 | 3 x 4 = 12 | 0 | Unknown (Padding) |
| 60 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the section descriptor |

> Note that the data size includes the padding size. The padding is not always at the end of the
> section data, it can also be after a table header followed by more section data.

A section can contain additional data not defined by the data size. This was observed in the sector
data section of an EWF2 file that was aborted and restarted.

#### Section types {#section_types}

<!-- rumdl-disable MD033 MD056 -->

| Value | Identifier | Description |
| --- | --- | --- |
| <td colspan="4">*Defined by "EnCase Evidence File Format Version 2"*</td> |
| 0x00000001 | | Device information |
| 0x00000002 | | Case data |
| 0x00000003 | | Sector data |
| 0x00000004 | | Sector table |
| 0x00000005 | | Error table |
| 0x00000006 | | Session table |
| 0x00000007 | | Increment data |
| 0x00000008 | | MD5 hash |
| 0x00000009 | | SHA1 hash |
| 0x0000000a | | Restart data |
| 0x0000000b | | Encryption keys |
| 0x0000000c | | Memory extents table |
| 0x0000000d | | Next |
| 0x0000000e | | Final information |
| 0x0000000f | | Done |
| 0x00000010 | | Analytical data |
| <td colspan="4">*Not defined by "EnCase Evidence File Format Version 2"*</td> |
| 0x00000020 | | Single files data |
| 0x00000021 | | Single files unknown table |
| 0x00000022 | | Single files MD5 hash table |
| 0x00000023 | | Unknown (Single files unknown table) |

<!-- rumdl-enable MD033 MD056 -->

#### Data flags {#data_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | MD5HASHED | The data integrity hash is set |
| 0x00000002 | ENCRYPTED | The data is encrypted |

### Device information

The device information section can be found:

* in every segment file after the file header in EWF2-Ex01
* in every segment file after section 0x00000020 in EWF2-Lx01

When encryption is enabled the device information is encrypted.

The device information section contains a serialized object string that consist of:

| Line | Value | Description |
| --- | --- | --- |
| 1 | 1 | Number of objects |
| 2 | "main" | Object name |
| 3 | | Attribute tags |
| 4 | | Attribute values |
| 5 | | Empty line |

#### Attribute tags

| Identifier | Type | Description |
| --- | --- | --- |
| sn | Text | Drive serial number |
| md | Text | Drive model |
| lb | Text | Drive label |
| ts | Integer 64-bit | Number of sectors |
| hs | Integer 64-bit | Number of sectors of the HPA protected sectors |
| dc | Integer 64-bit | Number of sectors of the DCO protected sectors |
| dt | Enumeration | [Drive type](#drive_type) |
| pid | Integer 32-bit | Process identifier, which is set when the memory of an individual process is acquired |
| rs | Integer 32-bit | Number of sectors of a PALM RAM device |
| ls | Integer 32-bit | Number of sectors in the SMART or ATA general logs, where the latter is returned by the ATA READ_LOG_EXT command |
| bp | Integer 32-bit | Bytes per sector |
| ph | Boolean | Is physical |

> Note that EnCase 7 has been observed to generated strange values for the device serial number.

#### Drive type {#drive_type}

| Value | Identifier | Description |
| --- | --- | --- |
| a | | RAM disk |
| c | | Optical disc (CD-ROM) |
| f | | Fixed |
| l | | Single files (Logical evidence) |
| m | | Memory |
| p | | PALM |
| r | | Removable |

### Case data

The case data section can be found:

* in every segment file after the device information section in EWF2-Ex01
* in every segment file after the file header in EWF2-Lx01

When encryption is enabled the case data is encrypted.

The case data section contains a serialized object string that consist of:

| Line | Value | Description |
| --- | --- | --- |
| 1 | 1 | Number of objects |
| 2 | "main" | Object name |
| 3 | | Attribute tags |
| 4 | | Attribute values |
| 5 | | Empty line |

#### Attribute tags

| Identifier | Type | Description |
| --- | --- | --- |
| nm | Text | Name, which is similar to Description in EWF1 |
| cn | Text | Case number |
| en | Text | Evidence number |
| ex | Text | Examiner name |
| nt | Text | Notes |
| av | Text | Application version, which contains the version of the application used for acquisition |
| os | Text | Operating system, which contains the operating system used used for acquisition |
| tt | Timestamp | Target time, which contains the date and time of the system used for acquisition in UTC and is similar to Acquired date in EWF1 |
| at | Timestamp | Actual time, which contains an user provided date and time and is similar to System date in EWF1 |
| tb | Integer 64-bit | Number of chunks (blocks) |
| cp | Integer 32-bit | [Compression method](#compression_methods), which is empty (not 0) when the compression method is no compression |
| sb | Integer 32-bit | Number of sectors per chunk (block) |
| gr | Integer 32-bit | Error granularity |
| wb | Integer 32-bit | Write-blocker type |

> Note that EnCase 7 only provides the following number of sectors per chunk: 64, 128, 256, 512,
> 1024 which is referred by the application as block size. The thorough error granularity in
> EnCase 7 corresponds to 1 sector.

TODO: "EnCase Evidence File Format Version 2" defines the "Actual time" as in UTC, but if this is
user provided can UTC still be guaranteed?

#### Write-blocker type

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | | FastBloc |
| 2 | | Tableau |

### Sector data

The first sector data section can be found in every segment file after the case data section.
Successive sector data sections are found after the sector table section.

When encryption is enabled the sector data is encrypted.

The sector data is stored in chunks. "EnCase Evidence File Format Version 2" states that each chunk
must be stored 16-byte aligned and padded with 0-byte values if necessary. Although it can read non
16-byte aligned chunks.

If the sector compression method defined in case data section is set the chunk is compressed and
the chunk data flag COMPRESSED is set. The checksum intrinsic to the compression method is used to
verify the integrity of the chunk data. The chunk data flag CHECKSUMED is not set.

If a chunk is not compressed an Adler32 checksum of the data is stored after the chunk data and the
chunk data flag CHECKSUMED is set.

Pattern fill seems to be a special case of compression and the COMPRESSED flag is set in
combination with the PATTERNFILL flag. In EnCase pattern fill is not used when writing files and
the compression is set to none. Keramics, when reading files, ignores the PATTERNFILL flag if the
corresponding COMPRESSED flag is not set.

If the PATTERNFILL flag is set the chunk data size in the sector table entry is set to 0 and the
chunk data offset contains a 64-bit pattern to fill the chunk data.

### Sector table

The sector table is stored as an array of sector table entries (chunk descriptor or block offset).
It defines the location of the chunk data in the segment file.

The sector table section can be found in every segment file after the sector data section. Every
sector data section should be followed by a section table section.

When encryption is enabled the sector table is encrypted.

The sector table consists of:

* the sector table header
* an array of sector table entries
* the sector table footer

#### Sector table header

The sector table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | First chunk number, where 0 is the first chunk number for the entire image |
| 8 | 4 | | Number of entries |
| 12 | 4 | 0 | Unknown (Padding) |
| 16 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the sector table header |

The sector table header should be followed by 12 bytes of alignment padding.

TODO: determine if EnCase support non-contiguous images.

TODO: determine if EnCase writes about 1600 entries per section.

#### Sector table entry

A sector table entry is 16 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Chunk data offset or fill pattern if corresponding flag is set |
| 8 | 4 | | Chunk data size |
| 12 | 4 | | Chunk data flags |

#### Chunk data flags

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | COMPRESSED | The chunk is compressed |
| 0x00000002 | CHECKSUMED | The chunk is followed by an Adler32 checksum |
| 0x00000004 | PATTERNFILL | The chunk is sparse and the value in the chunk data offset is used to fill the chunk data at run-time |

The PATTERNFILL flag should be ignored if the COMPRESSED flag is not set.

#### Sector table footer

The sector table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which is an Adler-32 of all the previous data within the sector table entries |

The sector table footer should be followed by 12 bytes of alignment padding.

### Error table

The error table is stored as an array of error table entries. It defines the sector ranges that
could not be read correctly during acquisition.

The error table section is optional, it does not need to be present. If it does it resides in the
last segment file before the MD5 hash section.

When encryption is enabled the error table is encrypted.

The error table consists of:

* the error table header
* an array of error table entries
* the error table footer

#### Error table header

The error table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of entries |
| 4 | 12 | | Unknown (Empty value) |
| 16 | 4 | | Checksum, which is an Adler-32 of all the previous data within the error table header |

The error table header should be followed by 12 bytes of alignment padding.

> Note that this differs from what is documented in "EnCase Evidence File Format Version 2".

#### Error table entry

An error table entry is 16 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Start sector |
| 8 | 4 | | Number of sectors |
| 12 | 4 | 0 | Unknown (Padding) |

#### Error table footer

The error table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the array of error table entries |

The error table footer should be followed by 12 bytes of alignment padding.

### Session table

The session table is stored as an array of session table entries. It defines the sessions of the
optical disc stored in the set of segment files.

The session table section is optional, it does not need to be present. If it does it resides in the
last segment file before the error table section.

When encryption is enabled the session table is encrypted.

The session table consists of:

* the session table header
* an array of session table entries
* the session table footer

#### Session table header

The session table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of entries |
| 4 | 12 | | Unknown (Empty value) |
| 16 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the session table header |

The session table header should be followed by 12 bytes of alignment padding.

> Note that this differs from what is documented in "EnCase Evidence File Format Version 2".

#### Session table entry

A session table entry is 32 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | First sector |
| 8 | 4 | | Session flags |
| 12 | 5 x 4 = 20 | 0 | Unknown (Padding) |

> Note that for an optical disc, observed for a CD, the first session sector is stored as 16,
> although the actual session starts at sector 0. Could this value be overloaded to indicate
> the size of the reserved space between the start of the session and the ISO 9660 volume
> descriptor.

#### Session flags

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | | If set the track is an audio track otherwise the track is a data track |

EnCase stores the data of audio tracks of an optical disc as 0-byte data with a sector size
of 2048. It is therefore assumed that the format is only to support data tracks with a sector size
of 2048.

#### Session table footer

The session table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which is an Adler-32 of all the previous data within the array of session table entries |

The session table footer should be followed by 12 bytes of alignment padding.

### Increment data

TODO: complete this section, need example data.

### MD5 hash

The MD5 hash section contains the MD5 hash of the data stored in the set of segment files.

The MD5 hash section is optional, it does not need to be present. If it does it resides in the last
segment file before the SHA1 hash section.

When encryption is enabled the MD5 hash is encrypted.

The MD5 hash data is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | MD5 hash |
| 16 | 4 | | Checksum, which is an Adler-32 of the MD5 hash |

The MD5 hash data should be followed by 12 bytes of alignment padding.

### SHA1 hash

The SHA1 hash section contains the SHA1 hash of the data stored in the set of segment files.

The SHA1 hash section is optional, it does not need to be present. If it does it resides in the
last segment file before the analytical data section.

When encryption is enabled the SHA1 hash is encrypted.

The SHA1 hash data is 24 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 20 | | SHA1 hash |
| 20 | 4 | | Checksum, which is an Adler-32 of the SHA1 hash |

The MD5 hash data should be followed by 8 bytes of alignment padding.

### Restart data

The restart data section is optional, it does not need to be present. If it does it resides in the
last segment file before the done section.

TODO: determine if the restart data stored after or before the encryption keys

> Note that the "main" and "rl" object tags are not explicitly defined in the string.

The restart data section contains a serialized object string that consist of:

| Line | Value | Description |
| --- | --- | --- |
| 1 | | Object tags |
| 2 | | Attribute tags |
| 3 | | Segments of the restart object |

The segments of the restart object likely represent the "tree view" in the evidence view within
EnCase. In the example below there are 3 segments, the first segment having a sub object that has
"expanded" properties and containing another sub object that contains the actual restart data.

```text
1	1
p	d	sr	sp
0	1

0	1
5
0	0
			1216
```

#### Object tags

| Column | Value | Description |
| --- | --- | --- |
| 1 | 1 | Number of child objects, where the restart data should contain a single restart object |
| 2 | 1 | Unknown (Constant value) |

#### Attribute tags

| Value | Identifier | Description |
| --- | --- | --- |
| p | Integer 32-bit | [Properties](#attribute_tag_proprerties), which contains flags, see next paragraph, defaults to 0 if not set |
| d | Timestamp | Start date and time, which contains the date and time the acquisition process was (re-)started |
| sr | Integer 64-bit | First sector acquired in the acquisition process |
| sp | Integer 64-bit | Last sector acquired in the acquisition process |

#### Properties {#attribute_tag_proprerties}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | STATEFOLDER | Item is a folder/container |
| 0x02 | STATESELECTED | Item is selected (highlighted in blue) |
| 0x04 | STATEEXPANDED | Item is expanded |
| 0x08 | STATEINCLUDE | Item is included (green-plated) |

> Note hat according to Guidance Software this value is used to store saved stated. In this context
> the value should always set to 0 but can contain other values in different contexts. EnCase can
> choose to ignore these values.

### Encryption keys

In EWF2 the data and some of the metadata can be encrypted, the encrypted keys section contains
information necessary for decrypting the data.

The encryption keys section is optional, it does not need to be present. If it does it resides in
the last segment file before the done section.

TODO: determine if the encryption keys are stored after or before the restart data.

The encryption keys is variable of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Size, which includes the size of the padding  |
| 4 | 4 | | Unknown (Checksum?) |
| 8 | 8 | 2 | Unknown (Algorithm ID?), where 2 represents AES-256 |
| 16 | ... | | Unknown (Encrypted data?) |

The encryption keys should be followed by 12 bytes of alignment padding.

"EnCase Evidence File Format Version 2" refers to a separate "document outlining the encryption
support for Ex01 for further detail". This document was never made public and after inquiring with
Guidance Software, they indicated that they were not disclosing information about Ex01 encryption.

### Memory extents table

The memory extents table is stored as an array of memory extents table entries. It defines the
extents of memory stored in the set of segment files.

TODO: determine the location in segment files, and if this section is affected by encryption.

TODO: determine if this table also comes with a table header and footer.

#### Memory extents table entry

A memory extents table entry is 16 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Start page |
| 8 | 8 | | Number of pages |

### Next

The next section is without data and marks the end of the segment file indicating more segment
files are in the set. It should be the last section in a segment file, other than the last segment
file.

### Final information

> Note that "EnCase Evidence File Format Version 2" defines this section as currently unused.

### Done

The done section is without data and marks the end of the segment file indicating this is the last
segment file in the set. It should be the last section in the last segment file.

### Analytical data

The analytical data section is optional, it does not need to be present. If it does it resides in
the last segment file before the restart data section.

When encryption is enabled the analytical data is encrypted.

The analytical data section contains a serialized object string that consist of:

| Line | Value | Description |
| --- | --- | --- |
| 1 | 1 | Number of objects |
| 2 | "main" | Object name |
| 3 | | Attribute tags |
| 4 | | Attribute values |
| 5 | | Empty line |

> Note that "EnCase Evidence File Format Version 2" does not define the format of this section in
> detail.

#### Attribute tags

| Identifier | Type | Description |
| --- | --- | --- |
| tps | Integer 64-bit | The (total) number of bytes not written for use of pattern fill |

### Single files data

The single files data section is only present in EWF2-Lx01.

The single files data section can be found in the last segment file after the last sector table
section.

TODO: determine how the single files data section behaves in non-closed LEF files.

This section has the section integrity hash set.

The single files data section contains a non-compressed serialized object data which is similar to
the EnCase 7 [ltree data](ewf.md#ltree_data) in EWF-L01.

### 0x00000021 table

The 0x00000021 table consists of:

* the 0x00000021 table header
* an array of 0x00000021 table entries
* the 0x00000021 table footer

#### 0x00000021 table header

The 0x00000021 table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of entries |
| 4 | 12 | | Unknown (Empty value) |
| 16 | 4 | | Checksum, which is an Adler-32 of all the previous data within the 0x00000021 table header |

The 0x00000021 table header should be followed by 12 bytes of alignment padding.

#### 0x00000021 table entry

An 0x00000021 table entry is 8 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (Start offset in the data?) |

#### 0x00000021 table footer

The 0x00000021 table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which is an Adler-32 of all the previous data within the array of 0x00000021 table entries |

The 0x00000021 table footer should be followed by 12 bytes of alignment padding.

### Single files MD5 hash table

The single files MD5 hash table consists of:

* the single files MD5 hash table header
* an array of single files MD5 hash table entries
* the single files MD5 hash table footer

#### single files MD5 hash table header

The 0x00000021 table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of entries |
| 4 | 12 | | Unknown (Empty value) |
| 16 | 4 | | Checksum, which is an Adler-32 of all the previous data within the single files MD5 hash table header |

The single files MD5 hash table header should be followed by 12 bytes of alignment padding.

#### single files MD5 hash table entry

A single files MD5 hash table entry is 8 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | MD5 hash |

#### single files MD5 hash table footer

The single files MD5 hash table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which is an Adler-32 of all the previous data within the array of single files MD5 hash table entries |

The single files MD5 hash table footer should be followed by 12 bytes of alignment padding.

### 0x00000023 table

The 0x00000023 table consists of:

* the 0x00000023 table header
* an array of 0x00000023 table entries
* the 0x00000023 table footer

#### 0x00000023 table header

The 0x00000023 table header is 20 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Number of entries |
| 4 | 12 | | Unknown (Empty value) |
| 16 | 4 | | Checksum, which is an Adler-32 of all the previous data within the 0x00000023 table header |

The 0x00000023 table header should be followed by 12 bytes of alignment padding.

#### 0x00000023 table entry

An 0x00000023 table entry is 8 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (Start offset in the data?) |

#### 0x00000023 table footer

The 0x00000023 table footer is 4 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which is an Adler-32 of all the previous data within the array of 0x00000023 table entries |

The 0x00000023 table footer should be followed by 12 bytes of alignment padding.

> Note that if the number of table entries is odd the alignment padding is only 4 bytes.

## Serialized (file) object data

The serialized object data is stored as a compressed UTF-16 string with byte-order-mark. Commonly
the string is encoded in little-endian. The compression method is defined in the file header of the
segment file.

The serialized object data consists of:

* the first line containing the number of objects in the string
* the object data

The object serialization format uses the following special character values:

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | | Escaped line feed |
| 0x0002 | | Escaped carriage return |
| 0x0003 | | Escaped tab |
| | | |
| 0x0009 | | Value delimiter |
| 0x000a | | Line delimiter |

> Note that "EnCase Evidence File Format Version 2" states line feed (0x000d) as line delimiter
> this should be line feed (0x000a).

### Object

An object consists of multiple lines:

| Line | Value | Description |
| --- | --- | --- |
| 1 | | Object name |
| 2 | | Attribute tags |

### Data types

| Identifier | Type | Description |
| --- | --- | --- |
| | Boolean | Boolean defined as: false => (empty) or true => a single character containing "1" |
| | Enumeration | Single character that represent a value in an enumeration |
| | Array of Integer 64-bit | A space separated list of 64-bit unsigned integers |
| | Integer 32-bit | Decimal representation of a 32-bit unsigned integer |
| | Integer 64-bit | Decimal representation of a 64-bit unsigned integer |
| | Object | Sub (or child) object |
| | Text | Text, where EnCase appearst to limit the string to 3000 characters |
| | Timestamp | Decimal representation of a 32-bit unsigned integer containing the number of seconds since Jan 1, 1970 00:00:00 UTC |

### Sub objects

Sub object are represented using the following value pairs.

| Column | Value | Description |
| --- | --- | --- |
| 1 | | Object type (Save Code), which according to Guidance Software this value should be 0 (NodeClass) for most use cases |
| 2 | | Number of child objects |

So if there are 3 objects, all 3 have the attribute tags x, y and z:

* A: which has 2 sub objects B and C
* B: which has no children
* C: which has no children

This is serialized as:

```text
x	y	z
0       2
A	A	A
0       0
B	B	B
0       0
C	C	C
```

For sake of the example the attribute values have been marked with the identifier of the object.

## Encryption

TODO: complete this section.
