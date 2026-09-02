# Volume Shadow Snapshot (volsnap) format

As of Windows Vista the Volume Shadow Snapshot (VSS) stores persistent shadow copies on the local
NTFS volume.

## Overview

According to "Shadow Copies and Shadow Copy Sets" a shadow copy is a snapshot of a volume. A shadow
copy can be part of a set which contains a collection of shadow copies of various volumes, taken at
the same time.

Volume Shadow Snapshot (VSS) can use different providers to store shadow copies, this document
focuses on the "Microsoft Software Shadow Copy provider 1.0" (GUID:
b5946137-7b9f-4925-af80-51abd60b20d5) and will refer to it as volsnap. The volsnap provider stores
the copies on the local volume using 16 KiB blocks.

Volsnap uses the GUID 3808876b-c176-4e48-b7ae-04046e6cc752 to identify its data or metadata files.
It leverages several metadata files in "\\System Volume Information" directory:

* Volsnap catalog; stored in the metadata file named {%VOLSNAPGUID%}
* Volsnap store; stored in the metadata file named {%GUID%}{%VOLSNAPGUID%}

Where %VOLSNAPGUID% (\_VSP_DIFF_AREA_FILE_GUID) contains the volsnap identifier and
%GUID% contains a time/MAC based GUID.

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Date and time values | FILETIME in UTC |
| Character strings | UCS-2 little-endian, which allows for unpaired Unicode surrogates such as "U+d800" and "U+dc00" |

## Volume header

The volsnap volume header is part of the NTFS volume header (or $Boot metadata file). The volsnap
volume header data is stored at offset 7680 (0x1e00) of the volume and is at least 100 bytes in
size, but presumably 512 bytes, and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | volsnap identifier, which contains a GUID |
| 16 | 4 | | Format version |
| 20 | 4 | 0x01 | Record type |
| 24 | 8 | 0x1e00 | Current offset, which is relative to the start of the volume |
| 32 | 8 | 0x1e00 | Unknown (Next offset?), which is relative to the start of the volume |
| 40 | 8 | | Unknown (empty value) |
| 48 | 8 | | Catalog offset, which is relative to the start of the volume or contains 0 if there is no catalog |
| 56 | 8 | | Maximum size, in number of bytes or contains 0 if unbounded |
| 64 | 16 | | Volume identifierwhich contains a GUID |
| 80 | 16 | | Shadow copy storage volume identifier, which contains a GUID |
| 96 | 4 | | Unknown |
| 100 | 412 | | Unknown (empty values) |

### Version

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | | Windows Vista, Windows 7 |
| 2 | | Windows 8 |

## Catalog

The catalog contains information about the individual stores. The catalog consists of one or more
catalog blocks. Each catalog block is 16384 (0x4000) bytes of size and consists of:

* catalog block header
* an array of catalog entries

The volsnap catalog metadata files contains the catalog blocks stored directly after one-and-other.

If the volume does not contain a catalog when there are no snapshots (stored) but volsnap is
enabled.

### Catalog block header

The catalog block header is 128 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | volsnap identifier, which contains a GUID |
| 16 | 4 | 0x01 | Version |
| 20 | 4 | 0x02 | Record type |
| 24 | 8 | | Relative (catalog block) offset, which is relative to the start of the first catalog block |
| 32 | 8 | | Current (catalog block) offset, which is relative to the start of the volume |
| 40 | 8 | | Next (catalog block) offset, which is relative to the start of the volume or contains 0 if this is the last block |
| 48 | 80 | | Unknown (empty values) |

### Catalog entry

Each catalog entry consists of a catalog entry type 0x02. A corresponding type 0x03 is required if
the shadow copy is stored in a store, which is the case as of Windows Vista.

> Note that a Windows 2003 R2 catalog does not contain catalog entry type 0x03.

TODO: Determine how Windows 2003 R2 volumes store the snapshot data

The type 0x02 and type 0x03 entries are not necessarily stored directly after one-and-other and can
be scattered over the catalog. For now it is assumed that entry type 0x02 must be defined before
entry type 0x03.

Also these entries are not necessarily stored in order of age.

There can be unused catalog entries (of type 0x01) as well. Empty catalog entries seem to consist
entirely of 0-bytes.

#### Unused catalog entry (type 0x01)

An unused catalog entry (type 0x01) is 128 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | 0x01 | Catalog entry type |
| 8 | 120 | | Unknown (empty values) |

#### Catalog entry type 0x02

A catalog entry type 0x02 is 128 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | 0x02 | Catalog entry type |
| 8 | 8 | | Volume size |
| 16 | 16 | | Store identifier, which contains a GUID that is used in the store filename |
| 32 | 8 | | Unknown (Sequence number) |
| 40 | 8 | | Unknown (Flags?), seen 0x40 in Windows in Vista and 7 and 0x440 in Windows 8 (file backup?) |
| 48 | 8 | | Shadow copy creation time, which contains a FILETIME |
| 56 | 72 | | Unknown (empty values) |

#### Catalog entry type 0x03

A catalog entry type 0x03 is 128 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | 0x03 | Catalog entry type |
| 8 | 8 | | Store block list offset, which is relative to the start of the volume |
| 16 | 16 | | Store identifier, which contains a GUID, that is used in the store filename |
| 32 | 8 | | Store header offset, which is relative to the start of the volume |
| 40 | 8 | | Store block range list offset, which is relative to the start of the volume |
| 48 | 8 | | Store (current) bitmap offset, which is relative to the start of the volume |
| 56 | 8 | | NTFS (metadata) file reference |
| 64 | 8 | | Unknown (Allocated size) |
| 72 | 8 | | Store previous bitmap offset, which is relative to the start of the volume or contains 0 if not used |
| 80 | 8 | | Unknown (store index?) |
| 88 | 40 | | Unknown (empty) |

## Store

The store contains information about the shadow volume; it actually contains copies of previous
versions of data blocks on the volume.

The stores must be applied starting with the most recent on top of the current volume. E.g. if
there are 3 stores and we want to access the state of the oldest (number 1) we must first apply the
changes in store 3 over the current volume, the changes in store 2 over the resulting volume, and
finally the changes in store 1 over the resulting volume.

The store consists of:

* store header
* store block list
* store block range list
* store bitmaps
* data blocks

### Store block header

The store block header is 128 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | volsnap identifier, which contains a GUID |
| 16 | 4 | 0x01 | Version |
| 20 | 4 | | Record type |
| 24 | 8 | | Relative (block) offset, which is relative to the start of the store |
| 32 | 8 | | Current (block) offset, which is relative to the start of the volume |
| 40 | 8 | | Next (block) offset, which is relative to the start of the volume or contains 0 if this is the last block |
| 48 | 8 | | Size of store information, whichis only used in first block header and should be 0 in successive block headers |
| 56 | 72 | | Unknown (empty value) |

#### Store block record types

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | | Unknown |
| 0x0001 | | Volume header |
| 0x0002 | | Catalog block header |
| 0x0003 | | Block descriptor list (Diff area table) |
| 0x0004 | | Store header |
| 0x0005 | | Unknown (Store block ranges list) |
| 0x0006 | | Store bitmap |

### Store information

The store information is stored directly after the store header.

The store information is variable of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Unknown (identifier?), which contains a GUID |
| 16 | 16 | | Shadow copy identifier, which contains a GUID |
| 32 | 16 | | Shadow copy set identifier, which contains a GUID |
| 48 | 4 | | [Snapshot context](#store_snapshot_context) |
| 52 | 4 | | Unknown (Provider?) |
| 56 | 4 | | [Attribute flags](#store_attribute_flags) |
| 60 | 4 | | Unknown (empty values) |
| 64 | 2 | | Operating machine string size, in number of bytes |
| 66 | (size) | | Operating machine string, which contains an UCS-2 little-endian string without end-of-string character |
| ... | 2 | | Service machine string size, in number of bytes |
| ... | (size) | | Service machine string, which contains an UCS-2 little-endian string without end-of-string character |
| ... | ... | | Unknown (empty value) |

> Note that the difference between the operating machine and the service machine is currently
> unknown.

#### Store snapshot context {#store_snapshot_context}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000000 | VSS_CTX_BACKUP (or Backup) | Standard backup context |
| | | |
| 0x00000009 | VSS_CTX_APP_ROLLBACK (or ApplicationRollback) | Persistent shadow copy |
| | | |
| 0x0000000d | VSS_CTX_CLIENT_ACCESSIBLE_WRITERS (or ClientAccessibleWriters) | Read-only shadow copy created with writer involvement |
| | | |
| 0x00000010 | VSS_CTX_FILE_SHARE_BACKUP | Non-persistent shadow copy created |
| | | |
| 0x00000019 | VSS_CTX_NAS_ROLLBACK | Persistent shadow copy of a NAS volume |
| | | |
| 0x0000001d | VSS_CTX_CLIENT_ACCESSIBLE | Read-only shadow copy for Shared Folders |
| | | |
| 0xffffffff | VSS_CTX_ALL | All types of shadow copy are available for administrative operations |

> Note that the store snapshot context value is a combination of (some of the) store attribute
> flags.

#### Store attribute flags {#store_attribute_flags}

"VSS_VOLUME_SNAPSHOT_ATTRIBUTES enumeration (vss.h)" refers to the store attribute flags as
\_VSS_VOLUME_SNAPSHOT_ATTRIBUTES.

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | VSS_VOLSNAP_ATTR_PERSISTENT | The shadow copy is persistent across reboots |
| 0x00000002 | VSS_VOLSNAP_ATTR_NO_AUTORECOVERY (or VSS_VOLSNAP_ATTR_READ_WRITE) | Auto-recovery is disabled, which appears to  not be shown by vssadmin |
| 0x00000004 | VSS_VOLSNAP_ATTR_CLIENT_ACCESSIBLE | The specified shadow copy is a client-accessible |
| 0x00000008 | VSS_VOLSNAP_ATTR_NO_AUTO_RELEASE | The shadow copy is not automatically deleted when the shadow copy requester process ends (no auto release) |
| 0x00000010 | VSS_VOLSNAP_ATTR_NO_WRITERS | No writers are involved in creating the shadow copy |
| 0x00000020 | VSS_VOLSNAP_ATTR_TRANSPORTABLE | The shadow copy is to be transported and therefore should not be exposed locally |
| 0x00000040 | VSS_VOLSNAP_ATTR_NOT_SURFACED | The shadow copy is not currently exposed (surfaced) |
| 0x00000080 | VSS_VOLSNAP_ATTR_NOT_TRANSACTED | Not transacted, which appears to not be shown by vssadmin |
| | | |
| 0x00010000 | VSS_VOLSNAP_ATTR_HARDWARE_ASSISTED | Indicates that a given provider is a hardware provider |
| 0x00020000 | VSS_VOLSNAP_ATTR_DIFFERENTIAL | Indicates that a given provider uses differential data or a copy-on-write mechanism to implement shadow copies |
| 0x00040000 | VSS_VOLSNAP_ATTR_PLEX | Indicates that a given provider uses a PLEX or mirrored split mechanism to implement shadow copies |
| 0x00080000 | VSS_VOLSNAP_ATTR_IMPORTED | The shadow copy of the volume was imported onto this machine |
| 0x00100000 | VSS_VOLSNAP_ATTR_EXPOSED_LOCALLY | The shadow copy is locally exposed |
| 0x00200000 | VSS_VOLSNAP_ATTR_EXPOSED_REMOTELY | The shadow copy is remotely exposed |
| 0x00400000 | VSS_VOLSNAP_ATTR_AUTORECOVER | Indicates that the writer will need to auto-recover the on post snapshot |
| 0x00800000 | VSS_VOLSNAP_ATTR_ROLLBACK_RECOVERY | Indicates that the writer will need to auto-recover the on post snapshot if the snapshot is used for rollback |
| 0x01000000 | VSS_VOLSNAP_ATTR_DELAYED_POSTSNAPSHOT | Delayed post snapshot, which is reserved for system use and appears to not be shown by vssadmin |
| 0x02000000 | VSS_VOLSNAP_ATTR_TXF_RECOVERY | Indicates that Transactional NTFS (TxF) recovery should be enforced during shadow copy creation, which appears to be not shown by vssadmin |

### Store block list

The store block list contains information about the data block ranges used by the snapshot.

The store block list is stored in blocks of 16384 (0x4000) bytes. Each store block list block
consists of:

* a store block header of type 3
* an array of store block descriptors

#### Block descriptor

The block descriptor is 32 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Original data block offset, which is relative to the start of the volume |
| 8 | 8 | | Relative store data block offset, which is relative to the start of the store. TODO: determine if the lower bits are used for different purpose |
| 16 | 8 | | Store data block offset, which is relative to the start of the volume |
| 24 | 4 | | Flags |
| 28 | 4 | | Allocation bitmap, which is used if flag 0x02 is set, otherwise is should contain a value of 0 |

#### Store block descriptor flags

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | | Is forwarder. The absolute offset is set to 0 and the relative offset maps to the original offset of the next block |
| 0x00000002 | | Overlay. The block descriptor is an overlay. The allocation bitmap value contains information about the block fill |
| 0x00000004 | | Not used. If set, the block is ignored |
| 0x00000008 | | Unknown |
| 0x00000010 | | Unknown |
| 0x00000020 | | Unknown |
| 0x00000040 | | Unknown |
| 0x00000080 | | Unknown |

#### Successive block descriptors

> Note that this section is not complete yet, since the meaning of several flags is unknown.

Successive block descriptors with the same original offset are be handled differently based on
their flags and position in the block list. The block list is scanned front to back.

For the new block descriptor:

```text
* If the not used flag is set (0x04):
    * Ignore the new block descriptor

* If the overlay flag (0x02) is not set:
    * If there is a corresponding block descriptor in the reverse block list:
      Meaning that the original offset (of the new block descriptor) matches
      the relative offset of a forwarder block descriptor in the reverse block
      list.
        * Replace the original offset with that of the forwarder block
          descriptor in the reverse block list.
        * Remove the forwarder block descriptor from the reverse block list.
        * If the forwarder flag (0x01) (of the new block descriptor) is set:
            * If the original offset (of the new block descriptor) is the same
              as the relative offset:
                * Ignore the new block descriptor

* If no previous block descriptor was found:
    * Add the new block descriptor to the block list.
* Else:
    * If the overlay flag (0x02) is set:
      The new block descriptor contains an overlay. The allocation bitmap
      contains information about which part of the block is used. Every bit
      in the allocation bitmap signifies a block of 512 bytes. The LSB in
      the allocation bitmap represent the first 512 bytes in the block.
      Normally the relative offset is should not be 1, but this seems to be
      ignored if it is.

        * If an existing overlay block descriptor was defined:
            * Extended the existing overlay.
              Normally the relative offset should be 1 and the original offset
              should match that of the existing overlay block descriptor. If
              not these values seem to be ignored and the existing overlay
              is extended with the allocation bitmap in the new block descriptor.
        * Else:
            * Replace the existing block descriptor. Existing overlay block
              descriptors are applied to the new block descriptor.

* If the forwarder flag (0x01) is set:
    * If no previous reverse block descriptor was found:
        * Add the new block descriptor to the reverse block list.
    * Else:
        * Replace the existing reverse block descriptor.
```

### Store block range list

The store block range list contains information about the data block ranges used by the store
itself. It is probably used to maintain these ranges on the volume layer, since the corresponding
NTFS file entry data runs are applied on the file system layer.

The store block range list is stored in blocks of 16384 (0x4000) bytes. Each store block range list
block consists of:

* a store block header of type 5
* an array of store block range list entries

#### Store block range entry

The store block range entry is 24 bytes of size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Store (block range start) offset, which is relative to the start of the volume |
| 8 | 8 | | Relative (block range start) offset, which is relative to the start of the store |
| 16 | 8 | | Block range size |

### Store bitmap

The store bitmap contains information about the allocation of blocks.

The store bitmap is stored in blocks of 16384 (0x4000) bytes. Each store bitmap block consists of:

* a store block header of type 6
* a bitmap

#### Store (current) bitmap data

Every bit in the store (current) bitmap represents a block of 16384 (0x4000) bytes, where the LSB
is the first bit in a byte.

If a bit is set, the corresponding block is considered not in-use (or not allocated) by the store.

The use of this bitmap is described in the section: [reading snapshot data](#reading_snapshot_data).

#### Store previous bitmap data

Every bit in the store previous bitmap represents a block of 16384 (0x4000) bytes, where the LSB is
the first bit in a byte.

If a bit is set, the corresponding block is not in-use (or not allocated) by the previous store.

Note that the first store can also contain a previous bitmap if an older store before it was
removed.

The use of this bitmap is described in the section: [reading snapshot data](#reading_snapshot_data).

### Store data block

The store data is stored in blocks of 16384 (0x4000) bytes.

### Reading snapshot data {#reading_snapshot_data}

For the size of the data that will fit in the buffer:

```text
* If the block offset has a corresponding block descriptor:
    * The data is defined by block descriptor and has a maximum size accordingly
    * If this is the active store and the block has an overlay:
        * If the overlay applies:
            * use the overlay block descriptor

    * If the forwarder flag (0x01) is set
      and there is a next store:
        * read the block from the next store using the relative store offset
    * Else:
        * read the block from the current volume using the store offset

* Else:
    * If there is a next store:
        * read the block from the next store
    * Else if the block offset has a corresponding reverse block descriptor:
        * read the block from the current volume
    * Else if the active store is the most recent (last) store
      and the block is flagged in the current bitmap
      and ( the store has no previous bitmap
            or the block is flagged in the previous bitmap ):
        * zero-fill the block
    * Else:
        * read the block from the current volume

    * Increment the block offset with the size of the block data that was read
```

> Note that on Windows the actual behavior of unused block is undefined. A read of a corresponding
> block will return successful but will not alter the buffer passed to the read. For sanitation
> purposes Keramics will zero-fill the block.

## Format edge cases and corruption scenarios

This chapter contains several corruption scenarios that have been encountered "in the wild".

### Catalog volume size out of bounds

> Note that this currently considered a corruption scenario future findings may or may not prove
> otherwise.

The volume size of one of the catalog entries exceeds the size of the underlying volume and does
not corresponds with the volume size defined by the rest of the catalog entries.

### Scope snapshots

Technically scoped snaphots are a feature of volsnapa as of of Windows 8 or Windws Server 2012 and
not a corruption scenario. It has been captured as a corruption scenario since it leads to some
interesting side effects within file content of the snapshot.

Scope snapshots functionality can be controlled via the Windows Registry value:

```text
Key path: HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore
Value name: ScopeSnapshots
```

Per "Scoped Snapshots vmicvss Error 13 on Server 2012, Windows 8" a scope snapshot is a special
volume snapshot for volsnap performance, which is mainly used by Windows critical updates. Scope
means the volsnap only creates Copy on Write (Shadow) volume for the files that are involved in the
updates instead of all the files on the volume.

## References

* [Shadow Copies and Shadow Copy Sets](https://learn.microsoft.com/en-us/windows/win32/vss/shadow-copies-and-shadow-copy-sets)
* [Scoped Snapshots vmicvss Error 13 on Server 2012, Windows 8](https://www.mcbsys.com/blog/2013/05/scoped-snapshots-vmicvss-error-13-on-server-2012-windows-8/),
  by M. Berry
* [VSS_VOLUME_SNAPSHOT_ATTRIBUTES enumeration (vss.h)](https://learn.microsoft.com/en-us/windows/win32/api/vss/ne-vss-vss_volume_snapshot_attributes)
