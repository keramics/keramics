# QEMU Copy-On-Write (QCOW) image file format

The QEMU Copy-On-Write (QCOW) image file format is used by the QEMU Open Source
Process Emulator to store disk images (storage media)

## Overview

A QCOW image file consists of:

* the file header
  * optional file header extensions
* the level 1 table (cluster aligned)
* the reference count table (cluster aligned)
* reference count blocks
* snapshot headers (8-byte aligned on cluster boundary)
* clusters containing:
  * level 2 tables
  * storage media data

The storage media data is stored in clusters. Each cluster is a multitude of
512 bytes. The level 1 (L1) table contains level 1 reference of level 2 (L2)
tables. The level 2 tables contain level 2 references of the storage media
clusters.

There are multiple versions of the QCOW image file format. QCOW (version 1)
and QCOW2 (version 2 and later) are sometimes considered even as separate image
formats. Version 3 is considered as an extended version of QCOW2.

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | big-endian in most cases, note that some values are in little-endian
| Date and time values | Number of seconds since Jan 1, 1970 00:00:00 UTC (POSIX epoch)
| Character strings | UTF-8

> Note that this docuement assumes that character strings are stored in UTF-8

The number of bytes per sector is 512.

### Encryption

The QCOW image format can encrypted the media data stored in the image format.
Currently supported encryption methods are:

* AES-CBC 128-bit
* Linux Unified Key Setup (LUKS)

If no encryption is used the encryption method in the file header is set to
none (0).

> Note it is currently unknown if the format supports compression and encryption
> at the same time. It does not appear to be supported by qemu-img.

#### AES-CBC 128-bit

Both encryption and decryption use:

* AES-CBC with a 128-bits key decryption of sector data

The key is direct copy of the first 16 characters of a user provided (narrow
character) password. If the password is smaller than 16 characters. The
remaining key data is set to 0-byte values.

> Note that it is currently unclear which character sets are allowed and how
> characters outside the 7-bit ASCII set should be handled.

The initialization vector of the AES-CBC is using media data sector number
(relative to the start of the disk) in little-endian format as the first 64
bits of the 128 bit initialization vector. The remaining initialization vector
data is set to 0-byte values. The first sector number is 0 and the bytes per
sector are 512.

#### Linux Unified Key Setup (LUKS)

TODO: complete section

## File header

### File header – version 1

The file header - version 1 is 48 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "QFI\xfb" or "\x51\x46\x49\xfb" | The signature (or magic identifier)
| 4 | 4 | 1 | Format version
| 8 | 8 | | Backing file name offset
| 16 | 4 | | Backing file name size
| 20 | 4 | | Modification date and time, which contains a POSIX timestamp
| 24 | 8 | | Storage media size
| 32 | 1 | | Number of cluster block bits
| 33 | 1 | | Number of level 2 table bits
| 34 | 2 | | [yellow-background]*Unknown (empty values)*
| 36 | 4 | | Encryption method
| 40 | 8 | | Level 1 table offset

The cluster block size is calculated as:

```
cluster block size = 1 << number of cluster block bits
```

The level table 2 size is calculated as:

```
level table 2 size = ( 1 << number of level 2 table bits ) * 8
```

The level 1 table size is calculated as:

```
level 1 table size = cluster block size * ( 1 << number of level 2 table bits )

if( media size % level 1 table size != 0 )
{
    level 1 table size = ( media size / level 1 table size ) + 1
}
else
{
    level 1 table size = media size / level 1 table size
}
level 1 table size *= 8
```

The backing file name is set in snapshot image files and is normally stored
after the file header.

### File header – version 2

The file header - version 2 is 72 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "QFI\xfb" or "\x51\x46\x49\xfb" | The signature (or magic identifier)
| 4 | 4 | 2 | Format version
| 8 | 8 | | Backing file name offset
| 16 | 4 | | Backing file name size
| 20 | 4 | | Number of cluster block bits
| 24 | 8 | | Storage media size
| 32 | 4 | | Encryption method
| 36 | 4 | | Number of level 1 table references
| 40 | 8 | | Level 1 table offset
| 48 | 8 | | Reference count table offset
| 56 | 4 | | Reference count table clusters
| 60 | 4 | | Number of snapshots
| 64 | 8 | | Snapshots offset

The cluster block size is calculated as:

```
cluster block size = 1 << number of cluster block bits
```

The number of level 2 table bits is calculated as:

```
number of level 2 table bits = number of cluster block bits - 3
```

The level table 2 size is calculated as:

```
level table 2 size = ( 1 << number of level 2 table bits ) * 8
```

The level 1 table size is calculated as:

```
level 1 table size = number of level 1 table references * 8
```

The backing file name is set in snapshot image files and is normally stored
after the file header.

### File header – version 3

The file header - version 3 is 104 or 112 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | "QFI\xfb" or "\x51\x46\x49\xfb" | The signature (or magic identifier)
| 4 | 4 | 3 | Format version
| 8 | 8 | | Backing file name offset
| 16 | 4 | | Backing file name size
| 20 | 4 | | Number of cluster block bits
| 24 | 8 | | Storage media size
| 32 | 4 | | Encryption method
| 36 | 4 | | Number of level 1 table references
| 40 | 8 | | Level 1 table offset
| 48 | 8 | | Reference count table offset
| 56 | 4 | | Reference count table clusters
| 60 | 4 | | Number of snapshots
| 64 | 8 | | Snapshots offset
| 72 | 8 | | Incompatible feature flags
| 80 | 8 | | Compatible feature flags
| 88 | 8 | | Auto-clear feature flags
| 96 | 4 | | Reference count order
| 100 | 4 | 104 or 112 | File header size, which contains the size of the file header, this value does not include the size of the file header extensions
| <td colspan="4"> *If file header size equals 112*
| 104 | 1 | | Compression method
| 105 | 7 | | Unknown (padding)

The cluster block size is calculated as:

```
cluster block size = 1 << number of cluster block bits
```

The number of level 2 table bits is calculated as:

```
number of level 2 table bits = number of cluster block bits - 3
```

The level table 2 size is calculated as:

```
level table 2 size = ( 1 << number of level 2 table bits ) * 8
```

The level 1 table size is calculated as:

```
level 1 table size = number of level 1 table references * 8
```

The backing file name is set in snapshot image files and is normally stored
after the file header.

### Encryption methods

| Value | Identifier | Description
| --- | --- | ---
| 0 | QCOW_CRYPT_NONE | No encryption
| 1 | QCOW_CRYPT_AES | AES-CBC 128-bits encryption
| 2 | QCOW_CRYPT_LUKS | Linux Unified Key Setup (LUKS) encryption

### Incompatible feature flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | QCOW2_INCOMPAT_DIRTY |
| 0x00000002 | QCOW2_INCOMPAT_CORRUPT |
| 0x00000004 | QCOW2_INCOMPAT_DATA_FILE |
| 0x00000008 | QCOW2_INCOMPAT_COMPRESSION |
| 0x00000010 | QCOW2_INCOMPAT_EXTL2 |

### Compatible feature flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | QCOW2_COMPAT_LAZY_REFCOUNTS |

### Auto-clear feature flags

| Value | Identifier | Description
| --- | --- | ---
| 0x00000001 | QCOW2_AUTOCLEAR_BITMAPS |
| 0x00000002 | QCOW2_AUTOCLEAR_DATA_FILE_RAW |

### Compression methods

| Value | Identifier | Description
| --- | --- | ---
| 0 | | ZLIB compression

### File header extensions

A file header extension consist of:

* file header extension header
* file header extension data

#### File header extension header

The file header extension header is 8 bytes in size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 4 | | The extension type (signature)
| 4 | 4 | | The extension data size

#### File header extension types

| Value | Identifier | Description
| --- | --- | ---
| 0x0537be77 | QCOW2_EXT_MAGIC_CRYPTO_HEADER | Crypto header
| 0x23852875 | QCOW2_EXT_MAGIC_BITMAPS | Bitmaps
| 0x44415441 or "DATA" | QCOW2_EXT_MAGIC_DATA_FILE | Data-file
| 0x6803f857 | QCOW2_EXT_MAGIC_FEATURE_TABLE | Feature table
| 0xe2792aca | QCOW2_EXT_MAGIC_BACKING_FORMAT | Backing format

#### Backing format file header extension

The backing format file header extension header is of variable size and consist
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Backing format identifier, which contains an UTF-8 string without end-of-string character

#### Bitmaps file header extension

TODO: complete section

#### Crypto header file header extension

The crypto header file header extension header is 16 bytes in size and consist
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | The crypto data offset
| 8 | 8 | | The crypto data size

#### Data-file file header extension

The data-file file header extension header is of variable size and consist of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | ... | | Data-file filename, which contains an UTF-8 string without end-of-string character

#### Feature table file header extension

TODO: complete section

## Level 1 table

The level 1 table contains level 2 table references.

A reference value of 0 represents unused or unallocated and is considered as
sparse or stored in a corresponding backing file.

### Level 2 table reference – version 1

The level 2 table reference is 8-bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 63 bits | | Level 2 table offset, which contains an offset relative from the start of the file
| 7.7 | 1 bit | QCOW_OFLAG_COMPRESSED | Is compressed flag

### Level 2 table reference – version 2 or 3

The level 2 table reference is 8-bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 62 bits | | Level 2 table offset, which contains an offset relative from the start of the file
| 7.6 | 1 bit | QCOW_OFLAG_COMPRESSED | Is compressed flag
| 7.7 | 1 bit | QCOW_OFLAG_COPIED | Is copied flag

The is copied flag indicates that the reference count of the corresponding
level 2 table is exactly one.

## Level 2 table

The level 2 table contains cluster block references.

The level 2 table size is calculated as:

```
level 2 table size = ( 1 << number of level 2 table bits ) * 8
```

A reference value of 0 represents unused or unallocated and is considered as
sparse or stored in a corresponding backing file.

### Cluster block reference – version 1

The cluster block reference - version 1 is 8-bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 63 bits | | Cluster block offset, which contains an offset relative to the start of the cluster block
| 7.7 | 1 bit | QCOW_OFLAG_COMPRESSED | Is compressed flag

### Cluster block reference – version 2 or 3

The cluster block reference - version 2 or 3 is 8-bytes in size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0.0 | 62 bits | | Cluster block offset, which contains an offset relative to the start of the cluster block
| 7.6 | 1 bit | QCOW_OFLAG_COMPRESSED | Is compressed flag
| 7.7 | 1 bit | QCOW_OFLAG_COPIED | Is copied flag

The is copied flag indicates that the reference count of the corresponding
cluster block is exactly one.

## Reference count table

The cluster data blocks are referenced counted. For every cluster data block a
16-bit reference count is stored in the reference count table.

The reference count table is stored in cluster block sizes. The file header
contains the number of blocks (or reference count table clusters).

TODO: complete section

### Notes

```
reference count cluster block offset = cluster data block offset /
reference count table offset = cluster data block /

In order to obtain the reference count of a given cluster, you split the
cluster offset into a refcount table offset and refcount block offset.

Since a refcount block is a single cluster of 2 byte entries, the lower
cluster_size - 1 bits is used as the block offset and the rest of the bits are
used as the table offset.

One optimization is that if any cluster pointed to by an L1 or L2 table entry
has a refcount exactly equal to one, the most significant bit of the L1/L2
entry is set as a "copied" flag. This indicates that no snapshots are using
this cluster and it can be immediately written to without having to make a copy
for any snapshots referencing it.
```

## Cluster data block

To retrieve a cluster data block corresponding a certain storage media offset:

Determine the level 1 table index from the offset:

```
level 1 table index bit shift = number of cluster block bits + number of level 2 table bits
```

For version 1:

```
level 1 table index = ( offset & 0x7fffffffffffffffULL ) >> level 1 table index bit shift
```

For version 2 and 3:

```
level 1 table index = ( offset & 0x3fffffffffffffffULL ) >> level 1 table index bit shift
```

Retrieve the level 2 table offset from the level 1 table. If the level table 2
offset is 0 and the image has a backing file the cluster data block is stored
in the backing file otherwise the cluster block is considered sparse.

Read the corresponding level 2 table.

Determine the level 2 table index from the offset:

```
level 2 table index bit mask = ~( 0xffffffffffffffffULL << number of level 2 table bits )
```

```
level 2 table index = ( offset >> number of cluster block bits ) >> level 2 table index bit mask
```

Retrieve the cluster block offset from the level 2 table. If the cluster block
offset is 0 and the image has a backing file the cluster data block is stored
in the backing file otherwise the cluster block is considered sparse.

### Uncompressed cluster data block

If the is compressed flag (QCOW_OFLAG_COMPRESSED) is not set:

```
cluster block bit mask = ~( 0xffffffffffffffffULL << number of cluster block bits )
```

```
cluster block data offset = ( offset & cluster block bit mask ) + cluster block offset
```

Note that in version 2 or 3 the last cluster block in the file can be smaller than
the cluster block size defined by the number of cluster block bits in the file
header. This does not seem to be the case for version 1.

### Compressed cluster data block

If the is compressed flag (QCOW_OFLAG_COMPRESSED) is set the cluster block data
is stored using the compression method defined by the file header or DEFLATE by
default.

Multiple compressed cluster data blocks are stored together in cluster block
sizes. The compressed cluster data blocks are sector (512 bytes) aligned.

The compressed data uses a DEFLATE (inflate) window bits value of -12

#### Compressed chunk data block – version 1

```
compressed size bit shift = 63 - number of cluster block bits
```

```
compressed block size = ( ( cluster block offset & 0x7fffffffffffffffULL ) >> compressed size bit shift
```

```
compressed block offset &= ~( 0xffffffffffffffffULL << compressed size bit shift )
```

#### Compressed chunk data block – version 2 or 3

```
compressed size bit shift = 62 - ( number of cluster block bits – 8 )
```

According to "the QCOW2 Image Format" the compressed block size is calculated
as following:

```
compressed block size = ( ( ( cluster block offset & 0x3fffffffffffffffULL ) >> compressed size bit shift ) + 1 ) * 512
```

Since the compressed block size is stored in 512 byte sectors this value does
not contain the exact byte size of the compressed cluster block data. It
sometimes lacks the size of the last partially filled sector and one sector
should be added if possible within the bounds of the cluster blocks size and
the file size.

```
cluster block offset &= ~( 0xffffffffffffffffULL << compressed size bit shift )
```

## Snapshots

As of version 1 QCOW can use the backing file name in the file header to point
to a backing file (or parent image) that contains the snapshot image where the
current image only contains the modifications. Version 2 adds support to store
snapshot inside the image.

### Snapshot header - version 2 or 3

An in-image snapshot is created by adding a snapshot header, copying the L1
table and incrementing the reference counts of all L2 tables and data clusters
referenced by the L1 table.

The snapshot header is of variable size and consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 8 | | Level 1 table offset
| 8 | 4 | | Level 1 size
| 12 | 2 | | Identifier string size
| 14 | 2 | | Name size
| 16 | 4 | | Date in seconds
| 20 | 4 | | Date in nano seconds
| 24 | 8 | | VM clock in nano seconds
| 32 | 4 | | VM state size
| 36 | 4 | | Extra data size
| 40 | ...  | | Extra data
| ...  | ...  | | Identifier string size
| ...  | ...  | | Name

TODO: complete section

## References

* [The QCOW Image Format](https://web.archive.org/web/20201006212750/https://people.gnome.org/~markmc/qcow-image-format-version-1.html), by Mark McLoughlin
* [The QCOW2 Image Format](https://web.archive.org/web/20121004073848/http://people.gnome.org/~markmc/qcow-image-format.html), by Mark McLoughlin
