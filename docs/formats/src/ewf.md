# Expert Witness Compression Format (EWF)

EWF is short for Expert Witness Compression Format. It is a file type used to
store storage media images for digital forensic purposes. It is currently
widely used in the field of computer forensics in proprietary tooling like
EnCase en FTK. The [original specification](http://www.asrdata.com/whitepaper-html)
of the format was provided by ASR Data for the SMART application

The EWF format was succeeded by the [Expert Witness Compression Format version 2](ewf2.md)
in EnCase 7 (EWF2-Ex01 and EWF2-Lx01). EnCase 7 also uses a different version
of EWF-L01 then its predecessors.

## Overview

The Expert Witness Compression Format (EWF) is used to store:

* storage media images, such as hard disks, USB sticks, optical disks
* individual volumes or partitions
* "physical" RAM and process memory 

EWF can store data compressed or uncompressed, in a single image in one or more
segment files. Each segment file consist of a standard header, followed by
multiple sections. A single section cannot span multiple files. Sections are
arranged back-to-back.

### Terminology

In this document when referred to the EWF format it refers to the original
specification by ASR Data. The newer formats like that of EnCase are deducted
from the original specification and will be referred to as the EWF-E01, because
of the default file extension. Whereas the Logical File Evidence (LVF) format
introduced in EnCase 5, which is also stored in the EWF format will be referred
to as EWF-L01. The SMART format is viewed separately to allow for discussion if
the implementation differs from the specification by ASR Data and will be
referred to as the EWF-S01, because of the default file extension.

All offsets are relative to the beginning of an individual section, unless
otherwise noted. EnCase allows a maximum size of a segment file to be 2000 MiB.
This has to do with the size of the offset of the chunk of media data. This is
a 32 bit value where the most significant bit (MSB) is used as a compression
flag. Therefore the maximum offset size (31 bit) can address about 2048 MiB. In
EnCase 6.7 an addition was made to the table value to provide for a base offset
to allow for segment files greater than 2048 MiB.

A chunk is defined as the sector size (per default 512 bytes) multiplied by the
block size, the number of sectors per chunk (block) (per default 64 sectors).
The data within the EWF format is stored in little-endian. The terms block and
chunk are used intermittently.

## Segment file

EWF stores data in one or more segment files (or segments). Each segment file
consists of:

* A file header.
* One or more sections.

### File header

Each segment file starts with a file header.

EWF defines that the file header consists of 2 parts, namely:

* a signature part
* fields part

#### EWF, EWF-E01 and SMART (EWF-S01)

The file header, used by both the EWF-E01 and SMART (EWF-S01) formats, is 13
bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "EVF\x09\x0d\x0a\xff\x00" | Signature
| 8 | 1 | 0x01 | Start of fields
| 9 | 2 | | Segment number, which must be 1 or higher
| 11 | 2 | 0x0000 | End of fields

The segment number contains a number which refers to the number of the segment
file, starting with 1 for the first file.

> Note this means there could only be a maximum of 65535 (0xffff) files, if it
> is an unsigned value.

#### EWF-L01

The file header, used by the EWF-L01 format, is 13 bytes in size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | "LVF\x09\x0d\x0a\xff\x00" | Signature
| 8 | 1 | 0x01 | Start of fields
| 9 | 2 | | Segment number, which must be 1 or higher
| 11 | 2 | 0x0000 | End of fields

The segment number contains a number which refers to the number of the segment
file, starting with 1 for the first file.

> Note this means there could only be a maximum of 65535 (0xffff) files, if it
> is an unsigned value.

### Segment file extensions

The SMART (EWF-S01) and the EWF-E01 formats use a different naming convention
for the segment files.

#### SMART (EWF-S01)

The SMART (EWF-S01) extension naming has two distinct parts.

* The first segment file has the extension '.s01'.
  * The next segment file has the extension '.s02.
  * This will continue up to '.s99'.
* After which the next segment file has the extension '.saa'.
  * The next segment file has the extension '.sab'.
  * This will continue up to '.saz'.
  * The next segment file has the extension '.sba'.
  * This will continue up to '.szz'.
  * The next segment file has the extension '.faa'.
  * This will continue up to '.zzz'.
  * Not confirmed but other sources report it will even continue to the use the extensions '.{aa'.

Keramics supports extensions up to .zzz

#### EWF-E01

The EWF-E01 extension naming has two distinct parts.

* The first segment file has the extension '.E01'.
  * The next segment file has the extension '.E02.
  * This will continue up to '.E99'.
* After which the next segment file has the extension '.EAA'.
  * The next segment file has the extension '.EAB'.
  * This will continue up to '.EAZ'.
  * The next segment file has the extension '.EBA'.
  * This will continue up to '.EZZ'.
  * The next segment file has the extension '.FAA'.
  * This will continue up to '.ZZZ'.
  * Not confirmed but other sources report it will even continue to the use the extensions '.[AA'.

Keramics supports extensions up to .ZZZ

#### EWF-L01

The EWF-L01 extension naming has two distinct parts.

* The first segment file has the extension '.L01'.
  * The next segment file has the extension '.L02.
  * This will continue up to '.L99'.
* After which the next segment file has the extension '.LAA'.
  * The next segment file has the extension '.LAB'.
  * This will continue up to '.LAZ'.
  * The next segment file has the extension '.LBA'.
  * This will continue up to '.LZZ'.
  * The next segment file has the extension '.MAA'.
  * This will continue up to '.ZZZ'.
  * Not confirmed but other sources report it will even continue to the use the extensions '.[AA'.

Keramics supports extensions up to .ZZZ

### Segment file set identifier GUID

Segment file sets do not have a strict unique identifier. However the
[volume section](#volume_section) contains a GUID that can be used for this
purpose. Where:

* linen 5 to 6 use a time and MAC address based version (1) of the GUID
* EnCase 5 to 7 and linen 6 to 7 use a random based version (4) of the GUID

> Note that in linen 6 the switch from a version 1 to 4 GUID was somewhere made 
> between version 6.01 and 6.19.

See RFC4122 for more information about the different GUID versions.

## The sections

The remainder of the segment file consists of sections. Every section starts
with the same data this will be referred to as the section header. The section
header could also be referred as the section header, but this allows for
unnecessary confusion with the [header section](#header_section).

### Section header

The section header consist of 76 bytes, it contains information about a
specific section.

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Section type, a string containing the section type definition, such as "header" or "volume"
| 16 | 8 | | Next section offset, where the offset is relative from the start of the segment file
| 24 | 8 | | Section size
| 32 | 40 | 0x00 | Unknown (Padding)
| 72 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the section header.

Some sections contain additional data, refer to paragraph section types for
more information.

> Note Expert Witness 1.35 (for Windows) does not set the section size.

> Note that in EnCase 2 DOS version the padding itself does not contains 0-byte
> values but data, probably the memory is not filled with 0-byte values.

### Section types

There are multiple section types. [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
defines the following:

* Header section
* Volume section
* Table section
* Next and Done section

The following sections type were found analyzing more recent EnCase files (EWF-E01):

* Header2 section
* Disk section
* Sectors section
* Table2 section
* Data section
* Error2 section
* Session section
* Hash section
* Digest section

The following sections type were found analyzing more recent EnCase files (EWF-L01):

* Ltree section
* Ltypes section

### Header2 section

The header2 section is identified in the section data type field as "header2".
Some aspects of this section are:

* Found in EWF-E01 in EnCase 4 to 7, and EWF-L01 in EnCase 5 to 7
* Found at the start of the first segment file. Not found in subsequent segment files.
* The same header2 section is found twice directly after one and other.

The additional data this section contains is the following:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 76 (0x4c) | (variable) | Information about the acquired media.

The information about the acquired media consists of [zlib compressed data](zlib.md).
It contains text in UTF16 format specifying information about the acquired
media. The text multiple lines separated by an end of line character(s).

The first 2 bytes of the UTF16 string are the byte order mark (BOM):

* 0xff 0xfe for UTF-16 litte-endian
* 0xfe 0xff for UTF-16 big-endian

In the next paragraphs the various variants of the header2 section are described.

#### EnCase 4 (EWF-E01)

In EnCase 4 (EWF-E01) the header2 information consist of 5 lines, and contains
the equivalent information as the [header section](#header_section).

| Line number | Value | Description
| --- | --- | ---
| 1 | 1 | The number of categories provided
| 2 | main | The name/type of the category provided
| 3 | | Identifiers for the values in the 4th line
| 4 | | The data for the different identifiers in the 3rd line
| 5 | | (an empty line)

The end of line character(s) is a newline (0x0a).

> Note this end of line character differs from the one used in the [header section](#header_section).

The 3rd and the 4th line consist of the following tab (0x09) separated values.

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | a | Unique description
| 2 | c | Case number
| 3 | n | Evidence number
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | av | Version, which contains the EnCase version used to acquire the media
| 7 | ov | Platform, which contains the platform/operating system used to acquire the media
| 8 | m | Acquisition date and time
| 9 | u | System date and time
| 10 | p | Password hash

Also see [header2 values](#header2_values)

> Note the hashing algorithm is the same as for the [header section](#header_section).

#### EnCase 5 to 7 (EWF-E01)

In EnCase 5 to 7 (EWF-E01) the header2 information consist of 17 lines, and
contains:

| Line number | Value | Description
| --- | --- | ---
| 1 | 3 | The number of categories provided
| 2 | main | The name/type of the category provided
| 3 | | Identifier for the values in the category
| 4 | | The data for the different identifiers in the category
| 5 | | (an empty line)
| 6 | srce | The name/type of the category provided, also see [sources category](#sources_category1)
| 7 | |
| 8 | | Identifier for the values in the category
| 9 | | The data for the different identifiers in the category
| 10 | |
| 11 | | (an empty line)
| 12 | sub | The name/type of the category provided, also see [subjects category](#subjects_category1)
| 13 | |
| 14 | | Identifier for the values in the category
| 15 | | The data for the different identifiers in the category
| 16 | |
| 17 | | (an empty line)

The end of line character(s) is a newline (0x0a).

##### Main category

The 3rd and the 4th line consist of the following tab (0x09) separated values.

> Note the actual values in this category are dependent on the version of
> EnCase.

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | a | Unique description
| 2 | c | Case number
| 3 | n | Evidence number
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | md | The model of the media, such as hard disk model (introduced in EnCase 6)
| 7 | sn | The serial number of media (introduced in EnCase 6)
| 8 | l | The device label (introduced in EnCase 6.19)
| 9 | av | Version, which contains the EnCase version used to acquire the media. EnCase limits this value to 12 characters
| 10 | ov | Platform, which contains the platform/operating system used to acquire the media
| 11 | m | Acquisition date and time
| 12 | u | System date and time
| 13 | p | Password hash
| 14 | pid | Process identifier, which contains the identifier of the process memory acquired (introduced in EnCase 6.12/Winen 6.11)
| 15 | dc | Unknown
| 16 | ext | Extents, which contains the extents of the process memory acquired (introduced in EnCase 6.12/Winen 6.11)

Also see [header2 values](#header2_values)

> Note that both the acquiry and system date and time are empty in a file
> created by winen.

> Note that rhe date values in the [header section](#header_section) (not the
> header2 section) are set to: Thu Jan  1 00:00:00 1970. Where the time is
> dependent on the time zone and daylight savings.

##### <a name="sources_category1"></a>Sources category

Line 6 the srce category contains information about acquisition sources.

TODO: describe what a source is in the context of EnCase.

Line 7 consists of 2 values, namely the values are "0 1".

The 8th line consist of the following tab (0x09) separated values. Note that
the actual values in this category are dependent on the version of EnCase.

| Identifier number | Character in 8rd line | Meaning
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the source
| 4 | ev | Evidence number, which contains a string
| 5 | tb | Total bytes, which contains an integer
| 6 | lo | Logical offset, which contains an integer which is -1 when value is not set
| 7 | po | Physical offset, which contains an integer which is -1 when value is not set
| 8 | ah | MD5 hash, which contains a string with the MD5 hash of the source
| 9 | sh | SHA1 hash, contains a string with the SHA1 hash of the source (introduced in EnCase 6.19)
| 10 | gu | Device GUID, which contains a string with a GUID or "0" if not set
| 11 | pgu | Primary device GUID, which contains a string with a GUID or "0" if not set (introduced in EnCase 7)
| 12 | aq | Acquisition date and time, which contains an integer with a POSIX timestamp

Line 9 consists of 2 values, namely the values are "0 0".

Line 10 contains the values defined by line 8.

> Note the default values of some of these values has changed around EnCase
> 6.12.

If the "ha" value contains "00000000000000000000000000000000" this means the
MD5 hash is not set. The same applies for the "sha" value when it contains
"0000000000000000000000000000000000000000" the SHA1 has is not set.

##### <a name="subjects_category1"></a>Subjects category

Line 12 the sub category contains information about subjects.

TODO: describe what a subject is in the context of EnCase.

Line 13 consists of 2 values, namely the values are "0 1".

The 14th line consist of the following tab (0x09) separated values.

| Identifier number | Character in 14rd line | Meaning
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the subject
| 4 | nu | Unknown (Number)
| 5 | co | Unknown (Comment)
| 6 | gu | Unknown (GUID)

Line 15 consists of 2 values, namely the values are "0 0".

Line 16 contains the values defined by line 14. Note that the default values of
some of these values has changed around EnCase 6.12.

#### EnCase 5 to 7 (EWF-L01)

The EnCase 5 to 7 (EWF-E01) header2 section specification also applies to the
EnCase 5 to 7 (EWF-L01) format. However:

* both the acquired and system date and time are not set

#### <a name="header2_values"></a>Header2 values

| Identifier | Description | Notes
| --- | --- | ---
| a | Unique description | Free form string. Note that EnCase might not respond when this value is large e.g. >= 1 MiB
| av | Version | Free form string. EnCase limits this string to 12 - 1 characters
| c | Case number | Free form string. EnCase limits this string to 3000 - 1 characters
| dc | Unknown |
| e | Examiner name | Free form string. EnCase limits this string to 3000 - 1 characters
| ext | Extents | Extents header value
| l | Device label | Free form string
| m | Acquisition date and time | String containing POSIX 32-bit epoch timestamp, e.g. "1142163845" which represents the date: March 12 2006, 11:44:05
| md | Model | Free form string. EnCase limits this string to 3000 - 1 characters
| n | Evidence number | Free form string. EnCase limits this string to 3000 - 1 characters
| ov | Platform | Free form string. EnCase limits this string to 24 - 1 characters
| pid | Process identifier | String containing the process identifier (pid) number
| p | Password hash | String containing the password hash. If no password is set it should be simply the character '0'.
| sn | Serial Number | Free form string. EnCase limits this string to 3000 - 1 characters
| t | Notes | Free form string. EnCase limits this string to 3000 - 1 characters
| u | System date and time | String containing POSIX 32-bit epoch timestamp, e.g. "1142163845" which represents the date: March 12 2006, 11:44:05

> Note the restrictions were tested with EnCase 7.02.01, older versions could
> have a restriction of 40 characters instead of 3000 characters.

##### Extents header value

An extents header value consist of:

```
number of entries
entries that consist of: S <1> <2> <3>
```

#### <a name="header_section"></a>Header section

The header section is identified in the section data type field as "header".
Some aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
* Found in EWF-E01 in EnCase 1 to 7 or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7, and SMART (EWF-S01)
* Found at the start of the first segment file or in EnCase 4 to 7 after the header2 section in the first segment file. Not found in subsequent segment files.

The additional data this section contains is the following:

| Offset | Number of bytes | Description
| --- | --- | ---
| 76 (0x4c) | (variable) | Information about the acquired media.

The information about the acquired media consists of [zlib compressed data](zlib.md).
It contains text in ASCII format specifying information about the acquired
media. The text multiple lines separated by an end of line character(s).

In the next paragraphs the various variants of the header section are
described. In all cases the information consists of at least 4 lines:

| Line number | Value | Description
| --- | --- | ---
| 1 | 1 | The number of categories provided
| 2 | main | The name/type of the category provided
| 3 | | Identifiers for the values in the 4th line
| 4 | | The data for the different identifiers in the 3rd line

An additional 5th line is found in FTK Imager, EnCase 1 to 7 (EWF-E01).

| Line number | Value | Description
| --- | --- | ---
| 5 | | (an empty line)

#### EWF format

Some aspects of this section are:

* [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html) specifies the end of line character(s) is a newline (0x0a).

According to [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
the 3rd and the 4th line consist of the following tab (0x09) separated values:

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | c | Case number
| 2 | n | Evidence number
| 3 | a | Unique description
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | m | Acquisition date and time
| 7 | u | System date and time
| 8 | p | Password hash
| 9 | r | Compression level

Also see [header values](#header_values)

[ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html) states that the Expert Witness Compression uses 'f', fastest compression.

#### EnCase 1 (EWF-E01)

Some aspects of this section are:

* The header section is defined only once.
* It is the first section of the first segment file. It is not found in subsequent segment files.
* The header data itself is compressed using zlib.
* The end of line character(s) is a carriage return (0x0d) followed by a newline (0x0a).

The 3rd and the 4th line consist of the following tab (0x09) separated values"

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | c | Case number
| 2 | n | Evidence number
| 3 | a | Unique description
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | m | Acquisition date and time
| 7 | u | System date and time
| 8 | p | Password hash
| 9 | r | Compression level

Also see [header values](#header_values)

#### SMART (EWF-S01)

Some aspects of this section are:

* The header section is defined once.
* It is the first section of the first segment file. It is not found in subsequent segment files.
* The header data is always processed by zlib, however the same compression level is used as for the chunks. This could mean compression level 0 which is no compression.

> The SMART format uses the FTK Imager (EWF-E01) specification for this section.
> Note that this could be something FTK Imager specific.

#### EnCase 2 and 3 (EWF-E01)

Some aspects of this section are:

* The same header section defined twice.
* It is the first and second section of the first segment file. It is not found in subsequent segment files.
* The header data itself is compressed using zlib.
* The end of line character(s) is a carriage return (0x0d) followed by a newline (0x0a).

The 3rd and the 4th line consist of the following tab (0x09) separated values:

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | c | Case number
| 2 | n | Evidence number
| 3 | a | Unique description
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | av | Version, which contains the EnCase version used to acquire the media
| 7 | ov | Platform, which contains the platform/operating system used to acquire the media
| 8 | m | Acquisition date and time
| 9 | u | System date and time
| 10 | p | Password hash
| 11 | r | Compression level

Also see [header values](#header_values)

#### EnCase 4 to 7 (EWF-E01)

Some aspects of this section are:

* The header is defined only once.
* It resides after the header2 sections of the first segment file. It is not found in subsequent segment files.
* The header data itself is compressed using zlib.
* The end of line character(s) is a carriage return (0x0d) followed by a newline (0x0a).

The 3rd and the 4th line consist of the following tab (0x09) separated values:

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | c | Case number
| 2 | n | Evidence number
| 3 | a | Unique description
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | av | Version, which contains the EnCase version used to acquire the media
| 7 | ov | Platform, which contains the platform/operating system used to acquire the media
| 8 | m | Acquisition date and time
| 9 | u | System date and time
| 10 | p | Password hash

Also see [header values](#header_values)

#### linen 5 to 7 (EWF-E01)

Some aspects of this section are:

* The same header section defined twice.
* It is the first and second section of the first segment file. It is not found in subsequent segment files.
* The header data itself is compressed using zlib.
* The end of line character(s) is a newline (0x0a).

The header information consist of 18 lines

The remainder of the string contains the following information:

| Line number | Value | Description
| --- | --- | ---
| 1 | 3 | The number of categories provided
| 2 | main | The name/type of the category provided
| 3 | | Identifier for the values in the 4th line
| 4 | | The data for the different identifiers in the 3rd line
| 5 | | (an empty line)
| 6 | srce | The name/type of the section provided, also see [Sources category](#sources_category2)
| 7 | |
| 8 | | Identifier for the values in the section
| 9 | |
| 10 | |
| 11 | | (an empty line)
| 12 | sub | The name/type of the section provided, also see [Subjects category](#subjects_category2)
| 13 | |
| 14 | | Identifier for the values in the section
| 15 | |
| 16 | |
| 17 | | (an empty line)

The end of line character(s) is a newline (0x0a).

##### Main category - linen 5

The 3rd and the 4th line consist of the following tab (0x09) separated values.

> Note the actual values in this category are dependent on the version of linen.

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | a | Unique description
| 2 | c | Case number
| 3 | n | Evidence number
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | av | Version, which contains the linen version used to acquire the media
| 7 | ov | Platform, which contains the platform/operating system used to acquire the media
| 8 | m | Acquisition date and time
| 9 | u | System date and time
| 10 | p | Password hash

Also see [header values](#header_values)

##### Main category - linen 6 to 7

The 3rd and the 4th line consist of the following tab (0x09) separated values.

> Note the actual values in this category are dependent on the version of linen.

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | a | Unique description
| 2 | c | Case number
| 3 | n | Evidence number
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | md | The model of the media, such as hard disk model (Introduced in linen 6)
| 7 | sn | The serial number of media (Introduced in linen 6)
| 8 | l | The device label (Introduced in linen 6.19)
| 9 | av | Version, which contains the linen version used to acquire the media
| 10 | ov | Platform, which contains the platform/operating system used to acquire the media
| 11 | m | Acquisition date and time
| 12 | u | System date and time
| 13 | p | Password hash
| 14 | pid | Process identifier, which contains the identifier of the process memory acquired (Introduced in linen 6.19 or earlier)
| 15 | dc | Unknown (Introduced in linen 6)
| 16 | ext | Extents, which contains the extents of the process memory acquired (Introduced in linen 6.19 or earlier)

> Note as of linen 6.19 the acquire date and time is in UTC and the system date
> and time is in local time. Where as before both values were in local time.

Also see [header values](#header_values)

##### <a name="sources_category2"></a>Sources category

Line 6 the srce category contains information about acquisition sources

TODO: describe what a source is in the context of EnCase.

Line 7 consists of 2 values, namely the values are "0 1".

The 8th line consist of the following tab (0x09) separated values.

| Identifier number | Character in 8rd line | Meaning
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the source
| 4 | ev | Evidence number, which contains a string
| 5 | tb | Total bytes, which contains an integer
| 6 | lo | Logical offset, which contains an integer which is -1 when value is not set
| 7 | po | Physical offset, which contains an integer which is -1 when value is not set
| 8 | ah | Unknown (MD5?), which contains a string
| 9 | sh | Unknown (SHA1?), which contains a string (Introduced in linen 6.19 or earlier)
| 10 | gu | Device GUID, which contains a string with a GUID or "0" if not set
| 11 | aq | Acquisition date and time, which contains an integer with a POSIX timestamp

Line 9 consists of 2 values, namely the values are "0 0".

Line 10 contains the values defined by line 8.

> Note the default values of some of these values has changed around linen 6.19
> or earlier.

##### <a name="subjects_category2"></a>Subjects category

Line 12 the sub category contains information about subjects.

TODO: describe what a subject is in the context of EnCase.

Line 13 consists of 2 values, namely the values are "0 1".

The 14th line consist of the following tab (0x09) separated values.

| Identifier number | Character in 14rd line | Meaning
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the subject
| 4 | nu | Unknown (Number)
| 5 | co | Unknown (Comment)
| 6 | gu | Unknown (GUID)

Line 15 consists of 2 values, namely the values are "0 0".

Line 16 contains the values defined by line 14.

> Note the default values of some of these values has changed around linen 6.19
> or earlier.

#### FTK Imager (EWF-E01)

Some aspects of this section are:

* In FTK Imager (EWF-E01) the same header section defined twice.
* It is the first and second section of the first segment file. It is not found in subsequent segment files.
* The header data itself is compressed using zlib. Note that the compression level can be none and therefore the header looks uncompressed.
* In FTK Imager the end of line character(s) is a newline (0x0a).

The 3rd and the 4th line consist of the following tab (0x09) separated values:

| Identifier number | Character in 3rd line | Value in 4th line
| --- | --- | ---
| 1 | c | Case number
| 2 | n | Evidence number
| 3 | a | Unique description
| 4 | e | Examiner name
| 5 | t | Notes
| 6 | av | Version, which contains the FTK Imager version used to acquire the media
| 7 | ov | Platform, which contains the platform/operating system used to acquire the media
| 8 | m | Acquisition date and time
| 9 | u | System date and time
| 10 | p | Password hash
| 11 | r | Compression level

Also see [header values](#header_values)

#### EnCase 5 to 7 (EWF-L01)

The EnCase 4 to 7 (EWF-E01) header section specification is also used for the
EnCase 5 to 7 (EWF-L01) format, with the following aspects:

* In EnCase 5 both the acquired and system date and time are set to 0.
* In EnCase 6 and 7 both the acquired and system date and time are set to Jan 1, 1970 00:00:00 (the time is dependent on the local timezone and daylight savings)

#### <a name="header_values"></a>Header values

| Identifier | Description | Notes
| --- | --- | ---
| a | Unique description | Free form string. Note that EnCase might not respond when this value is  large e.g. >= 1 MiB
| av | Version | Free form string. EnCase limits this string to 12 - 1 characters
| c | Case number | Free form string. EnCase limits this string to 3000 - 1 characters
| dc | Unknown |
| e | Examiner name | Free form string. EnCase limits this string to 3000 - 1 characters
| ext | Extents | [Extents header value](#extents_header_value)
| l | Device label | Free form string
| m | Acquisition date and time | Contains a [date and time header value](#date_time_header_value).
| md | Model | Free form string. EnCase limits this string to 3000 - 1 characters
| n | Evidence number | Free form string. EnCase limits this string to 3000 - 1 characters
| ov | Platform | Free form string. EnCase limits this string to 24 -1 characters
| pid | Process identifier | String containing the process identifier (pid) number
| p | Password hash | String containing the password hash. If no password is set it should be simply the character '0'.
| r | Compression level | [Compression header value](#compression_header_value)
| sn | Serial Number | Free form string. EnCase limits this string to 3000 - 1 characters
| t | Notes | Free form string. EnCase limits this string to 3000 - 1 characters
| u | Systemdate and time | Contains a [date and time header value](#date_time_header_value).

> Note the restrictions were tested with EnCase 7.02.01, older versions could
> have a restriction of 40 characters instead of 3000 characters.

##### <a name="date_time_header_value"></a>Date and time header value

In EnCase a date and time contains a string of individual values separated by a
space, e.g. "2002 3 4 10 19 59", which represents March 4, 2002 10:19:59.

In linen a date and time contains a string with a POSIX 32-bit epoch timestamp,
e.g. "1142163845" which represents the date: March 12 2006, 11:44:05


##### <a name="extents_header_value"></a>Extents header value

An extents header value consist of:

```
number of entries
entries that consist of: S <1> <2> <3>
```

##### <a name="compression_header_value"></a>Compression header value

A compression header value consist of a single character that represent the
compression level.

| Character value | Meaning
| --- | ---
| b | Best compression is used
| f | Fastest compression is used
| n | No compression is used

##### Notes

There should not be a tab, carriage return and newline characters within the
text in the 4th line. Or is there a method to escape these characters?

[ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
states that these characters should not be used in the free form text. Need to
confirm this, the specification only speaks of a newline character.

Currently the password has no a additional value than allow an application
check it. The data itself is not protected using the password. The password
hashing algorithm is unknown. Need to find out. And does the algorithm differ
per EnCase version? probably not. The algorithm does not differ in EnCase
1 to 7. FTK Imager does not bother with a password.

### <a name="volume_section"></a>Volume section

The volume section is identified in the section data type field as "volume".
Some aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
* Found in EWF-E01 in EnCase 1 to 7 or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7, and SMART (EWF-S01)
* Found after the header section of the first segment file. Not found in subsequent segment files.

In the next paragraphs the various versions of the volume section are described.

#### EWF specification

The specification according to [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).

The volume section data is 94 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | 0x01 | Unknown (Reserved)
| 4 | 4 | | The number of chunks within the all segment files.
| 8 | 4 | | The number of sectors per chunk, which contains 64 per default.
| 12 | 4 | | The number of bytes per sectors, which contains 512 per default
| 16 | 4 | | The sectors count, the number of sectors within all segment files
| 20 | 20 | 0x00 | Unknown (Reserved)
| 40 | 45 | 0x00 | Unknown (Padding)
| 85 | 5 | | Signature, which contains the EWF file header signature
| 90 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the volume section data.

The number of chunks is a 32-bit value this means it maximum of addressable
chunks would be: 4294967295 (= 2^32 - 1). For a chunk size of 32768 x 4294967295 = about 127 TiB.
The maximum segment file amount is 2^16 - 1 = 65535. This allows for an equal
number of storage if a segment file is filled to its maximum number of chunks.

However Keramics is restricted at 14295 segment files, due to the extension
naming schema of the segment files.

#### SMART (EWF-S01)

The SMART format uses the EWF specification for this section.

In SMART the signature (reverse) value is the string "SMART" (0x53 0x4d 0x41
0x52 0x54) instead of the file header signature.

#### FTK Imager, EnCase 1 to 7 and linen 5 to 7 (EWF-E01)

The specification for FTK Imager, EnCase 1 to 7 and linen 5 to 7.

The volume section data is 1052 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 1 | | [Media type](#media_type)
| 1 | 3 | 0x00 | Unknown (empty values)
| 4 | 4 | | The number of chunks within the all segment files.
| 8 | 4 | | The number of sectors per chunk (or block size), which contains 64 per default. EnCase 5 is the first version which allows this value to be different than 64.
| 12 | 4 | | The number of bytes per sector
| 16 | 8 | | The sectors count, which contains the number of sectors within all segment files. This value probably has been changed in EnCase 6 from a 32-bit value to a 64-bit value to support media >2TiB
| 24 | 4 | | The number of cylinders of the C:H:S value, which most of the time this value is empty (0x00)
| 28 | 4 | | The number of heads of the C:H:S value, which most of the time this value is empty (0x00)
| 32 | 4 | | The number of sectors of the C:H:S value, which most of the time this value is empty (0x00)
| 36 | 1 | | [Media flags](#media_flags)
| 37 | 3 | 0x00 | Unknown (empty values)
| 40 | 4 | | PALM volume start sector
| 44 | 4 | 0x00 | Unknown (empty values)
| 48 | 4 | | SMART logs start sector, which contains an offset relative from the end of media, e.g. a value of 10 would refer to sector = number of sectors - 10
| 52 | 1 | | [Compression level](#compression_level) (Introduced in EnCase 5)
| 53 | 3 | 0x00 | Unknown (empty values, these values seem to be part of the compression level)
| 56 | 4 | | The sector error granularity, which contains the error block size (Introduced in EnCase 5)
| 60 | 4 | 0x00 | Unknown (empty values)
| 64 | 16 | | Segment file set identifier, which contains a GUID/UUID generated on the acquiry system probably used to uniquely identify a set of segment files (Introduced in EnCase 5)
| 80 | 963 | 0x00 | Unknown (empty values)
| 1043 | 5 | 0x00 | Unknown (Signature)
| 1048 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the volume section data.

TODO: a value that could be in the volume is the RAID stripe size

> Note that EnCase requires for media that contains no partition table that the
> is physical media flag is not set and vice versa. Other tools like FTK check
> the actual storage media data.

#### EnCase 5 to 7 (EWF-L01)

The EWF-L01 format uses the EnCase 5 (EWF-E01) volume section specification. However:

* the volume type contains 0x0e
* the number of chunks is 0
* the number of bytes per sectors is some kind of block size value (4096), perhaps the source file system block size
* the sectors count, represents some other value because ( sector_size x sector_amount != total_size ). The total size is in the ltree section.

#### <a name="media_type"></a>Media type

| Value | Identifier | Description
| --- | --- | ---
| 0x00 | | A removable storage media device
| 0x01 | | A fixed storage media device
| | |
| 0x03 | | An optical disc (CD/DVD/BD)
| | |
| 0x0e | | Logical Evidence (LEF or L01)
| | |
| 0x10 | | Physical Memory (RAM) or process memory

> Note that FTK imager versions, before version 2.9, set the storage media to
> fixed (0x01). The exact version of FTK imager where this behavior changed is 
> unknown.

#### <a name="media_flags"></a>Media flags

| Value | Identifier | Description
| --- | --- | ---
| 0x01 | | Is an image file. In FTK Imager, EnCase 1 to 7 this bit is always set, when not set EnCase seems to see the image file as a device
| 0x02 | | Is physical device or device type, where 0 represents a non physical device (logical) and 1 represents a physical device
| 0x04 | | Fastbloc write blocker used
| 0x08 | | Tableau write blocker used. This was added in EnCase 6.13

> Note that if both the the Fastbloc and Tableau write blocker media flags are
> set EnCase only shows the Fastbloc.

#### <a name="compression_level"></a>Compression level

| Value | Identifier | Description
| --- | --- | ---
| 0x00 | | no compression
| 0x01 | | good compression
| 0x02 | | best compression

> Note that EnCase 7 no longer provides the fast and best compression options.

### Disk section

The disk section is identified in the section data type field as "disk". Some
aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Not found in SMART (EWF-S01).

With a disk section in an FTK Imager 2.3 (EWF-E01) image it was confirmed that
the disk section is the same as the volume section.

> Note that the disk section was found only in FTK Imager 2.3 when acquiring a
> physical disk not a floppy. This requires additional research, it is currently
> assumed that the disk section some old method to differentiate between a
> partition (volume) image or a physical disk image.

### Data section

The data section is identified in the section data type field as "data". Some
aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Found in EWF-E01 in EnCase 1 to 7 or linen 5 to 7 or FTK Imager, and EWF-L01 in EnCase 5 to 7. Not found in SMART (EWF-S01).
* For multiple segment files it does not reside in the first segment file. For a single segment file it does.
* Found after the last table2 section in a single segment file or for multiple segment files at the start of the segment files, except for the first.
* The data section has data it should should contain the same information as the volume section.

The data section is a copy of the [volume section](#volume_section).

#### FTK Imager, EnCase 1 to 7 and linen 5 to 7 (EWF-E01)

> Note that in Logicube products (Talon (firmware predating April 2013) and
> Forensic dossier (before version 3.3.3RC16)) the checksum is not calculated
> and set to 0.

### Sectors section

The sectors section is identified in the section data type field as "sectors".
Some aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Found in EWF-E01 in EnCase 2 to 7, or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7. Not found in EnCase 1 (EWF-E01) or SMART (EWF-S01).
* The first sectors section can be found after the volume section in the first segment file or at the after the data section in subsequent segment files. Successive sector data sections are found after the sector table2 section.

The sectors section contains the actual chunks of media data.

* The sectors section can contain multiple chunks.
* The default size of a chunk is 32768 bytes of data (64 standard sectors, with a size of 512 bytes per sector). It is possible in EnCase 5 and 6 and linen 5 and 6 to change the number of sectors per block to 64, 128, 256, 1024, 2048, 4096, 8192, 16384 or 32768. In EnCase 7 and linen 7 this has been reduced to 64, 128, 256, 1024.

#### Data chunk

The first chunk is often located directly after the section header, although
the format does not require this.

When the data is compressed and the compressed data (with checksum) is larger
than the uncompressed data (without the checksum) the data chunk is stored
uncompressed. The default size of a chunk is 32768 bytes of data (64 standard
sectors).

An uncompressed data chunk is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Uncompressed chunk data
| ... | 4 | | Checksum, which contains an Adler-32 of the chunk data

The compressed data chunk consist of [zlib compressed data](zlib.md). The
checksum of the compressed data chunk is part the zlib compressed data format.

#### Optical disc images

For a MODE‑1 CD-ROM optical disc image EnCase only seems to support 2048 bytes
per sector (the data).

The raw sector size of a MODE-1 CD-ROM is 2352 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Synchronization bytes
| 16 | 2048 | | Data
| 2054 | 4 | | Error detection
| 2058 | 8 | 0x00 | Unknown (Empty values)
| 2066 | 276 | | Error correction

TODO: add information about Mode-2 and Mode-XA

### Table section

The table section is identified in the section data type field as "table". Some
aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Found in EWF-E01 in EnCase 1 to 7 or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7, and SMART (EWF-S01)

> Note that the offsets within the section header are 8 bytes (64 bits) of
> size while the offsets in the table entry array are 4 bytes (32 bits) in size.

In the next paragraphs the various versions of the table section are described.

#### EWF specification

Some aspects of the table section according to the EWF specification are:

* The first table section resides after the volume section in the first segment file or after the file header in subsequent segment files.
* It can be found in every segment file.

The table section consists of:

* the table header
* an array of table entries
* the data chunks

##### Table header

The table header is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | The number of entries
| 4 | 16 | 0x00 | Unknown (Padding)
| 20 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the table header data.

According to [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
* the number of entries, contains 0x01
* the table can hold 16375 entries if more entries are required an additional table section should be created.

##### Table entry

The table entry is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Chunk data offset

The most significant bit (MSB) in the chunk data offset indicates if the chunk
is compressed (1) or uncompressed (0).

A chunk data offset points to the start of the chunk of media data, which
resides in the same table section within the segment file. The offset contains
a value relative to the start of the file.

##### Data chunk

The first chunk is often located directly after the last table entry, although
the format does not require this.

A data chunk is always compressed even when no compression is required. This
approach provides a checksum for each chunk. The default size of a chunk is
32768 bytes of data (64 standard sectors). The resulting size of the
"compressed" chunk can therefore be larger than the default chunk size.

> Note that this was deducted from the behavior of FTK Imager for SMART (EWF-S01).

The compressed data chunk consist of [zlib compressed data](zlib.md). The
checksum of the compressed data chunk is part the zlib compressed data format.

#### SMART (EWF-S01)

The table section in the SMART (EWF-S01) format is equivalent to that of the
EWF specification.

#### EnCase 1 (EWF-E01)

Some aspects of this section are:

* The table section resides after the volume section in the first segment file or after the file header in subsequent segment files.
* It can be found in every segment file.

The table section consists of:

* the table header
* an array of table entries
* the table footer
* the data chunks

##### Table header

The table header is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | The number of entries
| 4 | 16 | 0x00 | Unknown (Padding)
| 20 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the table header data.

The table can hold 16375 entries if more entries are required an additional table section should be created.

##### Table entry

The table entry is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Chunk data offset

The most significant bit (MSB) in the chunk data offset indicates if the chunk
is compressed (1) or uncompressed (0).

A chunk data offset points to the start of the chunk of media data, which
resides in the same table section within the segment file. The offset contains
a value relative to the start of the file.

##### Table footer

The table footer is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum, which contains an Adler-32 of the offset array

##### Data chunk

The first chunk is often located directly after the table footer, although the
format does not require this.

When the data is compressed and the compressed data (with checksum) is larger
than the uncompressed data (without the checksum) the data chunk is stored
uncompressed. The default size of a chunk is 32768 bytes of data (64 standard
sectors).

An uncompressed data chunk is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Uncompressed chunk data
| ... | 4 | | Checksum, which contains an Adler-32 of the chunk data

The compressed data chunk consist of [zlib compressed data](zlib.md). The
checksum of the compressed data chunk is part the zlib compressed data format.

#### FTK Imager and EnCase 2 to 5 and linen 5 (EWF-E01)

Some aspects of this section are:

* The table section resides after the sectors section.
* It can be found in every segment file.
* The data chunks are no longer stored in this section but in the sectors section instead.
* The table2 section contains a mirror copy of the table section. In EWF-E01 it is always present.

The table section consists of:

* the table header
* an array of table entries
* the table footer

##### Table header

The sector table header is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | The number of entries
| 4 | 16 | 0x00 | Unknown (Padding)
| 20 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the table header data.

The table section can hold 16375 entries. A new table section should be created
to hold more entries. Both FTK Imager and EnCase 5 can handle more than 16375,
FTK 1 cannot. To contain more than 16375 chunks new sectors, table and table2
sections need to be created after the table2 section.

##### Table entry

The table entry is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Chunk data offset

The most significant bit (MSB) in the chunk data offset indicates if the chunk
is compressed (1) or uncompressed (0).

A chunk data offset points to the start of the chunk of media data, which
resides in the preceding sectors section within the segment file. The offset
contains a value relative to the start of the file.

##### Table footer

The table footer is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum, which contains an Adler-32 of the offset array

#### EnCase 6 to 7 and linen 6 to 7 (EWF-E01)

Some aspects of this section are:

* Every segment file contains its own table section.
* It resides after the sectors section.
* The data chunks are no longer stored in this section but in the sectors section instead.
* The table2 section contains a mirror copy of the table section. In EWF-E01 it is always present.

The table section consists of:

* the table header
* an array of table entries
* the table footer

##### Table header

The sector table header is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | The number of entries
| 4 | 4 | 0x00 | Unknown (Padding)
| 8 | 8 | | The table base offset
| 16 | 4 | 0x00 | Unknown (Padding)
| 20 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the table header data.

As of EnCase 6 the number of entries is no longer restricted to 16375 entries.
The new limit seems to be 65534.

##### Table entry

The table entry is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Chunk data offset

The most significant bit (MSB) in the chunk data offset indicates if the chunk
is compressed (1) or uncompressed (0).

A chunk data offset points to the start of the chunk of media data, which
resides in the preceding sectors section within the segment file. The offset
contains a value relative to the table base offset.

In EnCase 6.7.1 the sectors section can be larger than 2048Mb. The table
entries offsets are 31 bit values in EnCase6 the offset in a table entry value
will actually use *the full 32 bit* if the 2048Mb has been exceeded. This
behavior is no longer present in EnCase 6.8 so it is assumed to be a bug.
Libewf currently assumes that the if the 31 bit value overflows the following
chunks are uncompressed. This allows EnCase 6.7.1 faulty EWF files to be
converted by Keramics.

##### Table footer

The table footer is 4 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum, which contains an Adler-32 of the offset array

#### EnCase 6 to 7 (EWF-L01)

The EWF-L01 format uses the EnCase 6 to 7 (EWF-E01) table section specification.

### Table2 section

The table2 section is identified in the section data type field as "table2".
Some aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Found in EWF-E01 in EnCase 2 to 7, or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7. Not found in EnCase 1 (EWF-E01) or SMART (EWF-S01).
* Uses the same format as the table section.
* Resides directly after the table section.

#### FTK Imager and EnCase 2 to 7 and linen 5 to 7 (EWF-E01)

The table2 section contains a mirror copy of the table section. Probably
intended for recovery purposes.

#### EnCase 5 to 7 (EWF-L01)

The EWF-L01 format uses the EWF-E01 table2 section specification.

### Next section

The next section is identified in the section data type field as "next". Some
aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* Found in EWF-E01 in EnCase 1 to 7 or linen 5 to 7 or FTK Imager, EWF-L01 in EnCase 5 to 7, and SMART (EWF-S01)
* The last section within a segment other than the last segment file.
* The offset to the next section in the section header of the next section point to itself (the start of the next section).
* It should be the last section in a segment file, other than the last segment file.

#### SMART (EWF-S01)

It resides after the table or table2 section.

#### FTK Imager, EnCase and linen (EWF-E01)

It resides after the data section in a single segment file or for multiple
segment files after the table2 section.

In the EnCase (EWF-E01) format the size in the section header is 0 instead of
76 (the size of the section header).

> Note that FTK imager versions before 2.9 sets the section size to 76. At the
> moment it is unknown in which version this behavior was changed.

### Ltypes section

The ltypes section is identifier in the section data type field as "ltypes".
Some aspects of this section are:

* Found in EWF-L01 in of EnCase 7
* Found in the last segment file after table2 section before tree section.

The additional ltypes section data is 6 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 2 | | Unknown
| 2 | 2 | | Unknown
| 4 | 2 | | Unknown

### Ltree section

The ltree section is identifier in the section data type field as "ltree". Some
aspects of this section are:

* Found in EWF-L01 in of EnCase 5 to 7
* Found in the last segment file after ltypes section and before data section.

The ltree section consists of:

* ltree header
* ltree data

#### Ltree header

The ltree header is 48 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | Integrity hash, which contains the MD5 of the ltree data
| 16 | 8 | | Data size
| 24 | 4 | | Checksum, which contains an Adler-32 of all the data within the ltree header where the checksum value itself is zeroed out.
| 28 | 20 | | Unknown (empty values)

#### Ltree data

The ltree data string consists of an UTF-16 little-endian encoded string without
byte order mark. The ltree data is not strict UTF-16 since it allows for unpaired
surrogates, such as "U+d800" and "U+dc00".

Other observed characteristics where the names in the ltree deviate from
the original source:

* \[U+0001-U+0008] were converted to U+00ba
* \[U+0009, U+000a] were stripped
* \[U+000b, U+000c] were converted to U+0020
* U+000d was converted to U+0002
* U+00ba remained the same

Note that this behavior could be related to EnCase as well and might not be
specific for EWF-L01.

The ltree data string contains the following information:

| Line number | Value | Description
| --- | --- | ---
| 1 | 5 | The number of categories provided
| 2 | rec | Information about unknown, also see [Records category](#records_category3)
| ... | | (an empty line)
| ... | perm | Information about file permissions, also see [Permissions category](#permissions_category3)
| ... | | (an empty line)
| ... | srce | Information about acquisition sources, also see [sources category](#sources_category3)
| ... | | (an empty line)
| ... | sub | Information about unknown, also see [subjects category](#subjects_category3)
| ... | | (an empty line)
| ... | entry | Information about file entries, also see [File entries category](#file_entries_category3)
| ... | | (an empty line)

The end of line character(s) is a newline (0x0a).

#### <a name="records_category3"></a>Records category

The rec category contains information about records.

The 1st line of the category contains the string "rec".

The 2nd line of the category contains tab (0x09) separated type indicators.

| Identifier number | Type indicator | Description
| --- | --- | ---
| 1 | tb | Total bytes, which contains an integer with size of the logical file data (media data)
| 2 | cl | Unknown (Clusters?)
| 3 | n | Unknown (introduced in EnCase 6.19)
| 4 | fp | Unknown (introduced in EnCase 7)
| 5 | pg | Unknown (introduced in EnCase 7)
| 6 | lg | Unknown (introduced in EnCase 7)
| 7 | ig | Unknown (introduced in EnCase 7)

The 3rd line of the category consist of the tab (0x09) separated values.

#### <a name="permissions_category3"></a>Permissions category

The perm category contains information about file permissions.

The 1st line of the category contains the string "perm".

The 2nd line consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | | The number of permission groups in the category
| 2 | 1 | Unknown

The 3rd line of the category contains tab (0x09) separated type indicators. For
more information see the sections below.

The remaining lines in the category consist of:

* category root entry
  * zero or more permissions group entries
    * zero or more permission entries

Each entry consist of 2 lines:

| Line number | Value | Description
| --- | --- | ---
| 1 | | Number of entries
| 2 | | Tab (0x09) separated values that correspond to the type indicators.

The 1st line of the category root entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | | The number of permission groups in the category

The 1st line of the permission group entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | | The number of permissions in the group

The 1st line of the permission entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | 0 | Unknown

##### Permission type indicators

| Identifier number | Type indicator | Description
| --- | --- | ---
| 1 | p | Is parent, where 1 represents if the entry is a category root or permissions group and 0 represents if the entry is a permission
| 2 | n | Name, which contains a string
| 3 | s | Security identifier, which contains a string with either a [Windows NT security identifier (SID)](https://github.com/libyal/libfwnt/blob/master/documentation/Security%20Descriptor.asciidoc#security-identifier) or a POSIX user (uid) or group identifier (gid) in the format " number:" such as " 99:"
| 4 | pr | Property type, also see [permission types](#permission_types)
| 5 | nta | Access mask
| 6 | nti | Unknown (Windows NT access control entry (ACE) flags?, which contains an integer with a [Windows NT access control entry (ACE) flags](https://github.com/libyal/libfwnt/blob/master/documentation/Security%20Descriptor.asciidoc#access-control-entry-ace-flags)).
| 7 | nts | Unknown (Permission?) (Removed in EnCase 6)

##### <a name="permission_types"></a>Permission types

| Value | Identifier | Description
| --- | --- | ---
| (empty) | | Owner or category root
| 1 | | Group
| 2 | | Allow
| <td colspan="3"> &nbsp;
| 6 | | Other
| <td colspan="3"> &nbsp;
| 10 | | Unknown (permissions group?)

##### Access mask

Access mask seen in combination with property types 0, 1 and 6

| Value | Identifier | Description
| --- | --- | ---
| (empty) | | Owner or category root
| 0x00000001 | `[Lst Fldr/Rd Data]` | List folder / Read data
| 0x00000002 | `[Crt Fl/W Data]` | Create file / Write data
| <td colspan="3"> &nbsp;
| 0x00000020 | `[Trav Fldr/X Fl]` | Traverse folder / Execute file

Access mask seen in combination with property type 2

```
[0x001200a9] [R&X] [R] [Sync]
[0x001301bf] [M] [R&X] [R] [W] [Sync]
[0x001f01ff] [FC] [M] [R&X] [R] [W] [Sync]
```

| Value | Identifier | Description
| --- | --- | ---
| (empty) | | Owner or category root
| 0x00000001 | |
| 0x00000002 | |
| 0x00000004 | |
| 0x00000008 | |
| 0x00000010 | |
| 0x00000020 | |
| 0x00000040 | |
| 0x00000080 | |
| 0x00000100 | |
| <td colspan="3"> &nbsp;
| 0x00010000 | |
| 0x00020000 | |
| 0x00040000 | |
| 0x00080000 | |
| 0x00100000 | |

#### <a name="sources_category3"></a>Sources category

The srce category contains information about acquisition sources of the file entries.

TODO: describe what an acquisition source is in the context of EnCase.

The 1st line of the category contains the string "srce".

The 2nd line consists of 2 values.

| Value index | Value | Description
| --- | --- | ---
| 1 | | The number of sources in the category
| 2 | 1 | Unknown

The 3rd line of the category contains tab (0x09) separated type indicators. For
more information see the sections below.

The remaining lines in the category consist of:

* category root
  * zero or more source entries

Each entry consist of 2 lines:

| Line number | Value | Description
| --- | --- | ---
| 1 | | Number of entries
| 2 | | Tab (0x09) separated values that correspond to the type indicators.

The 1st line of the category root entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | | The number of sources in the category

The 1st line of the source entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | 0 | Unknown

##### Source type indicators

| Identifier number | Type indicator | Description
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the source
| 4 | ev | Evidence number, which contains a string
| 5 | do | Domain, which contains a string (introduced in EnCase 7.9)
| 6 | loc | Location, which contains a string (introduced in EnCase 7.9)
| 7 | se | Serial number, which contains a string (introduced in EnCase 7.9)
| 8 | mfr | Manufacturer, which contains a string (introduced in EnCase 7.9)
| 9 | mo | Model, which contains a string (introduced in EnCase 7.9)
| 10 | tb | Total bytes, which contains an integer
| 11 | lo | Logical offset, which contains an integer which is -1 when value is not set
| 12 | po | Physical offset, which contains an integer which is -1 when value is not set
| 13 | ah | MD5 hash, which contains a string with the MD5 hash of the source
| 14 | sh | SHA1 hash, which contains a string with the SHA1 hash of the source (introduced in EnCase 6.19)
| 15 | gu | Device GUID, which contains a string with a GUID or "0" if not set
| 16 | pgu | Primary device GUID, which contains a string with a GUID or "0" if not set (introduced in EnCase 7)
| 17 | aq | Acquisition date and time, which contains an integer with a POSIX timestamp
| 18 | ip | IP address, which contains a string (introduced in EnCase 7.9)
| 19 | si | Unknown (Static IP address?), Contains 1 if static, empty otherwise (introduced in EnCase 7.9)
| 20 | ma | MAC address, which contains a string without separator characters (introduced in EnCase 7.9)
| 21 | dt | Drive type, which contains a single character (introduced in EnCase 7.9)

The acquisition date and time is in the form of: "1142163845", which is a POSIX
epoch timestamp and represents the date: March 12 2006, 11:44:05.

If the "ha" value contains "00000000000000000000000000000000" this means the
MD5 hash is not set. The same applies for the "sha" value when it contains
"0000000000000000000000000000000000000000" the SHA1 has is not set.

If the "ma" value contains "000000000000" this means the MAC address is not
set.

##### Drive type

| Character value | Meaning
| --- | ---
| f | Fixed drive

#### <a name="subjects_category3"></a>Subjects category

The sub category contains information about TODO

TODO: describe what a subject is in the context of EnCase.

The 1st line of the category contains the string "sub".

The 2nd line consists of 2 values.

| Value index | Value | Description
| --- | --- | ---
| 1 | | The number of subjects in the category
| 2 | 1 | Unknown

The 3rd line of the category contains tab (0x09) separated type indicators. For
more information see the sections below.

The remaining lines in the category consist of:

* category root
  * zero or more subject entries

Each entry consist of 2 lines:

| Line number | Value | Description
| --- | --- | ---
| 1 | | Number of entries
| 2 | | Tab (0x09) separated values that correspond to the type indicators.

The 1st line of the category root entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | | The number of subject in the category

The 1st line of the subject entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | 0 | Unknown
| 2 | 0 | Unknown

##### Subject type indicators

| Identifier number | Type indicator | Description
| --- | --- | ---
| 1 | p |
| 2 | n |
| 3 | id | Identifier, which contains an integer identifying the subject
| 4 | nu | Unknown (Number)
| 5 | co | Unknown (Comment)
| 6 | gu | Unknown (GUID)

#### <a name="file_entries_category3"></a>File entries category

The entry category contains information about the file entries.

The 1st line of the category contains the string "entry".

The 2nd line consists of 2 values.

| Value index | Value | Description
| --- | --- | ---
| 1 | | The number of file entries in the category or 1 if unknown
| 2 | 1 | Unknown

The 3rd line of the category contains tab (0x09) separated type indicators. For
more information see the sections below.

The remaining lines in the category consist of:

* category root
  * zero or more file entries
    * zero or more sub file entries
      * ...

Each entry consist of 2 lines:

| Line number | Value | Description
| --- | --- | ---
| 1 | | Number of entries
| 2 | | Tab (0x09) separated values that correspond to the type indicators.

The 1st line of the category root entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | | 0 if not set or 26 if Unknown
| 2 | | The number of file entries in the category

The 1st line of the file entry consists of the following 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | | Number of file entries in the parent file entry or 0 if not set
| 2 | | The number of sub file entries in the file entry

##### EnCase 5 and 6 (EWF-L01) file entry type indicators

| Identifier number | Character in 29th line | Meaning
| --- | --- | ---
| 1 | p | Is parent, where 1 => if the entry is a directory and (empty) => if the entry is a file
| 2 | n | [Name](#file_entry_name)
| 3 | id | Identifier, contains an integer identifying the file entry
| 4 | opr | [File entry flags](#file_entry_flags)
| 5 | src | Source identifier, which contains an integer that corresponds to an identifier in the [Sources category](#sources_category3)
| 6 | sub | Subject identifier, which contains an integer that corresponds to an identifier in the [Subjects category](#subjects_category3)
| 7 | cid | Unknown (record type)
| 8 | jq | Unknown
| 9 | cr | Creation date and time
| 10 | ac | Access date and time, for which currently is assumed the precision is date only
| 11 | wr | (File) modification (last written) date and time
| 12 | mo | (File system) entry modification date and time
| 13 | dl | Deletion date and time
| 14 | aq | Acquisition date and time, which contains an integer with a POSIX timestamp
| 15 | ha | MD5 hash, which contains a string with the MD5 hash of the file data
| 16 | ls | File size in bytes. If the file size is 0 the data size should be 1
| 17 | du | Duplicate data offset, relative from the start of the media data
| 18 | lo | Logical offset, which contains an integer which is -1 when value is not set
| 19 | po | Physical offset, which contains an integer which is -1 when value is not set (or does this value contain the segment file in which the start of the data is stored, -1 for a single segment file?)
| 20 | mid | GUID, which contains a string with a GUID (introduced in EnCase 6.19)
| 21 | cfi | Unknown (introduced in EnCase 6.14)
| 22 | be | [Binary extents](#binary_extents)
| 23 | pm | Permissions group index, which contains an integer that corresponds to an identifier in the [Permissions category](#permissions_category3) or -1 if not set.  The value is 0 by default
| 24 | lpt | Unknown (introduced in EnCase 6.19)

The creation, access and last written date and time are in the form of:
"1142163845", which is a POSIX epoch timestamp and represents the date: March
12 2006, 11:44:05.

The "ha" value (Hash) consist of a MD5 hash string when file entries are
hashed. If the "ha" value contains "00000000000000000000000000000000" this
means the MD5 hash is not set.

###### Ltree file entries

The ltree entries of files and directories consist of entries starting with: 0
followed by the number of sub file entries.

The entries of files and directories:

| Line number | Value | Description
| --- | --- | ---
| 1 | (empty) | The root directory
| 2 | | The target drive/mount point
| 3 | | The actual single file entries

##### EnCase 7 (EWF-L01) file entry type indicators

| Identifier number | Character in 29th line | Meaning
| --- | --- | ---
| 1 | mid | GUID, which contains a string with a GUID
| 2 | ls | File size, in bytes. If the file size is 0 the data size should be 1
| 3 | be | [Binary extents](#binary_extents)
| 4 | id | Identifier, which contains an integer identifying the file entry
| 5 | cr | Creation date and time
| 6 | ac | Access date and time
| 7 | wr | (File) modification (last written) date and time
| 8 | mo | (File system) entry modification date and time
| 9 | dl | Deletion date and time
| 10 | sig | Unknown (Introduced in EnCase 7)
| 11 | ha | MD5 hash, which contains a string with the MD5 hash of the file data
| 12 | sha | SHA1 hash, which contains a string with the SHA1 hash of the file data. (Introduced in EnCase 7)
| 13 | ent | Unknown, seen "B" (Introduced in EnCase 7.9)
| 14 | snh | [Short name](#short_name) (or DOS 8.3 name) (Introduced in EnCase 7.9)
| 15 | p | Is parent, where "1" represents that the entry is a directory and "" (an empty string) that the entry is a file
| 16 | n | [Name](#file_entry_name)
| 17 | du | Duplicate data offset, relative from the start of the media data
| 18 | lo | Logical offset, which contains an integer which is -1 when value is not set
| 19 | po | Physical offset, which contains an integer which is -1 when value is not set (or does this value contain the segment file in which the start of the data is stored, -1 for a single segment file?)
| 20 | pm | Permissions group index, which contains an integer that corresponds to an identifier in the [Permissions category](#permissions_category3) or -1 if not set.  The value is 0 by default
| 21 | oes | Unknown (Original extents?) (Introduced in EnCase 7)
| 22 | opr | [File entry flags](#file_entry_flags)
| 23 | src | Source identifier, which contains an integer that corresponds to an identifier in the [Sources category](#sources_category3)
| 24 | sub | Subject identifier, which contains an integer that corresponds to an identifier in the [Subjects category](#subjects_category3)
| 25 | cid | Unknown (record type?)
| 26 | jq | Unknown
| 27 | alt | Unknown (Introduced in EnCase 7)
| 28 | ep | Unknown (Introduced in EnCase 7)
| 29 | aq | Acquisition date and time, which contains an integer with a POSIX timestamp
| 30 | cfi | Unknown
| 31 | sg | Unknown (Introduced in EnCase 7)
| 32 | ea | [Extended attributes](#extended_attributes) (Introduced in EnCase 7.9)
| 33 | lpt | Unknown

If the "ha" value contains "00000000000000000000000000000000" this means the
MD5 hash is not set. The same applies for the "sha" value when it contains
"0000000000000000000000000000000000000000" the SHA1 has is not set.

###### <a name="file_entry_name"></a>File entry name

A file entry name ("n" value):

* can contain path segment separator characters like "\\" and "/"
* uses the "MIDDLE DOT" Unicode character (U+00b7) as a (NTFS) alternative data stream (ADS) name seperator

> Note that a regular "MIDDLE DOT" Unicode character will be encoded in the
> same way so no real way to reliably tell the difference.

An empty name has been observed to be represented as "NoName".

###### <a name="short_name"></a>Short name

The short name ("snh") value contains 2 values:

| Value number | Value | Description
| --- | --- | ---
| 1 | | The number of characters in the short name including the end-of-string character
| 2 | | The short name string, without an end-of-string character

For example: "13 FILE10~1.TXT"

###### Original extents

TODO: add some text

```
1 30a555b 30a6000 12011ae00 9008d7 3f 43 1 12011ae00 30a6000 120113 30a6 9008d7 18530
```

###### Ltree file entries

The ltree entries of files and directories consist of entries starting with: 26
followed by the number of sub file entries.

The entries of files and directories:

| Line number | Value | Description
| --- | --- | ---
| 1 | LogicalEntries | The root directory
| 2 | | The target drive/mount point
| 3 | | The actual single file entries

##### <a name="file_entry_flags"></a>File entry flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | | Unknown (Is read-only?)
| 0x00000002 | Hidden | Is hidden
| 0x00000004 | System | Is system
| 0x00000008 | Archive | Is archive
| 0x00000010 | Sym Link | Is symbolic link, junction or reparse point
| | |
| 0x00000080 | Deleted | Is deleted
| | |
| 0x00001000 | Hard Linked | Is hard link
| 0x00002000 | Stream | Is stream
| | |
| 0x00100000 | Internal | Is internal (used in combination with 0x00000006?)
| | |
| 0x00200000 | Unallocated Clusters | Unknown
| 0x00400000 | | Unknown
| | |
| 0x01000000 | | Unknown
| 0x02000000 | Folder | Is folder
| 0x04000000 | | Data is sparse.

If 0x00002000 or 0x02000000 are not set the file entry is of type "File".

If the sparse data flag is set:

* the data size should be 1 and data should consist of a single byte value.
* the data size should be equal to the file size and data should be the same.

If the duplicate data offset value is not set the single byte value in the data
should be used to reconstruct the file data. E.g. if the file size is 4096 and
the data contains the byte value 0x00 the resulting file should consists of
4096 x 0x00 byte values.

If the duplicate data offset value is set the single byte in the data is
ignored and the duplicate data offset refers to the location where the data
stored.

##### <a name="binary_extents"></a>Binary extents value

The binary extents value contains 3 values separated by a space:

```
Unknown Offset Size
```

Where:

* unknown always is 1, could this be the number of extents?
* extent data offset, relative from the start of the media data
* extent data size

The offset and size are specified in hexadecimal values.

> Note that the binary extents value contains only 1 value for the first single file entry.

##### <a name="extended_attributes"></a>Extended attributes value

The extended attributes value contains base-16 encoded data, which consists of:

* Extended attributes header (stored as an extended attribute)
* One or more extended attributes

###### Extended attributes header

The extended attributes header is 37 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | 0 | Unknown (0 => root, 1 => otherwise)
| 4 | 1 | 1 | Unknown (0 => is leaf node, 1 => is branch node?)
| 5 | 4 | 11 | Number of characters in name string including the end-of-string character
| 9 | 4 | 1 | Number of characters in value string including the end-of-string character
| 13 | 22 | "Attributes\0" | Name string, which contains an UTF-16 little-endian encoded string including end-of-string character
| 35 | 2 | "\0" | Value string, which contains an UTF-16 little-endian encoded string including end-of-string character

###### Extended attribute

An extended attributes is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Unknown (0 => root, 1 => otherwise)
| 4 | 1 | | Unknown (0 => is leaf node, 1 => is branch node?)
| 5 | 4 | | Number of characters in name string including the end-of-string character
| 9 | 4 | | Number of characters in value string including the end-of-string character
| 13 | ... | | Name string, which contains an UTF-16 little-endian encoded string including end-of-string character
| ... | ... | | Value string, which contains an UTF-16 little-endian encoded string including end-of-string character

TODO: complete section

> Note that branch nodes are presuably used to group attributes, however these
> are not used consistently and are not shown by EnCase 7.

### Map section

Some aspects of this section are:

* Found in EWF-L01 in of EnCase 7 (First seen in EnCase 7.4.1.10)
* Found in the last segment file after data section before done section.

The map consists of:

* map string
* map entries array

#### Map string

The map string consists of an UTF-16 little-endian encoded string without the
UTF-16 endian byte order mark.

The map string contains the following information:

| Line number | Value | Description
| --- | --- | ---
| 1 | 1 | The number of categories provided
| 2 | r | Probably the type of information provided
| 3 | c | Identifier for the values in the 4th line
| 4 | | The data for the different identifiers in the 3rd line
| 5 | | (an empty line)

##### Map string values

| Identifier number | Character in 29th line | Meaning
| --- | --- | ---
| 1 | C | Number of map entries (count)

The number of map entries should match the number of file entries in the ltree.

#### Map entry

A map entry is 24 bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Unknown
| 4 | 4 | | Unknown (empty values or part of previous value)
| 8 | 16 | | Unknown

### Session section

The session section is identifier in the section data type field as "session". Some aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* It is not found in SMART (EWF-S01) and FTK Imager (EWF-E01).
* It is found in EnCase 5 and 6 (EWF-E01) files.
* It is only added to the last segment file for images of optical disc (CD/DVD/BD) media.
* It is found after the data section and before the error2 section.

The session section data consists of:

* The session header
* The session entries array
* The session footer

#### Session header

The session header is 36 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Number of sessions
| 4 | 28 | | Unknown (empty values)
| 32 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the additional session section data.

#### Session entry

A session entry is 32 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Flags
| 4 | 4 | | Start sector
| 8 | 24 | | Unknown (empty values)

EnCase stores audio tracks as 0 byte data with a sector size of 2048.

> Note that for a CD the first session sector is stored as 16, although the
> actual session starts at sector 0. Could this value be overloaded to indicate
> the size of the reserved space between the start of the session and the ISO
> 9660 volume descriptor.

#### Session flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | | If set the track is an audio track otherwise the track is a data track

#### Session footer

The session footer is 4 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum, which contains an Adler-32 of all the data within the session entries array

### Error2 section

The error2 section is identifier in the section data type field as "error2".
Some aspects of this section are:

* Not defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* It is not found in SMART (EWF-S01).
* It is found in, EnCase 3 to 7 and linen 5 to 7 (EWF-E01) files.
* It is only added to the last segment file when errors were encountered while reading the input.

TODO: check FTK Imager, EnCase 1 and 2 for presence of the error2 section.

It contains the sectors that have read errors. The sector where a read error
occurred are filled with zero's during acquiry by EnCase.

The error2 section data consists of:

* The error2 header
* The error2 entries array
* The error2 footer

#### Error2 header

The error2 header is 520 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Number of entries
| 4 | 512 | | Unknown (empty values)
| 516 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the error2 header data.

#### Error2 entry

An error2 entry is 8 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Start sector
| 4 | 4 | | The number of sectors

#### Error2 footer

The error2 footer is 4 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | Checksum, which contains an Adler-32 of all the data within the error2 entries array

### Digest section

The digest section is identified in the section data type field as "digest".
Some aspects of this section are:

* It is found in EnCase 6 to 7 files, as of EnCase 6.12 and linen 6.12 (EWF-E01).

The digest section contains a MD5 and/or SHA1 hash of the data within the chunks.

The digest section data is 80 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | MD5 hash of the media data
| 16 | 20 | | SHA1 hash of the media data
| 36 | 40 | 0x00 | Unknown (Padding)
| 76 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the digest section data.

### Hash section

The hash section is identified in the section data type field as "hash". Some
aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* It is found in SMART (EWF-S01) and FTK Imager, EnCase 1 to 7 and linen 5 to 7 (EWF-E01) files.
* It is not found in EnCase 5 (EWF-L01).
* The hash section is optional, it does not need to be present. If it does it resides in the last segment file before the done section.

The hash section contains a MD5 hash of the data within the chunks.

The hash section data is 36 byte in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 16 | | MD5 hash of the media data
| 16 | 16 | | Unknown
| 32 | 4 | | Checksum, which contains an Adler-32 of all the previous data within the additional hash section data.

#### Notes

Observations regarding the unknown value:

* is zero in SMART
* is zero in EnCase 3 and below
* in EnCase 4 the first 4 bytes are 0, the next 8 bytes seem random, the last 4 bytes seem fixed
* in EnCase 5 and 6 the first 8 bytes seem random, the last 8 bytes equal the file header signature
* in linen 5 the first and last set of 4 bytes seem the same, the second set of 4 bytes seem to be random, the third set of 4 bytes seem to contain a piece of the file header signature
* in linen 6 the first and third set of 4 bytes seem random, the second and last set of 4 bytes seem to be the same
* EnCase5 seems to contain a GUID of the acquired device?

Test with EnCase 4 show that:

* The value does not equal the checksum of the media data
* Does not differentiate for the same media acquired within the same program session, using different formats, but differ for different media and different program sessions

### Done section

The done section is identified in the section data type field as "done". Some
aspects of this section are:

* Defined in [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html).
* It is found in SMART (EWF-S01), FTK Imager, EnCase 1 to 7 and linen 5 to 7 (EWF-E01) and EnCase 5 (EWF-L01) files.
* The done section is the last section within the last segment file.
* The offset to the next section in the section header of the done section point to itself (the start of the done section).
* It should be the last section in the last segment file.

#### SMART (EWF-S01)

It resides after the table or table2 section.

#### FTK Imager, EnCase and linen (EWF-E01)

It resides after the data section in a single segment file or for multiple
segment files after the table2 section.

In the EnCase (EWF-E01) format the size in the section header is 0 instead of
76 (the size of the section header).

> Note that FTK imager versions before 2.9 sets the section size to 76. At the
> moment it is unknown in which version this behavior was changed.

#### Incomplete section

The incomplete section is identified in the section data type field as
"incomplete".

This section is seen rarely. It was seen in an EnCase 6.13 (EWF-E01) file as
the last last section within the last segment file. The incomplete section was
preceded by a hash and digest section, although later in the set of EWF files
another hash and digest section were defined.

It is currently assumed that the incomplete section indicates an incomplete
image created using remote imaging. The incomplete section contains data but
currently there is no indication what purpose the data has.

## EWF-X

EWF-X (extended) is an experimental format to enhance the EWF format. EWF-X is
based on the EWF-E01 format. EWF-X does not limit the table entries to 16375.
EWF-X is not the same as version 2 of EWF.

TODO: add note about the table entry limit.

### Sections

Additional sections provided in the EWF-X format are:

* xheader
* xhash

#### Xheader

The xheader section contains [zlib compressed data](zlib.md) containing XML
data containing the header values.

```
<?xml version="1.0" encoding="UTF-8"?>
<xheader>
        <case_number>1</case_number>
        <description>Description</description>
        <examiner_name>John D.</examiner_name>
        <evidence_number>1.1</evidence_number>
        <notes>Just a floppy in my system</notes>
        <acquiry_operating_system>Linux</acquiry_operating_system>
        <acquiry_date>Sat Jan 20 18:32:08 2007 CET</acquiry_date>
        <acquiry_software>ewfacquire</acquiry_software>
        <acquiry_software_version>20070120</acquiry_software_version>
</xheader>
```

#### Xhash

The xhash section contains [zlib compressed data](zlib.md) containing XML data
containing the hash values.

```
<?xml version="1.0" encoding="UTF-8"?>
<xhash>
        <md5>ae1ce8f5ac079d3ee93f97fe3792bda3</md5>
        <sha1>31a58f090460b92220d724b28eeb2838a1df6184</sha1>
</xhash>
```

### GUID

EWF-X uses a random based version of the GUID

## Corruption scenarios

This chapter contains several corruption scenarios that have been encountered
"in the wild".

### Corrupt uncompressed chunk

TODO: add description

### Corrupt compressed chunk

TODO: add description

### DEFLATE uncompressed block data with copy of uncompressed data size of 0

Seen in combination with some firmware versions of Tableau TD3 forensic imager.

In this corruption scenarion the copy of uncompressed data size value of the
[DEFLATE uncompressed block data](zlib.md) is set to 0 instead of the 1s
complement of the uncompressed data size.

Libewf currently does not handle this corruption scenario.

### Corrupt section header

TODO: add description

```
reading section header from file IO pool entry: 1 at offset: 415912423
type                      : table2
next offset               : 415978027
size                      : 65604
checksum                  : 0xf35f03e0
number of offsets         : 16375
base offset               : 0x00000000
checksum                  : 0x180d0137

reading section header from file IO pool entry: 1 at offset: 415978027
type                      : sectors
next offset               : 415978027
size                      : 0
checksum                  : 0x1ad00464
```

### Corrupt table section

TODO: add description

Scenarios:

* with and with out table 2
* corruption in number of entries
* corruption in entry data

### Corrupted segment file header

TODO: add description

### Partial segment file

TODO: add description

### Missing segment file(s)

TODO: add description

### Dual image: section size versus offset

The section headers define both the next section offset and the size of the
section. If an implementation reads only one of the two to determine the next
section, a dual EWF image can be crafted that consists of two separate images
including hashes.

Keramics will mark such an image as corrupted.

### Table entries offset overflow

In EnCase 6.7.1 the sectors section can be larger than 2048 MiB. The table
entries offsets are 31 bit values in EnCase6 the offset in a table entry value
will actually use the full 32 bit if the 2048 MiB has been exceeded. This
behavior is no longer present in EnCase 6.8 so it is assumed to be a bug.

Libewf currently assumes that the if the 31 bit value overflows the following
chunks are uncompressed. This allows EnCase 6.7.1 faulty EWF files to be
converted by Keramics.

### Multiple incomplete segment file set identifiers

Although rare it can occur that a set of EWF image files changes its segment
file set identifier. This was seen in an image created by EnCase 6.13,
presumably using remote imaging. The image contained 3 different segment file
set identifiers. The first changes after an incomplete section. The second one
changed without any clear indication. The corresponding data section also
changed in some extent e.g. compression method and media flags, the is physical
flag being dropped. The change was consistent across multiple segment files. It
is unlikely that deliberate manipulation is involved. EnCase considers the
image as invalid.

Although with some tweaking of the individual segment file sets could be read.
In this case the data read from the segment file sets was heavily corrupted.
For now Keramics does not support reading multiple segment files sets from a
single image, but this might change in the future.

## AD encryption

As of version 2.8 FTK Imager supports "AD encryption". Although the output file
uses the EWF extensions the file actually is a AES-256 encrypted container. The
EWF can be encrypted using a pass-phrase or a certificate.

TODO: link to format definition

## References

* [ASR Data - E01 Compression Format](http://www.asrdata.com/whitepaper-html)
* [RFC4122 - A Universally Unique Identifier (UUID) URN Namespace](http://www.ietf.org/rfc/rfc4122.txt)
