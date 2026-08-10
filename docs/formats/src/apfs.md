# Apple File System (APFS)

The Apple File System (APFS) is a volume and file system mainly used on platforms such as Mac OS and
iOS. APFS supersedes the [Hierarchical File System (HFS)](hfs.md) and was introduced in macOS High
Sierra (10.13) and iOS 10.3.

## Overview

APFS consists of:

* A container
  * Zero or more logical volumes that contain a file system

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Date and time values | number of nanoseconds since January 1, 1970 00:00:00 UTC (POSIX epoch), disregarding leap seconds |
| Character strings | Unicode strings are stored in UTF-8 |

<!-- rumdl-disable MD028 -->

> Note that date and values are signed integers to represent dates before January 1, 1970. Other
> sources are known to claim the date and time values are unsigned including Apple's own Apple File
> System Reference documentation.

> Note that (some) sources claim that APFS uses Unicode version 9.0. Support for codepoints of more
> recent Unicode versions has been observed.

<!-- rumdl-enable MD028 -->

### Terminology

| Term | Description |
| --- | --- |
| Physical volume | A volume in which the APFS container is stored |
| Logical volume | A volume in which an APFS file system is stored |

## Keys

To encrypt storage media APFS uses different kind of keys.

### Volume master key

The Volume Master Key (VMK) is used to encrypt the data of a specific volume.

### Volume key

For every volume on an Mac OS system with APFS, APFS provides for a volume password to unlock the
encrypted data. The volume password is used to determine a volume key.

## Encryption methods

APFS uses the AES-XTS encryption method to encrypt the key bag, file system metadata and content
data.

### AES-XTS

The AES-XTS encryption method uses:

* a primary key (key 1) to encrypt/decrypt the data (the whitened plaintext/ciphertext).
* a secondary key (key 2) to encrypt/ decrypt the tweak value, also referred to as the tweak key.
  The encrypted tweak value is used to whiten the plaintext/ciphertext.
* a tweak value

The cipher block size is 128 bytes.

The container key bag is encrypted using the "container identifier" of the container as both the
primary and tweak key. The sector number, relative to the start of the container, is used as the
tweak value.

> Note that when a T2 chip is present, it is currently assumed that the T2 is used to encrypted
> the container key bag instead of the "container identifier".

The unit size is the sector size, which is assumed to be 512 bytes also for 4 KiB sector media.

The volume key bag is encrypted using the "volume identifier" of the corresponding key bag entry,
as both the primary and tweak key. The sector number, relative to the start of the container, is
used as the tweak value.

The file system B-tree is encrypted using the volume master key and the sector number, relative to
the start of the container, is used as the tweak value.

### Key bag entries {#key_bag_entries}

## Objects

APFS uses the "object" data type to distinguish between different data types.

### Object header {#object_header}

The object header (obj_phys_t) is 32 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | [Object checksum](#object_checksum) (o_cksum) |
| 8 | 8 | | Object identifier (o_oid) |
| 16 | 8 | | Object transaction identifier (o_xid), which contains the identifier of the most recent transaction that this object was modified in |
| 24 | 4 | | [Object type](#object_types) (o_type) |
| 28 | 4 | | [Object subtype](#object_subtypes) (o_subtype) |

### Object checksum {#object_checksum}

The checksum algorithm:

```text
Fletcher-64 checksum of the data without the object checksum value and an initial value of 0
checksum_lower_32bit = 0xffffffff - ((fletcher_lower_32bit + fletcher_upper_32bit) mod 0xffffffff)
checksum_upper_32bit = 0xffffffff - ((fletcher_lower_32bit + checksum_lower_32bit) mod 0xffffffff)
checksum = (checksum_upper_32bit << 32) | checksum_lower_32bit
```

### Object identifiers

* For a physical object, its identifier is the logical block address on disk where the object is
  stored.
* For an ephemeral object, its identifier is a number.
* For a virtual object, its identifier is a number.

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | OID_INVALID | Invalid |
| 1 | OID_NX_SUPERBLOCK | Container superblock |
| | | |
| 1024 | OID_RESERVED_COUNT | Number of reserved object identifiers |

### Object types {#object_types}

The object type (o_type) value consists of a type and flags.

<!-- rumdl-disable MD033 MD056 -->

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000000 | OBJECT_TYPE_INVALID | Invalid. For a subtype this value represents not set or not specified |
| 0x00000001 | OBJECT_TYPE_NX_SUPERBLOCK | [Container superblock](#container_superblock) |
| 0x00000002 | OBJECT_TYPE_BTREE | B-Tree (root) |
| 0x00000003 | OBJECT_TYPE_BTREE_NODE | B-Tree node |
| 0x00000004 | | Unknown (MTree?) |
| 0x00000005 | OBJECT_TYPE_SPACEMAN | Space manager header |
| 0x00000006 | OBJECT_TYPE_SPACEMAN_CAB | Space manager [chunk information address block](#chunk_information_address_block) |
| 0x00000007 | OBJECT_TYPE_SPACEMAN_CIB | Space manager [chunk information block](#chunk_information_block) |
| 0x00000008 | OBJECT_TYPE_SPACEMAN_BITMAP | Space manager bitmap |
| 0x00000009 | OBJECT_TYPE_SPACEMAN_FREE_QUEUE | Space manager free queue |
| 0x0000000a | OBJECT_TYPE_EXTENT_LIST_TREE | Extent list tree |
| 0x0000000b | OBJECT_TYPE_OMAP | [Object map](#object_map) |
| 0x0000000c | OBJECT_TYPE_CHECKPOINT_MAP | Checkpoint map |
| 0x0000000d | OBJECT_TYPE_FS | [Volume (or file system) superblock](#volume_superblock) |
| 0x0000000e | OBJECT_TYPE_FS | [File system tree](#file_system) |
| 0x0000000f | OBJECT_TYPE_BLOCKREFTREE | [Extent-reference tree](#extent_reference_tree) |
| 0x00000010 | OBJECT_TYPE_SNAPMETATREE | [Snapshot metadata tree](#snapshot_metadata_tree) |
| 0x00000011 | OBJECT_TYPE_NX_REAPER | [Reaper](#reaper) |
| 0x00000012 | OBJECT_TYPE_NX_REAP_LIST | [Reaper list](#reaper_list) |
| 0x00000013 | OBJECT_TYPE_OMAP_SNAPSHOT | Object map snapshot |
| 0x00000014 | OBJECT_TYPE_EFI_JUMPSTART | [EFI jumpstart](#efi_jumpstart) |
| 0x00000015 | OBJECT_TYPE_FUSION_MIDDLE_TREE | [Fusion middle tree](#fusion_middle_tree) |
| 0x00000016 | OBJECT_TYPE_NX_FUSION_WBC | Fusion write-back cache |
| 0x00000017 | OBJECT_TYPE_NX_FUSION_WBC_LIST | Fusion write-back cache list |
| 0x00000018 | OBJECT_TYPE_ER_STATE | Unknown (ER state?) |
| 0x00000019 | OBJECT_TYPE_GBITMAP | Unknown (G Bitmap?) |
| 0x0000001a | OBJECT_TYPE_GBITMAP_TREE | Unknown (G Bitmap tree?) |
| 0x0000001b | OBJECT_TYPE_GBITMAP_BLOCK | Unknown (G Bitmap block?) |
| | | |
| 0x000000ff | OBJECT_TYPE_TEST | Unknown (test?) |
| | | |
| 0x0000ffff | OBJECT_TYPE_MASK | Object type bitmask |
| | | |
| <td colspan="3">*Flags used in combination with some of the object types*</td> |
| 0x08000000 | OBJ_NONPERSISTENT | Unknown (Non-persistent?) |
| 0x10000000 | OBJ_ENCRYPTED | Is encrypted |
| 0x20000000 | OBJ_NOHEADER | Has no object (obj_phys_t) header |
| | | |
| 0x00000000 | OBJ_VIRTUAL | Is virtual object |
| 0x40000000 | OBJ_PHYSICAL | Is physical object |
| 0x80000000 | OBJ_EPHEMERAL | Is ephemeral object |
| | | |
| 0xffff0000 | OBJECT_TYPE_FLAGS_MASK | Object type flags bitmask |
| 0xc0000000 | OBJ_STORAGETYPE_MASK | Object storage type bitmask |
| 0xf8000000 | OBJECT_TYPE_FLAGS_DEFINED_MASK | Unknown |
| | | |
| <td colspan="3">*Object types without flags*</td> |
| 0x6b657973 ("syek") | | Container key bag |
| 0x72656373 ("scer") | | Volume key bag |

<!-- rumdl-enable MD033 MD056 -->

### Object subtypes {#object_subtypes}

The object subtype is used by specific object types such as:

* B-Tree root
* B-Tree node

The object subtypes are the same as the [Object types](#object_types).

## B-tree {#btree}

A B-tree consists of:

* B-tree root object
  * Zero or more B-tree node objects

### B-tree root or node object

A B-tree root or node (or object) consists of:

* Object header
* B-tree node header
* B-tree entries (table of contents)
* keys data, where the first key is stored after the entries in increasing order
* Optional key free list
* unused data
* Optional value free list
* values data, where the first value is stored before the footer in descending order
* Optional B-tree footer, which is only stored in the root node

> Note that the Apple File System Reference documentation combines the B-Tree object and B-tree
> node header into a single structure referred to as btree_node_phys_t.

#### B-tree root object header

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header (btn_o)*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x00000002 or 0x40000002 | [Object type](#object_types) |
| 28 | 4 | | [Object subtype](#object_subtypes) |

<!-- rumdl-enable MD033 MD056 -->

> Note that object type can be 0x00000000 if the B-tree is empty.

#### B-tree node object header

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header (btn_o)*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x00000003 or 0x40000003 | [Object type](#object_types) |
| 28 | 4 | | [Object subtype](#object_subtypes) |

<!-- rumdl-enable MD033 MD056 -->

### B-tree node header

The B-tree node header is stored after the B-tree root or node object.

The B-tree node header is 24 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | [Flags](#btree_node_flags) (btn_flags) |
| 2 | 2 | | Level (btn_level) |
| 4 | 4 | | Number of keys in the node (btn_nkeys) |
| <td colspan="4">*Table space (btn_table_space)*</td> |
| 8 | 2 | | Entries data offset, which contains an offset relative to the end of the B-tree node header or -1 (0xffff) if not set (invalid) |
| 10 | 2 | | Entries data size |
| <td colspan="4">*Free space (btn_free_space)*</td> |
| 12 | 2 | | Unused data offset, which contains an offset relative to the end of the entries data or -1 (0xffff) if not set (invalid) |
| 14 | 2 | | Unused data size |
| <td colspan="4">*Key free list (btn_key_free_list)*</td> |
| 16 | 2 | | Unused key list offset, which contains an offset relative to unknown or -1 (0xffff) if not set (invalid) |
| 18 | 2 | | Unused key list size |
| <td colspan="4">*Value free list (btn_val_free_list)*</td> |
| 20 | 2 | | Unused value list offset, which contains an offset relative to unknown or -1 (0xffff) if not set (invalid) |
| 22 | 2 | | Unused value list size |

<!-- rumdl-enable MD033 MD056 -->

#### B-tree node flags {#btree_node_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | BTNODE_ROOT | Is root |
| 0x0002 | BTNODE_LEAF | Is leaf |
| 0x0004 | BTNODE_FIXED_KV_SIZE | Has a fixed-size entry (key and value) |
| 0x0008 | BTNODE_HASHED | B-tree branch nodes contain a hash of their sub nodes |
| 0x0010 | BTNODE_NOHEADER | The B-tree node are stored without [object header](#object_header), where the object header is filled with 0-byte values |
| | | |
| 0x8000 | BTNODE_CHECK_KOFF_INVAL | In transient state, which is used for in-memory purposes only |

### B-tree entries

The B-tree entries are stored after the B-tree node header.

#### Fixed-size B-tree entry

The fixed-size B-tree entry is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Key data offset (key_offs), which contains an offset relative to the end of the entries data |
| 2 | 2 | | Value data offset (value_offs), which contains a reversed offset relative to the start of the B-Tree footer |

#### Variable-size B-tree entry

The variable-size B-tree entry is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Key data offset (key_offs), which contains an offset relative to the end of the entries data |
| 2 | 2 | | Key data size (key_len) |
| 4 | 2 | | Value data offset (value_offs), which contains a reversed offset relative to the start of the B-Tree footer |
| 6 | 2 | | Value data size (value_len) |

### B-tree footer

The B-tree footer is stored at the end of the block that contains the B-tree root object.

The B-tree footer (btree_info_t) is 40 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Static information (btree_info_fixed_t)*</td> |
| 0 | 4 | | [Flags](#btree_flags) (bt_flags) |
| 4 | 4 | | Node size (bt_node_size) |
| 8 | 4 | | Key size (bt_key_size), which is set to 0 if key has a variable size |
| 12 | 4 | | Value size (bt_val_size), which is set to 0 if value has a variable size |
| <td colspan="4">&nbsp;</td> |
| 16 | 4 | | Maximum key size (bt_longest_key) |
| 20 | 4 | | Maximum value size (bt_longest_val) |
| 24 | 8 | | Total number of keys (bt_key_count) |
| 32 | 8 | | Total number of nodes (bt_node_count) |

<!-- rumdl-enable MD033 MD056 -->

#### B-tree flags {#btree_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | BTREE_UINT64_KEYS | Unknown |
| 0x00000002 | BTREE_SEQUENTIAL_INSERT | Unknown |
| 0x00000004 | BTREE_ALLOW_GHOSTS | Unknown |
| 0x00000008 | BTREE_EPHEMERAL | Unknown |
| 0x00000010 | BTREE_PHYSICAL | Unknown |
| 0x00000020 | BTREE_NONPERSISTENT | Unknown |
| 0x00000040 | BTREE_KV_NONALIGNED | Unknown |
| 0x00000080 | BTREE_HASHED | B-tree branch nodes contain a hash of their sub nodes |
| 0x00000100 | BTREE_NOHEADER | The B-tree node are stored without [object header](#object_header), where the object header is filled with 0-byte values |

## The container

APFS stores volumes inside a container. The maximum number of volumes is dependent on the size of
the container.

| Container size | Maximum number of volumes |
| --- | --- |
| 1 GiB | 2 |
| 2 GiB | 4 |
| 5 GiB | 10 |
| 10 GiB | 20 |
| 20 GiB | 40 |
| 100 GiB | 100 |
| 12 TiB | 100 |
| 1.2 PiB | 100 |
| 7.5 EiB | 100 |

The container consists of:

* current container superblock
* stored in the container checkpoint descriptor area:
  * current checkpoint map
  * previous checkpoint map(s)
  * previous container superblock(s)
* stored in the container:
  * space manager
  * container object map
  * reaper
  * crypto key
  * zero or more volumes
* Unknown: backup of current container superblock?

### Container superblock {#container_superblock}

The container superblock (nx_superblock_t) is 4096 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x80000001 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | "NXSB" | Signature (nx_magix) |
| 36 | 4 | | Block size (nx_block_size) |
| 40 | 8 | | Number of blocks (nx_block_count) |
| 48 | 8 | | [Container feature flags](#container_feature_flags) (nx_features) |
| 56 | 8 | | [Read-only compatible feature flags](#container_read_only_compatible_feature_flags) (nx_readonly_compatible_features) |
| 64 | 8 | | [Incompatible feature flags](#container_incompatible_feature_flags) (nx_incompatible_features) |
| 72 | 16 | | Container identifier (nx_uuid), which contains a big-endian UUID |
| 88 | 8 | | Next (available) object identifier (nx_next_oid) |
| 96 | 8 | | Next (available) transaction identifier (nx_next_xid) |
| 104 | 4 | | Checkpoint descriptor area number of blocks (nx_xp_desc_blocks), which contains the size of the checkpoint descriptor area and the MSB is a flag |
| 108 | 4 | | Checkpoint data area number of blocks (nx_xp_data_blocks), which contains the size of the checkpoint data area and the MSB is a flag |
| 112 | 8 | | Checkpoint descriptor area block number (nx_xp_desc_base), where the block number is relative to the start of the container of the checkpoint descriptor area if the MSB of nx_xp_desc_blocks is not set, otherwise the value contains the physical object identifier of a checkpoint descriptor area B-tree |
| 120 | 8 | | Checkpoint data area block number (nx_xp_data_base), where the block number is relative to the start of the container of the checkpoint data area if the MSB of nx_xp_data_blocks is not set |
| 128 | 4 | | Next available index in the checkpoint descriptor area (nx_xp_desc_next) |
| 132 | 4 | | Next available index in the checkpoint data area (nx_xp_data_next) |
| 136 | 4 | | Index of the checkpoint in the checkpoint descriptor area (nx_xp_desc_index) |
| 140 | 4 | | Size of the checkpoint in the checkpoint descriptor area, in number of blocks (nx_xp_desc_len) |
| 144 | 4 | | Index of the checkpoint in the checkpoint data area (nx_xp_data_index) |
| 148 | 4 | | Size of the checkpoint in the checkpoint data area, in number of blocks (nx_xp_data_len) |
| 152 | 8 | | Space manager object identifier (nx_spaceman_oid), where the object identifier can be resolved in the [checkpoint map](#checkpoint_map) |
| 160 | 8 | | Object map block number (nx_omap_oid), where the block number is relative to the start of the container of the [object map](#object_map) |
| 168 | 8 | | Reaper object identifier (nx_reaper_oid), where the object identifier can be resolved in the [checkpoint map](#checkpoint_map) |
| 176 | 4 | | Unknown (reserved for testing) (nx_test_type) |
| 180 | 4 | | Maximum number of volumes (nx_max_file_systems) supported by the container |
| 184 | 100 x 8 = 800 | | Array of volume object identifiers (nx_fs_oid), which can be resolved to a "physical" location using the [object map](#object_map) |
| 984 | 32 x 8 = 256 | | [Container counters](#container_counters) (nx_counters) |
| <td colspan="4">*Reserved (or blocked out) data area (nx_blocked_out_prange)*</td> |
| 1240 | 8 | | Reserved data area block number (nx_blocked_out_base), which contains a block number relative to the start of the container |
| 1248 | 8 | | Reserved data area number of blocks (nx_blocked_out_blocks) |
| <td colspan="4">&nbsp;</td> |
| 1256 | 8 | | Eviction tree (physical) object identifier (nx_evict_mapping_tree_oid) |
| 1264 | 8 | | [Container flags](#container_flags) (nx_flags) |
| 1272 | 8 | | [EFI jumpstart](#efi_jumpstart) (physical) object identifier (nx_efi_jumpstart), which contains a block number relative to the start of the container |
| 1280 | 16 | | Fusion set identifier (nx_fusion_uuid), which contains a big-endian UUID |
| <td colspan="4">*[Container key bag](#key_bag) area (nx_keylocker)*</td> |
| 1296 | 8 | | Container key bag area block number (nx_keybag_base), which contains a block number relative to the start of the container |
| 1304 | 8 | | Container key bag area number of blocks (nx_keybag_blocks) |
| <td colspan="4">&nbsp;</td> |
| 1312 | 4 x 8 = 32 | | Ephemeral information (nx_ephemeral_info) |
| 1344 | 8 | | Unknown (reserved for testing) (nx_test_oid) |
| 1352 | 8 | | [Fusion middle tree](#fusion_middle_tree) block number (nx_fusion_mt_oid), which contains a block number relative to the start of the container |
| 1360 | 8 | | Fusion write-back cache state object identifier (nx_fusion_wbc_oid), where the object identifier can be resolved in the [checkpoint map](#checkpoint_map) |
| <td colspan="4">*Fusion write-back cache area (nx_fusion_wbc)*</td> |
| 1368 | 8 | | Fusion write-back cache area block number (nx_fusion_wbc_base), which contains a block number relative to the start of the container |
| 1376 | 8 | | Fusion write-back cache area number of blocks (nx_fusion_wbc_blocks) |
| <td colspan="4">&nbsp;</td> |
| 1384 | 8 | | Newest version of software that mounted the container (nx_newest_mounted_version) |
| <td colspan="4">*Media key area (nx_mkb_locker)*</td> |
| 1392 | 8 | | Media key area block number, which contains a block number relative to the start of the container |
| 1400 | 8 | | Media key area number of blocks |
| <td colspan="4">&nbsp;</td> |
| 1408 | 2688 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

> Note that NXSB presumably is an abbreviation of NX superblock. At this point it is unclear what
> NX stands for.

#### Container flags {#container_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | NX_RESERVED_1 | Unknown (reserved) |
| 0x00000002 | NX_RESERVED_2 | Unknown (reserved) |
| 0x00000004 | NX_CRYPTO_SW | The encryption is performed in software |

#### Container feature flags {#container_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | NX_FEATURE_DEFRAG | Supports defragmentation |
| 0x0000000000000002 | NX_FEATURE_LCFD | Uses low-capacity Fusion Drive mode |

#### Container read-only compatible feature flags {#container_read_only_compatible_feature_flags}

Current no read-only compatible feature flags are defined.

#### Container incompatible feature flags {#container_incompatible_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | NX_INCOMPAT_VERSION1 | Pre-release version 1 of APFS |
| 0x0000000000000002 | NX_INCOMPAT_VERSION2 | Release version 2 of APFS |
| | | |
| 0x0000000000000100 | NX_INCOMPAT_FUSION | Supports Fusion Drives |

> Note that according to the Apple File System Reference documentation the pre-release version 1
> and release version 2 are incompatble.

#### Container counters {#container_counters}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | NX_CNTR_OBJ_CKSUM_SET| Number of times a checksum has been calculated when wrting to disk |
| 1 | NX_CNTR_OBJ_CKSUM_FAIL| Number of checksum errors when reading from disk |

> Note that the other 30 counters are presumed to be unused at this point.

### Checkpoint map {#checkpoint_map}

The checkpoint map contains a mapping between container metadata object identifiers and their
location in the container.

> Note that multiple successive checkpoint map objects can be used to store a check point map.

#### Checkpoint map object

The checkpoint map object (checkpoint_map_phys_t) is 4096 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x4000000c | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | [Flags](#checkpoint_flags) (cpm_flags) |
| 36 | 4 | | Number of entries (cpm_count) |
| 40 | 101 x 40 = 4040 | | Array of [checkpoint map entries](#checkpoint_map_entry) (cpm_map) |
| 4080 | 16 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

#### Checkpoint flags {#checkpoint_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | CHECKPOINT_MAP_LAST | Last checkpoint map object |

#### Checkpoint map entry {#checkpoint_map_entry}

The checkpoint map entry (checkpoint_mapping_t) is 40 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Object type](#object_types) (cpm_type) |
| 4 | 4 | | [Object subtype](#object_subtypes) (cpm_subtype) |
| 8 | 4 | | Size (cpm_size) in number of bytes |
| 12 | 4 | | Unknown (padding) (cpm_pad) |
| 16 | 8 | | File system object identifier (cpm_fs_oid) |
| 24 | 8 | | (Container) object identifier (cpm_oid) |
| 32 | 8 | | Physical address (cpm_paddr), which contains a block number relative to the start of the container |

<!-- rumdl-enable MD033 MD056 -->

## Object map {#object_map}

The object map contains a mapping between object identifiers and their "physical" location.

The object map consists of:

* object map (object)
* object map B-tree

### Object map object

The object map object (omap_phys_t) is 4096 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x4000000b | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | [Flags](#object_map_flags) (om_flags) |
| 36 | 4 | | Number of snapshots (om_snap_count) |
| 40 | 4 | | Object map B-tree type (om_tree_type) |
| 44 | 4 | | Object map snapshots B-tree type (om_snapshot_tree_type) |
| 48 | 8 | | Object map B-tree (root node) block number (om_tree_oid), which contains a block number relative to the start of the container |
| 56 | 8 | | Object map snapshots B-tree (root node) block number (om_snapshot_tree_oid), which contains a block number relative to the start of the container |
| 64 | 8 | | Most recent snapshot object identifier (om_most_recent_snap) |
| 72 | 8 | | Unknown transaction identifier (om_pending_revert_min) |
| 80 | 8 | | Unknown transaction identifier (om_pending_revert_max) |
| 88 | 4008 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

#### Object map flags {#object_map_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | OMAP_MANUALLY_MANAGED | No snapshot support |
| 0x00000002 | OMAP_ENCRYPTING | Encryption in progress |
| 0x00000004 | OMAP_DECRYPTING | Decryption in progress |
| 0x00000008 | OMAP_KEYROLLING | Re-encryption with new key in progress |
| 0x00000010 | OMAP_CRYPTO_GENERATION | Encryption configuration has changed |

### Object map B-tree

The object map values are stored in a [B-tree](#btree).

#### Object map B-tree key

The object map B-tree key (omap_key_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Key object identifier (ok_oid) |
| 8 | 8 | | Key object transaction identifier (ok_xid) |

#### Object map B-tree branch node value

An object map B-tree node contains branch node values if BTNODE_LEAF is not set. The corresponding
object map B-tree key represents the first key in the branch.

An object map B-tree branch node value is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Sub node block number, which contains a block number relative to the start of the container |

#### Object map value

An object map B-tree node contains object map values if BTNODE_LEAF is set.

The object map value (omap_val_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Value object flags](#object_map_value_flags) (ov_flags) |
| 4 | 4 | | Value object size (ov_size) |
| 8 | 8 | | Value object physical address (ov_paddr), which contains a block number relative to the start of the container |

##### Object map value flags {#object_map_value_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | OMAP_VAL_DELETED | Unknown |
| 0x00000002 | OMAP_VAL_SAVED | Unknown |
| 0x00000004 | OMAP_VAL_ENCRYPTED | Unknown |
| 0x00000008 | OMAP_VAL_NOHEADER | Unknown |
| 0x00000010 | OMAP_VAL_CRYPTO_GENERATION | Unknown |

#### Notes

TODO document omap_snapshot_t
TODO document Object Map Reaper Phases

### Space manager

The space manager (spaceman_phys_t) is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header (sm_o)*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x80000005 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | Block size (sm_block_size) |
| 36 | 4 | | Number of blocks per chunk (sm_blocks_per_chunk) |
| 40 | 4 | | Number of chunks per chunk information block (CIB) (sm_chunks_per_cib) |
| 44 | 4 | | Number of chunk information blocks (CIBs) per chunk information address block (CAB) (sm_cibs_per_cab) |
| <td colspan="4">*Space manager devices (sm_dev)*</td> |
| 48 | 48 | | Main device (SD_MAIN), which contains a [Space manager device](#space_manager_device) |
| | 96 | 48 | Tier2 device (SD_TIER2), which contains a [Space manager device](#space_manager_device) |
| <td colspan="4">&nbsp;</td> |
| 144 | 4 | | [Flags](#space_manager_flags) |
| 148 | 4 | | Unknown (sm_ip_bm_tx_multiplier) |
| 152 | 8 | | Unknown (sm_ip_block_count) |
| 160 | 4 | | Unknown (sm_ip_bm_size_in_blocks) |
| 164 | 4 | | Unknown (sm_ip_bm_block_count) |
| 168 | 8 | | Unknown (sm_ip_bm_base) |
| 176 | 8 | | Unknown (sm_ip_base) |
| 184 | 8 | | Unknown (sm_fs_reserve_block_count) |
| 192 | 8 | | Unknown (sm_fs_reserve_alloc_count) |
| <td colspan="4">*Space manager free queues (sm_fq)*</td> |
| 200 | 40 | | Unknown [space free queue](#space_manager_free_queue) (SFQ_IP) |
| 240 | 40 | | Main [space free queue](#space_manager_free_queue) (SFQ_MAIN) |
| 280 | 40 | | Tier2 [space free queue](#space_manager_free_queue) (SFQ_TIER2) |
| <td colspan="4">&nbsp;</td> |
| 320 | 2 | | Unknown (sm_ip_bm_free_head) |
| 322 | 2 | | Unknown (sm_ip_bm_free_tail) |
| 324 | 4 | | Unknown (sm_ip_bm_xid_offset), which contains an offset in bytes relative to the start of the space manager |
| 328 | 4 | | Unknown (sm_ip_bitmap_offset), which contains an offset in bytes relative to the start of the space manager |
| 332 | 4 | | Unknown (sm_ip_bm_free_next_offset), which contains an offset in bytes relative to the start of the space manager |
| 336 | 4 | 1 | Unknown (sm_version) |
| 340 | 4 | | Unknown (sm_struct_size) |
| <td colspan="4">*Space manager data zone (sm_datazone)*</td> |
| 344 | 8 x 72 | | Main allocation zones |
| 920 | 8 x 72 | | Tier2 allocation zones |
| <td colspan="4">&nbsp;</td> |
| 1492 | ... | | Unknown (data) |

<!-- rumdl-enable MD033 MD056 -->

#### Space manager flags {#space_manager_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | SM_FLAG_VERSIONED | Unknown |

#### Space manager device {#space_manager_device}

A space manager device (spaceman_device_t) is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Number of blocks (sm_block_count) |
| 8 | 8 | | Number of chunks (sm_chunk_count) |
| 16 | 4 | | Number of chunk information blocks (CIBs) (sm_cib_count) |
| 20 | 4 | | Number of chunk information address blocks (CABs) (sm_cab_count) |
| 24 | 8 | | Number of unused blocks (sm_free_count) |
| 32 | 4 | | Unknown (sm_addr_offset), which contains an offset in bytes relative to the start of the space manager |
| 36 | 4 | | Unknown (sm_reserved) |
| 40 | 8 | | Unknown (sm_reserved2) |

#### Space manager free queue {#space_manager_free_queue}

A space manager free queue (spaceman_free_queue_t) is 40 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (sfq_count) |
| 8 | 8 | | Space manager free queue tree object identifier (sfq_tree_oid) |
| 16 | 8 | | Space manager free queue oldest transaction identifier (sfq_oldest_xid) |
| 24 | 2 | | Unknown (sfq_tree_node_limit) |
| 26 | 2 | | Unknown (sfq_pad16) |
| 28 | 4 | | Unknown (sfq_pad32) |
| 32 | 8 | | Unknown (sfq_reserved) |

#### Space manager allocation zone {#space_manager_allocation_zone}

A space manager allocation zone (spaceman_allocation_zone_info_phys_t) is 72 bytes in size and
consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Current allocation [zone boundaries](#space_manager_zone_boundaries) (saz_current_boundaries) |
| 8 | 7 x 8 | | Previous allocation [zone boundaries](#space_manager_zone_boundaries) (saz_previous_boundaries) |
| 64 | 2 | | Unknown (saz_zone_id) |
| 66 | 2 | | Unknown (saz_previous_boundary_index) |
| 68 | 4 | | Unknown (saz_reserved) |

#### Space manager zone_boundaries {#space_manager_zone_boundaries}

A space manager zone boundaries (spaceman_allocation_zone_boundaries_t) is 8 bytes in size and
consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (saz_zone_start) |
| 8 | 8 | | Unknown (saz_zone_end) |

#### Notes

sm_addr_offset points to block number which points to a OBJECT_TYPE_SPACEMAN_CIB block. Probably an
OBJECT_TYPE_SPACEMAN_CAB block when necessary.

### Chunk information address block {#chunk_information_address_block}

The chunk information address block (cib_addr_block_t) is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header (cab_o)*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x40000006 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | Unknown (cab_index) |
| 36 | 4 | | Number of chunk information blocks (CIBs) (cab_cib_count) |
| <td colspan="4">*Chunk information block physical addresses (cab_cib_addr)*</td> |
| 40 | 8 x Number of CIBs | | Physical address of chunk information blocks (CIB) |

<!-- rumdl-enable MD033 MD056 -->

### Chunk information block {#chunk_information_block}

The chunk information block (chunk_info_block_t) is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header (cib_o)*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x40000007 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | Unknown (cib_index) |
| 36 | 4 | | Number of chunk information entries (cib_chunk_info_count) |
| <td colspan="4">*Chunk information entries (cib_chunk_info)*</td> |
| 40 | 8 x Number of entries | | Array of chunk information entries |

<!-- rumdl-enable MD033 MD056 -->

#### Chunk information entry {#chunk_information_entry}

The chunk information entry (chunk_info_t) is 32 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (ci_xid) |
| 8 | 8 | | Unknown (ci_addr) |
| 16 | 4 | | Unknown (ci_block_count) |
| 20 | 4 | | Unknown (ci_free_count) |
| 24 | 8 | | Unknown (ci_bitmap_addr) |

### Reaper {#reaper}

The reaper is of unknown size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x80000011 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 8 | | Unknown |
| | 8 | | Unknown |
| | 8 | | Unknown |
| | 8 | | Unknown |
| | 4 | | Unknown |
| | 4 | | Unknown |
| | 4 | | Unknown |
| | 4 | | Unknown |
| | 8 | | Unknown |
| | 8 | | Unknown |
| | 8 | | Unknown |
| | 4 | | Unknown |
| | 4 | | Unknown |

<!-- rumdl-enable MD033 MD056 -->

#### Reaper list {#reaper_list}

The reaper list entry is of unknown size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x80000012 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | | Unknown |
| 36 | 4 | | Unknown |
| 40 | 4 | | Unknown |
| 44 | 4 | | Unknown (max_record_count) |
| 48 | 4 | | Unknown (record_count) |
| 52 | 4 | | Unknown (first_index) |
| 56 | 4 | | Unknown (last_index) |
| 60 | 4 | | Unknown (free_index) |
| 64 | 100 x ... | | Array of [reaper list entries](#reaper_list_entry) (nrle) |

<!-- rumdl-enable MD033 MD056 -->

#### Reaper list entry {#reaper_list_entry}

The reaper list entry is 40 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Forward link (fwlink) |
| 4 | 4 | | Unknown |
| 8 | 4 | | Type (type) |
| 12 | 4 | | Block size (blksize) |
| 16 | 8 | | Object identifier (oid) |
| 24 | 8 | | Physical address (paddr), which contains a block number relative to the start of the container |
| 32 | 8 | | Object transaction identifier (xid) |

## Key bag {#key_bag}

The key bag consists of:

* Container or volume key bag object
* Key bag header
* Key bag entries

### Container key bag object

The container key bag object contains key data of the container.

The container key bag object is 32 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x6b657973 ("syek") | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |

<!-- rumdl-enable MD033 MD056 -->

### Volume key bag object

The volume key bag object contains key data of a specific volume.

The volume key bag object is 32 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x72656373 ("scer") | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |

<!-- rumdl-enable MD033 MD056 -->

### Key bag header

The key bag header (kb_locker_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 2 | Format version (kl_version) |
| 2 | 2 | | Number of entries (kl_nkeys) |
| 4 | 4 | | Key bag data size (kl_nbytes), which contains the size of the key bag data, this includes the size of key bag header |
| 8 | 8 | | Unknown (padding) |

### Key bag entries {#key_bag_entries}

A key bag entry consists of:

* a key bag entry header
* a key bag entry data
* alignment padding

The key bag entry header specifies the type of the key bag entry data.

The key bag entries are 16-byte aligned.

#### Key bag entry header

The key bag entry header (keybag_entry_t) is 24 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Volume identifer (ke_uuid), which contains a big-endian UUID |
| 16 | 2 | | [Entry type](#key_bag_entry_types) (ke_tag) |
| 18 | 2 | | Entry data size (ke_keylen) |
| 20 | 4 | | Unknown (padding) |

#### Key bag entry types {#key_bag_entry_types}

##### Container key bag entry types

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00 | KB_TAG_UNKNOWN | Unknown |
| 0x01 | KB_TAG_WRAPPING_KEY | Wrapping key |
| 0x02 | KB_TAG_VOLUME_KEY | Volume master key, which contains a [Key encrypted key (KEK) packed object](#key_bag_kek_packed_object) |
| 0x03 | KB_TAG_VOLUME_UNLOCK_RECORDS | Volume [key bag extent](#key_bag_data_extent) |
| 0x04 | KB_TAG_VOLUME_PASSPHRASE_HINT | Passphrase hint |
| | | |
| 0xf8 | KB_TAG_USER_PAYLOAD | Unknown (user payload) |

The volume master key is encryped with a volume key.

##### Volume key bag entry types

| Value | Identifier | Description |
| --- | --- | --- |
| 3 | | Volume key, which contains a [Key encrypted key (KEK) packed object](#key_bag_kek_packed_object) |
| 4 | | Password hint, which contains a string without end-of-string character |

The volume key is encryped with an user key.

#### Key bag packed object {#key_bag_packed_object}

The packed object consist of an object packed value that embeds attribute packed values.

##### Key bag packed value

The key bag packed value is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Value tag (or value type), where the most-significant bit represents a flag |
| 1 | 1 | | Value data size, where the most-significant bit represents a flag |
| ... | ... | | Value data |

A packed value with a tag and size of 0 signifies the end of the packed values.

> Note that the meaning of the value tags differ per packed object type.

##### Key encrypted key (KEK) packed object {#key_bag_kek_packed_object}

The packed object value tag of a key encrypted key is 0x30 and contains the following attribute
value tags:

| Value | Identifier | Description |
| --- | --- | --- |
| 0x80 | | Unknown |
| 0x81 | | HMAC |
| 0x82 | | Unknown (salt?) |
| | | |
| 0xa3 | | [Wrapped Key Encryption Key (KEK) packed object](#key_bag_wrapped_kek_packed_object) |

##### Wrapped Key Encryption Key (KEK) packed object {#key_bag_wrapped_kek_packed_object}

The packed object value tag of a wrapped kek encrypted key is 0xa3 and contains the following
attribute value tags:

| Value | Identifier | Description |
| --- | --- | --- |
| 0x80 | | Unknown |
| 0x81 | | Volume identifer, which contains a big-endian UUID |
| 0x82 | | [Wrapped Key Encryption Key (KEK) metadata](#wrapped_kek_metadata) |
| 0x83 | | Wrapped Key Encryption Key (KEK) data |
| 0x84 | | Number of iterations for the PBKDF2 algorithm |
| 0x85 | | Salt for the PBKDF2 algorithm |

#### Wrapped Key Encryption Key (KEK) metadata {#wrapped_kek_metadata}

The Wrapped Key Encryption Key (KEK) metadata is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Encryption method](#encryption_methods) |
| 4 | 2 | | Unknown |
| 6 | 1 | | Unknown |
| 7 | 1 | | Unknown |

##### Encryption methods {#encryption_methods}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | | Unknown (AES-256) |
| | | |
| 2 | | Unknown (AES-128 FVDE (CoreStorage FileVault) compatible) |
| | | |
| 16 | | Unknown (AES-256), which has been observed in combination with recovery password protected volume key |

#### Key bag data extent {#key_bag_data_extent}

The key bag data extent is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Key bag block number |
| 8 | 8 | | Key bag number of blocks |

## Volume

The volume consists of:

* volume superblock
* volume object map
* ...

> Note that an APFS volume has a corresponding "synthesized" device file though this cannot be
> directly read.

### Volume superblock {#volume_superblock}

The volume superblock (apfs_superblock_t) is 4096 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x0000000d or 0x4000000d (for snapshots) | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | "APSB" | Signature (apfs_magic) |
| 36 | 4 | | File system index (apfs_fs_index) |
| 40 | 8 | | [Volume feature flags](#volume_feature_flags) (apfs_features) |
| 48 | 8 | | [Read-only compatible feature flags](#volume_read_only_compatible_feature_flags) (apfs_readonly_compatible_features) |
| 56 | 8 | | [Incompatible feature flags](#volume_incompatible_feature_flags) (apfs_incompatible_features) |
| 64 | 8 | | (Last) unmount date and time (apfs_unmount_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 72 | 8 | | Number of reserved blocks (apfs_reserve_block_count) |
| 80 | 8 | | Number of quota blocks (apfs_quota_block_count) |
| 88 | 8 | | Number of allocated blocks (apfs_fs_alloc_count) |
| 96 | 20 | | [Encryption state](#encryption_state) (apfs_meta_crypto) |
| 116 | 4 | | File system root tree [object type](#object_types) (apfs_root_tree_type) |
| 120 | 4 | | Extent-reference tree [object type](#object_types) (apfs_extentref_tree_type) |
| 124 | 4 | | Snapshot metadata tree [object type](#object_types) (apfs_snap_meta_tree_type) |
| 128 | 8 | | Object map block number (apfs_omap_oid), which contains a block number relative to the start of the container of the [object_map](#object_map) |
| 136 | 8 | | File system root tree object identifier (apfs_root_tree_oid) |
| 144 | 8 | | [Extent-reference tree](#extent_reference_tree) block number (apfs_extentref_tree_oid) |
| 152 | 8 | | [Snapshot metadata tree](#snapshot_metadata_tree) block number (apfs_snap_meta_tree_oid) |
| 160 | 8 | | Rollback transaction identifier (apfs_revert_to_xid) |
| 168 | 8 | | Rollback (physical) object identifier (apfs_revert_to_sblock_oid) |
| 176 | 8 | | Next (available) file system object identifier (apfs_next_obj_id), where the upper 32-bit can contain 0xffffffff |
| 184 | 8 | | Number of files (apfs_num_files) |
| 192 | 8 | | Number of directories (apfs_num_directories) |
| 200 | 8 | | Number of symbolic links (apfs_num_symlinks) |
| 208 | 8 | | Number of other file system objects (apfs_num_other_fsobjects) |
| 216 | 8 | | Number of snapshots (apfs_num_snapshots) |
| 224 | 8 | | Total number of blocks allocated (apfs_total_blocks_alloced) |
| 232 | 8 | | Total number of blocks freed (apfs_total_blocks_freed) |
| 240 | 16 | | Volume identifier (apfs_vol_uuid), which contains a big-endian UUID |
| 256 | 8 | | Modification date and time (apfs_last_mod_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 264 | 8 | | [Volume flags](#volume_superblock_flags) (apfs_fs_flags) |
| 272 | 48 | | Creation [change information](#change_information) (apfs_formatted_by) |
| 320 | 8 x 48 = 384 | | 8 most recent modification [change information](#change_information) (apfs_modified_by) |
| 704 | 256 | | Volume label (or name) (apfs_volname) |
| 960 | 4 | | Next (available) document identifier (apfs_next_doc_id) |
| 964 | 2 | | [Volume role flags](#volume_role_flags) (apfs_role) |
| 966 | 2 | | Unknown (reserved) |
| 968 | 8 | | Active snapshot transaction identifier (apfs_root_to_xid) |
| 976 | 8 | | Encryption progress state (apfs_er_state_oid) |
| 984 | 8 | | Largest clone object identifier (apfs_cloneinfo_id_epoch) |
| 992 | 8 | | Largest clone transaction identifier (apfs_cloneinfo_xid) |
| 1000 | 8 | | Extended snapsnot metadata (virtual) object identifier (apfs_snap_meta_ext_oid) |
| 1008 | 16 | | Volume group identifier (apfs_volume_group_id), which contains a big-endian UUID |
| 1024 | 8 | | Integrity metadata (virtual) object identifier (apfs_integrity_meta_oid) |
| 1032 | 8 | | Extent tree (virtual) object identifier (apfs_fext_tree_oid) |
| 1040 | 4 | | Extent tree [object type](#object_types) (apfs_fext_tree_type) |
| 1044 | 4 | | Unknown (reserved_type) |
| 1048 | 8 | | Unknown (reserved_oid) |
| 1056 | 80 | | Unknown |
| 1136 | 2960 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

### Encryption state {#encryption_state}

The encryption state (wrapped_meta_crypto_state_t) is 20 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Major format version (major_version) |
| 2 | 2 | | Minor format version (minor_version) |
| 4 | 4 | | [Flags](#encryption_state_flags) (cpflags) |
| 8 | 4 | | Unknown (persistent_class) |
| 12 | 4 | | Unknown (key_os_version) |
| 16 | 2 | | Unknown (key_revision) |
| 18 | 2 | | Unknown (unused) |

#### Encryption state flags {#encryption_state_flags}

TODO: complete this section.

### Change information {#change_information}

The change information (apfs_modified_by_t) is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 32 | | Application (id), which consist of a string with the first 31 characters of the name and version of the application that changed the file system and 0 if not set |
| 32 | 8 | | Change date and time (timestamp), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 40 | 8 | | Change object transaction number (last_xid) or 0 if not set |

#### Volume flags {#volume_superblock_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | APFS_FS_UNENCRYPTED | Volume is unencrypted |
| 0x0000000000000002 | APFS_FS_EFFACEABLE (or APFS_FS_RESERVED_2) | Unknown (Volume supports effaceable storage?) |
| 0x0000000000000004 | APFS_FS_RESERVED_4 | Unknown (reserved) |
| 0x0000000000000008 | APFS_FS_ONEKEY | Volume uses software encryption with a single key (volume master key) |
| 0x0000000000000010 | APFS_FS_SPILLEDOVER | Volume has run out of allocated space on the solid-state drive |
| 0x0000000000000020 | APFS_FS_RUN_SPILLOVER_CLEANER | Volume has spilled over and the spillover cleaner must be run |
| 0x0000000000000040 | APFS_FS_ALWAYS_CHECK_EXTENTREF | Volume extent reference tree must be consulted before overwriting an extent |
| 0x0000000000000080 | APFS_FS_RESERVED_80 | Unknown (reserved) |
| 0x0000000000000080 | APFS_FS_RESERVED_100 | Unknown (reserved) |

#### Volume features flags {#volume_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | APFS_FEATURE_DEFRAG_PRERELEASE | Unknown |
| 0x0000000000000002 | APFS_FEATURE_HARDLINK_MAP_RECORDS | Unknown |
| 0x0000000000000004 | APFS_FEATURE_DEFRAG | Unknown |
| 0x0000000000000008 | APFS_FEATURE_STRICTATIME | Unknown |
| 0x0000000000000010 | APFS_FEATURE_VOLGRP_SYSTEM_INO_SPACE | Unknown |

#### Volume read-only compatible feature flags {#volume_read_only_compatible_feature_flags}

Current no read-only compatible feature flags are defined

#### Volume incompatible feature flags {#volume_incompatible_feature_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | APFS_INCOMPAT_CASE_INSENSITIVE | Filenames are case insensitive |
| 0x0000000000000002 | APFS_INCOMPAT_DATALESS_SNAPS | Volume contains one or more snapshots without data |
| 0x0000000000000004 | APFS_INCOMPAT_ENC_ROLLED | Encryption keys of the volume have been changed |
| 0x0000000000000008 | APFS_INCOMPAT_NORMALIZATION_INSENSITIVE | Filenames are normalization insensitive |
| 0x0000000000000010 | APFS_INCOMPAT_INCOMPLETE_RESTORE | Unknown |
| 0x0000000000000020 | APFS_INCOMPAT_SEALED_VOLUME | Unknown |
| 0x0000000000000040 | APFS_INCOMPAT_RESERVED_40 | Unknown |

#### Volume role flags {#volume_role_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | APFS_VOL_ROLE_NONE | None |
| 0x0001 | APFS_VOL_ROLE_SYSTEM | System |
| 0x0002 | APFS_VOL_ROLE_USER | User |
| 0x0004 | APFS_VOL_ROLE_RECOVERY | Recovery |
| 0x0008 | APFS_VOL_ROLE_VM | VM |
| 0x0010 | APFS_VOL_ROLE_PREBOOT | Preboot |
| 0x0020 | APFS_VOL_ROLE_INSTALLER | Installer |

## File system {#file_system}

The file system structures are stored in a [B-tree](#btree).

The file system B-tree uses identifiers similar to catalog identifiers (CNIDs) on
[Hierarchical File System (HFS)](hfs.md). In this document these identifiers are referred to as
File System object identifiers (FSOIDs) to contrast other object identifiers (OIDs).

| FSOID | Identifier | Assignment |
| --- | --- | --- |
| 0 | | Unknown (Reserved) |
| 1 | | Parent identifier of the root directory (folder), nameless |
| 2 | | Directory identifier of the root directory (folder), named "root" |
| 3 | | Unknown, named "private-dir" |

### File system B-tree key

The file system B-tree key is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object identifier and type (obj_id_and_type)*</td> |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | | [File system data type](#file_system_data_types) |
| 8 | ... | | Optional additional key data dependent on the data type |

<!-- rumdl-enable MD033 MD056 -->

### File system data types {#file_system_data_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0 | APFS_TYPE_ANY | Unknown (Any) |
| 0x1 | APFS_TYPE_SNAP_METADATA | [Snapshot metadata](#snapshot_metadata) |
| 0x2 | APFS_TYPE_EXTENT | [Extent](#extent) |
| 0x3 | APFS_TYPE_INODE | [Inode](#inode) |
| 0x4 | APFS_TYPE_XATTR | [Extended attribute](#extended_attribute) (xattr) |
| 0x5 | APFS_TYPE_SIBLING_LINK | [Sibling link](#sibling_link) |
| 0x6 | APFS_TYPE_DSTREAM_ID | [Data stream identifier](#data_stream_identifier) |
| 0x7 | APFS_TYPE_CRYPTO_STATE | [Encryption state](#encryption_state) |
| 0x8 | APFS_TYPE_FILE_EXTENT | [File extent](#file_extent) |
| 0x9 | APFS_TYPE_DIR_REC | [Directory record](#directory_record) |
| 0xa | APFS_TYPE_DIR_STATS | [Directory stats](#directory_stats) |
| 0xb | APFS_TYPE_SNAP_NAME | [Snapshot name](#snapshot_name) |
| 0xc | APFS_TYPE_SIBLING_MAP | [Sibling map](#sibling_map) |
| | | |
| 0xf | APFS_TYPE_INVALID | Invalid |

### File system B-tree branch node value

A file system B-tree node contains branch node values if BTNODE_LEAF is not set. The corresponding
file system B-tree key represents the first key in the branch.

A file system B-tree branch node value is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | B-tree sub node object identifier, which can be resolved to a "physical" location using the [object map](#object_map) |

### Snapshot metadata {#snapshot_metadata}

The snapshot metadata value (j_snap_metadata_val_t) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Extent-reference tree block number, which contains a block number relative to the start of the container |
| 8 | 8 | | Volume superblock block number, which contains a block number relative to the start of the container |
| 16 | 8 | | Creation time, which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 24 | 8 | | Change (or last modification) time, which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 32 | 8 | | Unknown (inum) |
| 40 | 4 | | Extent-reference tree [object type](#object_types) (extentref_tree_type) |
| 44 | 4 | | [Flags](#snapshot_metadata_flags) |
| 48 | 2 | | Name string size (name_len), which includes the size of the end-of-string character |
| 50 | ... | | Name string (name), which contains an UTF-8 encoded string with an end-of-string character |

#### Snapshot metadata flags {#snapshot_metadata_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | SNAP_META_PENDING_DATALESS | Unknown |

### Extent {#extent}

#### Extent key data

The extent key data (j_phys_ext_key_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x2 | [File system data type](#file_system_data_types) |

#### Extent value data

The extent value data (j_phys_ext_val_t) is 20 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Extent size and data type (len_and_kind)*</td> |
| 0 | 60 bits | | Extent data size |
| 7.4 | 4 bits | | [File system data type](#file_system_data_types) |
| <td colspan="4">&nbsp;</td> |
| 8 | 8 | | File system object identifier of owner (owning_obj_id) |
| 16 | 4 | | Reference count (refcnt) |

<!-- rumdl-enable MD033 MD056 -->

### Inode {#inode}

#### Inode key data

The inode key data (j_inode_key_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x3 | [File system data type](#file_system_data_types) |

#### Inode value data

The inode value data (APFS_TYPE_INVALID) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Parent file system object identifier (parent_id) |
| 8 | 8 | | Data stream file system object identifier (private_id), which contains the file system object identifier of the file extents that make up the data stream |
| 16 | 8 | | Creation date and time (create_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 24 | 8 | | Modification date and time (mod_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 32 | 8 | | Inode change date and time (change_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 48 | 8 | | Access date and time (access_time), which consists of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 56 | 8 | | [Inode flags](#inode_flags) (internal_flags) |
| 64 | 4 | | Number of children (nchildren) or number of (hard) links (nlink) |
| 68 | 4 | | Unknown (default_protection_class) |
| 72 | 4 | | Unknown (write_generation_counter) |
| 76 | 4 | | [BSD file entry flags](#bsd_file_entry_flags) (bsd_flags) |
| 80 | 4 | | Owner user identifier (owner) |
| 84 | 4 | | Group identifier (gid) |
| 86 | 2 | | [File mode](#file_modes) |
| 88 | 2 | | Unknown (pad1) |
| 90 | 8 | | Unknown (pad2) |
| 98 | ... | | [Extended fields](#extended_fields) (xfields) |

> Note that Mac OS stat command treats nchildren equivalent to nlink.

##### Inode flags {#inode_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000000000000001 | INODE_IS_APFS_PRIVATE | The inode is used internally, typically for a data stream |
| 0x0000000000000002 | INODE_MAINTAIN_DIR_STATS | The inode tracks the size of all of its children |
| 0x0000000000000004 | INODE_DIR_STATS_ORIGIN | The inode has the INODE_MAINTAIN_DIR_STATS flag set explicitly, not due to inheritance |
| 0x0000000000000008 | INODE_PROT_CLASS_EXPLICIT | The inode data protection class was set explicitly when the inode was created |
| 0x0000000000000010 | INODE_WAS_CLONED | The inode was created by cloning another inode |
| 0x0000000000000020 | INODE_FLAG_UNUSED | Unknown (Reserved) |
| 0x0000000000000040 | INODE_HAS_SECURITY_EA | The inode has an access control list (security extended attribute) |
| 0x0000000000000080 | INODE_BEING_TRUNCATED | The inode was truncated |
| 0x0000000000000100 | INODE_HAS_FINDER_INFO | The inode has a Finder info extended field |
| 0x0000000000000200 | INODE_IS_SPARSE | The inode has a sparse byte count extended field |
| 0x0000000000000400 | INODE_WAS_EVER_CLONED | The inode has been cloned at least once |
| 0x0000000000000800 | INODE_ACTIVE_FILE_TRIMMED | The inode is an overprovisioning file that has been trimmed |
| 0x0000000000001000 | INODE_PINNED_TO_MAIN | The inode file content is always on the main storage device. This flag is used for Fusion drives where the main storage is a solid-state drive |
| 0x0000000000002000 | INODE_PINNED_TO_TIER2 | The inode file content is always on the secondary storage device. This flag is used for Fusion drives where the secondary storage is a (magnetic) hard drive |
| 0x0000000000004000 | INODE_HAS_RSRC_FORK | The inode has a resource fork |
| 0x0000000000008000 | INODE_NO_RSRC_FORK | The inode does not have a resource fork |
| 0x0000000000010000 | INODE_ALLOCATION_SPILLEDOVER | The inode file content has some space allocated outside of the preferred storage tier for that file |

##### File modes {#file_modes}

| Value | Identifier | Description |
| --- | --- | --- |
| 0xf000 (0170000) | S_IFMT | File type bitmask |
| 0x1000 (0010000) | S_IFIFO | Named pipe |
| 0x2000 (0020000) | S_IFCHR | Character-special file (Character device) |
| 0x4000 (0040000) | S_IFDIR | Directory |
| 0x6000 (0060000) | S_IFBLK | Block-special file (Block device) |
| 0x8000 (0100000) | S_IFREG | Regular file |
| 0xa000 (0120000) | S_IFLNK | Symbolic link |
| 0xc000 (0140000) | S_IFSOCK | Socket |
| 0xe000 (0160000) | S_IFWHT | Whiteout |

A whiteout is a file entry that covers up all entries of a particular name from lower branches.

##### BSD file entry flags {#bsd_file_entry_flags}

The BSD file entry flags are defined in the `<sys/stat.h>` header file.

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000ffff | UF_SETTABLE | bitmask of owner changeable flags |
| | | |
| 0x00000001 | UF_NODUMP | do not dump file entry |
| 0x00000002 | UF_IMMUTABLE | file entry is immutable and may not be changed |
| 0x00000004 | UF_APPEND | writes to file entry may only append |
| 0x00000008 | UF_OPAQUE | directory is opaque wrt. union |
| 0x00000010 | UF_NOUNLINK | file entry may not be removed or renamed, which is not implement in Mac OS |
| 0x00000020 | UF_COMPRESSED | file entry is compressed |
| 0x00000040 | UF_TRACKED | notify about file entry changes |
| 0x00000080 | UF_DATAVAULT | entitlement required for reading and writing |
| | | |
| 0x00008000 | UF_HIDDEN | file entry is hidden |
| | | |
| 0xffff0000 | SF_SETTABLE | bitmask of superuser changeable flags |
| | | |
| 0x001f0000 | SF_SUPPORTED | bitmask of superuser supported flags |
| | | |
| 0x00010000 | SF_ARCHIVED | file entry is archived |
| 0x00020000 | SF_IMMUTABLE | file entry is immutable and may not be changed |
| 0x00040000 | SF_APPEND | writes to file entry may only append |
| 0x00080000 | SF_RESTRICTED | entitlement required for writing |
| 0x00100000 | SF_NOUNLINK | file entry may not be removed, renamed or used as mount point |
| 0x00200000 | SF_SNAPSHOT | snapshot inode, which is not implement in Mac OS |

### Extended attribute {#extended_attribute}

#### Extended attribute key data

The extended attribute key data (j_xattr_key_t) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x4 | [File system data type](#file_system_data_types) |
| 8 | 2 | | Name string size (name_len), which includes the size of the end-of-string character |
| 10 | ... | | [Name string](#extended_attribute_names) (name), which contains an UTF-8 encoded string with an end-of-string character |

> Note that the name of an extended attribute appears to be case senstive even on a case
> insensitive file system.

#### Extended attribute value data

The extended attribute value data (j_xattr_val_t) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | [Flags](#extended_attribute_flags) (flags) |
| 2 | 2 | | Extended attribute data size |
| 4 | ... | | Extended attribute data |

> Note that extended attribute data size can contain 0 if extended attribute flags
> XATTR_DATA_EMBEDDED is set.

#### Extended attribute names {#extended_attribute_names}

| Name | Description |
| --- | --- |
| com.apple.assetsd.dbRebuildInProgress | |
| com.apple.assetsd.dbRebuildUuid | |
| com.apple.assetsd.thumbnailCameraPreviewImageAssetID | |
| com.apple.assetsd.UUID | |
| com.apple.decmpfs | [Apple File System Compression (decmpfs)](decmpfs.md) extended attribute. |
| com.apple.FinderInfo | |
| com.apple.fs.symlink | Symbolic link |
| com.apple.genstore.info | |
| com.apple.genstore.origdisplayname | |
| com.apple.genstore.orig_perms_v1 | |
| com.apple.genstore.origposixname | |
| com.apple.GeoServices.SHA1 | |
| com.apple.installd.installType | |
| com.apple.installd.uniqueInstallID | |
| com.apple.lastuseddate#PS | |
| com.apple.metadata:_kMDItemUserTags | |
| com.apple.metadata:com_apple_backup_excludeItem | |
| com.apple.metadata:kMDItemDownloadedDate | |
| com.apple.metadata:kMDItemWhereFroms | |
| com.apple.metadata:kMDLabel_fwlfb7nbt2o7degof3q2o2btjy | |
| com.apple.quarantine | |
| com.apple.ResourceFork | Resource fork |
| com.apple.rootless | |
| com.apple.system.Security | |
| com.apple.TextEncoding | |
| LastUpgradeCheck | |
| lock | |
| org.chromium.crashpad.database.initialized | |

#### Extended attribute flags {#extended_attribute_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0001 | XATTR_DATA_STREAM | Extended attribute data is stored in a data stream, where the attribute data contains an 8-byte file system object identifier of the corresponding [data stream](#extended_attribute_data_stream) |
| 0x0002 | XATTR_DATA_EMBEDDED | Extended attribute data is stored directly in the record |
| 0x0004 | XATTR_FILE_SYSTEM_OWNED | Extended attribute record is owned by the file system |
| 0x0008 | XATTR_RESERVED_8 | Unknown (Reserved) |

#### Extended attribute data stream {#extended_attribute_data_stream}

The extended attribute data stream (j_xattr_dstream_t) is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Data stream file system object identifier (xattr_obj_id), which contains the file system object identifier of the file extents that make up the data stream |
| 8 | 48 | | [Data stream attribute](#data_stream_attribute) |

### Sibling link {#sibling_link}

#### Sibling link key data

The sibling link key data (j_sibling_key_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x4 | [File system data type](#file_system_data_types) |
| 8 | 8 | | Sibling map identifier (sibling_id), which contains the file system object identifier of the sibling map record |

#### Sibling link value data

The sibling link value data (j_sibling_val_t) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Parent file system object identifier (parent_id) |
| 8 | 2 | | Name string size (name_len), which includes the size of the end-of-string character |
| 10 | ... | | Name string (name), which contains an UTF-8 encoded string with an end-of-string character |

### Data stream identifier {#data_stream_identifier}

#### Data stream identifier key data

The data stream key data (j_dstream_id_key_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x6 | [File system data type](#file_system_data_types) |

#### Data stream identifier value data

The data stream value data (j_dstream_id_val_t) is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Reference count (refcnt) |

### File extent {#file_extent}

#### File extent key data

The file extent key data (j_file_extent_key_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x8 | [File system data type](#file_system_data_types) |
| 8 | 8 | | Logical address (logical_addr), which contains an offset relative to the start of the file entry data |

#### File extent value data

The file extent value data (j_file_extent_val_t) is 24 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Extent data size and flags (len_and_flags)*</td> |
| 0 | 7 | | Extent data size |
| 7 | 1 | | [Flags](#file_extent_flags) |
| 8 | 8 | | Physical block number (phys_block_num), which contains a block number relative to the start of the container |
| 16 | 8 | | Encryption identifier (crypto_id), which contains an unknown value and 0 if not set |

<!-- rumdl-enable MD033 MD056 -->

#### File extent flags {#file_extent_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | | Unknown (Is encrypted?) |

> Note at according to the Apple File System Reference documentation there are currently no flags
> defined. It also refers to `len_and_flags` as `len_and_kind` interchangeably.

### Directory record {#directory_record}

The directory record can have 2 different types of keys:

* Key with name
* Key with name and hash

<!-- rumdl-disable MD028 -->

> Note that apprears that current APFS file system use a key with name and hash. Apple File System
> Reference documentation does not indicate how to distinguish between the two, but one method is
> to compare calculated and stored size of the key data.

> Note that B-Tree branch nodes are sorted using the case-sensitive name, even when the file system
> is case-insensitive.

<!-- rumdl-enable MD028 -->

#### Directory record key data with name

The directory record key data with name (j_drec_key_t) is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object identifier and type (hdr)*</td> |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x9 | [File system data type](#file_system_data_types) |
| <td colspan="4">&nbsp;</td> |
| 8 | 2 | | Name string size (name_len), which includes the size of the end-of-string character |
| 10 | ... | | Name string (name), which contains an UTF-8 encoded string with an end-of-string character |

<!-- rumdl-enable MD033 MD056 -->

#### Directory record key data with name and hash

The directory record key data with name and hash (j_drec_hashed_key_t) is of variable size and
consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object identifier and type (hdr)*</td> |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x9 | [File system data type](#file_system_data_types) |
| <td colspan="4">*Name string size and hash (name_len_and_hash)*</td> |
| 8 | 11 bits | | Name string size, which includes the size of the end-of-string character |
| 9.3 | 21 bits | | [Name hash](#directory_entry_name_hash) |
| <td colspan="4">&nbsp;</td> |
| 12 | ... | | Name string (name), which contains an UTF-8 encoded string with an end-of-string character |

<!-- rumdl-enable MD033 MD056 -->

#### Directory record value data

The directory record value data (j_drec_val_t) is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | File system object identifier of the directory entry (file_id) |
| 8 | 8 | | Date and time the directory entry was added (date_added), which consist of a signed integer that contains the number of nanoseconds since January 1, 1970 00:00:00 UTC or 0 if not set |
| 16 | 2 | | [Directory entry flags](#directory_entry_flags) |
| 18 | ... | | [Extended fields](#extended_fields) (xfields) |

##### Directory entry flags {#directory_entry_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | DT_UNKNOWN | Unknown |
| 0x0001 | DT_FIFO | Named pipe |
| 0x0002 | DT_CHR | Character-special file (Character device) |
| | | |
| 0x0004 | DT_DIR | Directory |
| | | |
| 0x0006 | DT_BLK | Block-special file (Block device) |
| | | |
| 0x0008 | DT_REG | Regular file |
| | | |
| 0x000a | DT_LNK | Symbolic link |
| | | |
| 0x000c | DT_SOCK | Socket |
| | | |
| 0x000e | DT_WHT | Whiteout |
| | | |
| 0x000f | DREC_TYPE_MASK | Directory type bitmask |
| 0x0010 | RESERVED_10 | Unknown (reserved) |

A whiteout is a file entry that covers up all entries of a particular name from lower branches.

##### Directory entry name hash {#directory_entry_name_hash}

The name hash of a directory entry is calculated as following:

* If the file system is case-insensitive represent the name in lower-case
* Represent the name as an Unicode string in Normalization Form Canonical Decomposition (NFD)
* Format the Unicode string as a little-endian UTF-32 stream without a byte-order-mark or
  end-of-string character
* Calculate a CRC-32c checksum of the UTF-32 stream with an initial checksum of 0xffffffff (-1)
* The lower 22-bits of checksum form the hash

The CRC-32 calculation uses the Castagnoli polynomial (0x1edc6f41), also known as CRC-32C (or
CRC32-C). The CRC-32 calculation does not use the XOR with 0xffffffff before and after the
calculation, which is also referred to as weak CRC-32 calculation.

### Directory stats {#directory_stats}

#### Directory stats key data

The directory stats key data (j_dir_stats_key_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0xa | [File system data type](#file_system_data_types) |

#### Directory stats value data

The directory stats value data (j_dir_stats_val_t) is 32 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Number of children (num_children) |
| 8 | 8 | | Total size (total_size) |
| 16 | 8 | | Parent directory file system object identifier (chained_key) |
| 24 | 8 | | Generation count (gen_count) |

### Snapshot name {#snapshot_name}

The snapshot name (j_snap_name_val_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | Snapshot metdata object identifier |
| 7.4 | 4 bits | 0x1 | [File system data type](#file_system_data_types) |

### Sibling map {#sibling_map}

#### Sibling map key data

The sibling map key data (j_sibling_map_key_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 60 bits | | File system object identifier (FSOID) |
| 7.4 | 4 bits | 0x4 | [File system data type](#file_system_data_types) |

#### Sibling map value data

The sibling map value data (j_sibling_map_val_t) is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | File system object identifier (file_id) |

### Extended fields {#extended_fields}

Directory entries and inodes use extended fields to store additional attributes, such as the
filename.

The extended fields (xf_blob_t) consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Number of extended fields (xf_num_exts) |
| 2 | 2 | | Extended field value data size (xf_used_data) |
| <td colspan="4">*Extended field data (xf_data)*</td> |
| 4 | ... | | Array of [extended field descriptors](#extended_field_descriptor) |
| ... | ... | | Extended field value data |

<!-- rumdl-enable MD033 MD056 -->

> Note that extended field values are stored 8-byte aligned in the extended field value data.

#### Extended field descriptor {#extended_field_descriptor}

An extended field descriptor (x_field_t) is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | [Extended field type](#extended_field_types) (x_type) |
| 1 | 1 | | [Extended field flags](#extended_field_flags) (x_flags) |
| 2 | 2 | | Extended field data size (x_size) |

#### Extended field types {#extended_field_types}

##### Directory record extended field types

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | DREC_EXT_TYPE_SIBLING_ID | Hard link sibling identifier, where the extended field data contains a 64-bit integer value |

##### Inode extended field types {#inode_extended_field_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | INO_EXT_TYPE_SNAP_XID | Transaction identifier of a snapshot, where the extended field data contains a 64-bit integer value |
| 2 | INO_EXT_TYPE_DELTA_TREE_OID | Object identifier of the snapshot extent delta list, where the extended field data contains a 64-bit integer value |
| 3 | INO_EXT_TYPE_DOCUMENT_ID | Document identifier, where the extended field data contains a 32-bit integer value |
| 4 | INO_EXT_TYPE_NAME | Filename, where the extended field data contains an UTF-8 string with end-of-string character |
| 5 | INO_EXT_TYPE_PREV_FSIZE | Previous file size, where the extended field data contains a 64-bit integer value |
| 6 | INO_EXT_TYPE_RESERVED_6 | Unknown (Reserved) |
| 7 | INO_EXT_TYPE_FINDER_INFO | Finder information, where the extended field data contains a 32-bit integer value |
| 8 | INO_EXT_TYPE_DSTREAM | Data stream, where the extended field data contains a [data stream attribute](#data_stream_attribute) |
| 9 | INO_EXT_TYPE_RESERVED_9 | Unknown (Reserved) |
| 10 | INO_EXT_TYPE_DIR_STATS_KEY | Directory statistics; it is unknown if the extended field data contains an object identifier of the directory statistics or a j_dir_stats_val_t structure, seen 8 byte value |
| 11 | INO_EXT_TYPE_FS_UUID | Mounted file system identifier, where the extended field data contains a 128-bit UUID value |
| 12 | INO_EXT_TYPE_RESERVED_12 | Unknown (Reserved) |
| 13 | INO_EXT_TYPE_SPARSE_BYTES | Number of sparse bytes in the data stream, where the extended field data contains a 64-bit integer value |
| 14 | INO_EXT_TYPE_RDEV | Block or character [device identifier](#device_identifier), where the extended field data contains a 32-bit integer value |
| 15 | INO_EXT_TYPE_PURGEABLE_FLAGS | Information about a purgeable file; unknown, defined as reserved, seen 8 byte value |
| 16 | INO_EXT_TYPE_ORIG_SYNC_ROOT_ID | Unknown (Inode number of the sync-root hierarchy) |

#### Extended field flags {#extended_field_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x01 | XF_DATA_DEPENDENT | Contents of the extended field is dependent on the data stream (file contents) |
| 0x02 | XF_DO_NOT_COPY | Do not duplicate the extended field when copied |
| 0x04 | XF_RESERVED_4 | Unknown (Reserved) |
| 0x08 | XF_CHILDREN_INHERIT | Newly created sub directory entries (children) inherit the extended field |
| 0x10 | XF_USER_FIELD | Extended field was added by an user-space program |
| 0x20 | XF_SYSTEM_FIELD | Extended field was added by the system (kernel) |
| 0x40 | XF_RESERVED_40 | Unknown (Reserved) |
| 0x80 | XF_RESERVED_80 | Unknown (Reserved) |

#### Device identifier {#device_identifier}

The device identifier can be stored in different formats, such as: native, 386bsd, 4bsd, bsdos,
freebsd, hpux, isc, linux, netbsd, osf1, sco, solaris, sunos, svr3, svr4 and ultrix.

The "native" and "hpux" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | Major device number |
| 1 | 2 | 0 | Unknown |
| 3 | 1 | | Minor device number |

The "386bsd", "4bsd", "freebsd", "isc", "linux", "netbsd", "sco", "sunos", "svr3" and "ultrix"
device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | 0 | Unknown |
| 2 | 1 | | Major device number |
| 3 | 1 | | Minor device number |

The "solaris" and "svr4" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 18 bits | | Minor device number |
| 2.2 | 14 bits | | Major device number |

The "bsdos" and "osf1" device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 20 bits | | Minor device number |
| 2.4 | 12 bits | | Major device number |

The "bsdos" alternative device identifier is 4 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 8 bits | | Sub unit number |
| 1.0 | 12 bits | | Unit number |
| 2.4 | 12 bits | | Major device number |

### Data stream attribute {#data_stream_attribute}

The data stream attribute (j_dstream_t) is 40 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Used size (size) |
| 8 | 8 | | Allocated size (alloced_size) |
| 16 | 8 | | (Default) encryption identifier (default_crypto_id) |
| 24 | 8 | | Total number of bytes written to data stream (total_bytes_written) |
| 32 | 8 | | Total number of bytes read from data stream (total_bytes_written) |

## File content {#file_content}

APFS supports multiple ways to store file content:

* Data fork
* Compressed data extended attribute
* Compressed data extended attribute with resource fork
* Resource fork
* Extended attribute (named fork)

### Data fork

The file content size is stored in an INO_EXT_TYPE_DSTREAM
[inode extended field type](#inode_extended_field_types).

The file content data can be located through the [file extents](#file_extent) for the data stream
file system object identifier in the [file system tree](#file_system).

If the volume is encrypted the file content is encrypted with the encryption identifier in defined
by the [file extent](#file_extent).

If the [inode flag](#inode_flags) INODE_IS_SPARSE is set the file contains one or more spare file
extents. A sparse file extent has a physical block number of 0.

### Compressed data extended attribute

The file content data and size are stored in the compressed data header of a "com.apple.decmpfs"
[extended attribute](#extended_attribute).

Also see: [Apple File System Compression (decmpfs)](decmpfs.md).

### Compressed data extended attribute with resource fork

The file content size is stored in the compressed data header of a "com.apple.decmpfs"
[extended attribute](#extended_attribute).

The file content data is stored in a "com.apple.ResourceFork"
[extended attribute](#extended_attribute).

Also see: [Apple File System Compression (decmpfs)](decmpfs.md).

### Resource fork

TODO: complete this section.

### Extended attribute (named fork)

TODO: complete this section.

## EFI jumpstart {#efi_jumpstart}

The EFI jumpstart (nx_efi_jumpstart_t) is of variable size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x00000014 | [Object type](#object_types) |
| 28 | 4 | 0x00000000 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| 32 | 4 | "RDSJ" | Signature (nej_magic) |
| 36 | 4 | 1 | Format version (nej_version) |
| 40 | 4 | | Unknown (nej_efi_file_len?) |
| 44 | 4 | | Number of extents (nej_num_extents) |
| 48 | 16 x 8 | | Unknown (nej_reserved?) |
| 176 | number of extents x 16 | | [EFI jumpstart extents](#efi_jumpstart_extent) (nej_rec_extents), which contains the location where the EFI driver is stored |

<!-- rumdl-enable MD033 MD056 -->

## EFI jumpstart extent {#efi_jumpstart_extent}

The EFI jumpstart extent (prange_t) is 16 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Block number |
| 8 | 8 | | Number of blocks |

## Extent-reference tree {#extent_reference_tree}

TODO: complete this section.

## Snapshots

TODO: complete this section.

### Snapshot metadata tree {#snapshot_metadata_tree}

The snapshot metadata tree consists of:

* snapshot metadata tree (object)
* snapshot metadata B-tree

### Snapshot metadata tree object

The snapshot metadata tree object is 32 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x40000002 or 0x40000003 | [Object type](#object_types) |
| 28 | 4 | 0x00000010 | [Object subtype](#object_subtypes) |

<!-- rumdl-enable MD033 MD056 -->

### Snapshot metadata B-tree

The object map values are stored in [B-tree](#btree).

#### Snapshot metadata B-tree key

The snapshot metadata B-tree key (j_snap_metadata_key_t or j_snap_name_key_t) is of variable size
and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Key object identifier (hdr) |
| <td colspan="4">*If key object identifier data type is APFS_TYPE_SNAP_NAME*</td> |
| 8 | ... | | Snapshot name string, which contains an UTF-8 encoded string with an end-of-string character |

<!-- rumdl-enable MD033 MD056 -->

#### Snapshot metadata B-tree branch node value

A snapshot metadata B-tree node contains branch node values if BTNODE_LEAF is not set. The
corresponding inapshot metadata B-tree key represents the first key in the branch.

A snapshot metadata B-tree branch node value is 8 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Sub node block number, which contains a block number relative to the start of the container |

#### Snapshot metadata B-tree leaf node value

The contents of a snapshot metadata B-tree leaf node depends on the
[file system data type](#file_system_data_types) of the key object identifier.

| Value | Description |
| --- | --- |
| APFS_TYPE_SNAP_METADATA | [Snapshot metadata](#snapshot_metadata) object identifier |
| APFS_TYPE_SNAP_NAME | [Snapshot name](#snapshot_name) |

## Fusion drives

A Fusion drive consists of a main SSD and a tier2 magnetic disk that together form one logical APFS
container.

### Fusion middle tree {#fusion_middle_tree}

TODO: complete this section.

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| <td colspan="4">*Object header*</td> |
| 0 | 8 | | [Object checksum](#object_checksum) |
| 8 | 8 | | Object identifier |
| 16 | 8 | | Object transaction identifier (xid) |
| 24 | 4 | 0x40000002 | [Object type](#object_types) |
| 28 | 4 | 0x00000015 | [Object subtype](#object_subtypes) |
| <td colspan="4">*Object values*</td> |
| ... | ... | | Unknown |

<!-- rumdl-enable MD033 MD056 -->

## Format edge cases and corruption scenarios

### Container key bag is hardware encrypted but volume is not encrypted

Seen in APFS containers created by certain digital forensics tools. The container key bag is either
hardware encrypted or contains random data but the volume is not encrypted.

## Notes

TODO describe evict_mapping_val_t

## References

<!-- rumdl-disable MD013 -->

* [Apple File System Reference](https://developer.apple.com/support/downloads/Apple-File-System-Reference.pdf),
  by Apple
