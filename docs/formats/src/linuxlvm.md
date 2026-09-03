# Linux Logical Volume Manager (LVM) format

The Linux Logical Volume Manager (LVM) format is used by the Logical Volume Manager (LVM) on Linux.

## Overview

A Linux LVM consist of:

* Volume group
* Physical volumes
* Logical volumes

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Date and time values | POSIX timestamp in local timezone and/or ctime date and time string |
| Character strings | ASCII string with an end-of-string character |

According to "RHEL 5 - Logical Volume Manager Administration" the number of bytes per sector is 512.

Checksums use a "weak" CRC-32 checksum, which is a CRC-32 without the initial and final XOR
with 0xffffffff, using the polynominal 0xedb88320 and initial value 0xf597a6cf.

### Terminology

| Term | Description |
| --- | --- |
| logical extent (LE) | An extent (data range) that makes up the logical volume |
| logical volume (LV) | A volume within the LVM, comparable to a partition in a [MBR](mbr.md) or [GPT](gpt.md) volume system |
| physical extent (PE) | An extent (data range) that makes up the physical volume |
| physical volume (PV) | Typically a physical volume is a hard disk, though it may well just be any other device that behaves like a hard disk, such as a software RAID device |
| volume group (VG) | A collection of Logical Volumes and Physical Volumes |

## Physical volume

A physical volume consist of:

* Empty sector
* The physical volume label header
  * The physical volume header
    * data area descriptor list
    * metadata area descriptor list
* The metadata area
* Data area (or data extents)

### Physical volume label

The physical volume label is stored in the second sector of the physical volume. The physical
volume label is currently 512 bytes in size and consists of:

* physical volume label header
* physical volume header

> Note that according to "RHEL 5 - Logical Volume Manager Administration" the physical volume label
> can be stored in any of the first 4 sectors.

### Physical volume label header

The physical volume label header (struct label_header) is 32 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "LABELONE" | Signature (or identifier) |
| 8 | 8 | | Sector number, which contains the sector number of the physical volume label header |
| 16 | 4 | | Checksum, which contains a CRC-32 for offset 20 to end of the physical volume label sector |
| 20 | 4 | | Data offset (or header size), which contains an offset in bytes relative from the start of the physical volume label header |
| 24 | 8 | "LVM2\x20001" | Type indicator |

### Physical volume header

The physical volume header (struct pv_header) is of variable size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 32 | | Physical volume identifier, which contains a UUID stored as an ASCII string |
| 32 | 8 | | Physical volume size, in number of bytes |
| 40 | ... | | List of [data area descriptors](#data_area_descriptor), where the last descriptor in the list is terminator and consists of 0-byte values |
| ... | ... | | List of [metadata area descriptors](#data_area_descriptor), where the last descriptor in the list is terminator and consists of 0-byte values |

The physical volume identifier can be used to uniquely identify a physical volume. The physical
volume identifier is stored as "9LBcEB7PQTGIlLI0KxrtzrynjuSL983W" but is equivalent to its
formatted variant "9LBcEB-7PQT-GIlL-I0Kx-rtzr-ynju-SL983W", which is used in the metadata.

> Note that the data area size can be 0, and currently it is assumed that this represents all
> remaining (available) space within the physical volume.

#### Data area descriptor {#data_area_descriptor}

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Data area offset, which contains an offset in bytes relative to the start of the physical volume |
| 8 | 8 | | Data area size, in number of bytes |

## The metadata area

The metadata area consist of:

* Metadata area header
* Metadata

According to "RHEL 5 - Logical Volume Manager Administration" the metadata area is a circular
buffer. New metadata is appended to the old metadata and then the pointer to the start of it is
updated. The metadata area, therefore, can contain copies of older versions of the metadata.

### Metadata area header

The metadata area header (struct mda_header) is 512 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Checksum, which contains the CRC-32 for offset 4 to end of the metadata area header |
| 4 | 16 | "\x20LVM2\x20x[5A%r0N\*>" | Signature |
| 20 | 4 | 1 | Format version |
| 24 | 8 | | Metadata area offset, which contains an offset in bytes relative to the start of the physical volume |
| 32 | 8 | | Metadata area size, in number of bytes |
| 40 | 4 x 24 = 96 | | List of [raw location descriptors](#raw_location_descriptor), where the last descriptor in the list is terminator and consists of 0-byte values |
| 136 | 376 | 0 | Unknown (unused) |

#### Raw location descriptor {#raw_location_descriptor}

The raw location descriptor (struct raw_locn) is 24 bytes in size and consist of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Data area offset, which contains an offset in bytes relative to the start of the metadata area |
| 8 | 8 | | Data area size, in number of bytes |
| 16 | 4 | | Checksum, which contains the CRC-32 of the data area described by the raw location descriptor |
| 20 | 4 | | [Flags](#raw_location_descriptor_flags) |

> Note that the data area size can be 0. It is assumed it represents the remaining available data.

#### Raw location descriptor flags {#raw_location_descriptor_flags}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x00000001 | RAW_LOCN_IGNORED | The raw location descriptor should be ignored |

## Metadata

The metadata consist of:

* Volume group main section
  * Physical volumes object
    * Physical volume object
  * Logical volumes object
    * Logical volume object
      * Segment object
* Top level properties

According to "RHEL 5 - Logical Volume Manager Administration" by default, an identical copy of the
metadata is maintained in every metadata area in every physical volume within the volume group. The
metadata is stored as ASCII.

The metadata can also be stored in a stand-alone file.

### Example

```text
# Generated by LVM2: Tue Jan 30 16:28:15 2007

contents = "Text Format Volume Group"
version = 1

description = "Created *before* executing 'lvextend -L+5G /dev/myvg/mylv /dev/sdc'"

creation_host = "tng3-1"  # Linux tng3-1 2.6.18-8.el5 #1 SMP Fri Jan 26 14:15:21 EST 2007 i686
creation_time = 1170196095  # Tue Jan 30 16:28:15 2007

myvg {
  id = "0zd3UT-wbYT-lDHq-lMPs-EjoE-0o18-wL28X4"
  seqno = 3
  status = ["RESIZEABLE", "READ", "WRITE"]
  extent_size = 8192    # 4 Megabytes
  max_lv = 0
  max_pv = 0

  physical_volumes {

    pv0 {
      id = "ZBW5qW-dXF2-0bGw-ZCad-2RlV-phwu-1c1RFt"
      device = "/dev/sda"   # Hint only

      status = ["ALLOCATABLE"]
      dev_size = 35964301   # 17.1491 Gigabytes
      pe_start = 384
      pe_count = 4390 # 17.1484 Gigabytes
    }

    ...
  }
  logical_volumes {

    mylv {
      id = "GhUYSF-qVM3-rzQo-a6D2-o0aV-LQet-Ur9OF9"
      status = ["READ", "WRITE", "VISIBLE"]
      segment_count = 2

      segment1 {
        start_extent = 0
        extent_count = 1280   # 5 Gigabytes

        type = "striped"
        stripe_count = 1  # linear

        stripes = [
          "pv0", 0
        ]
      }
      segment2 {
        start_extent = 1280
        extent_count = 1280   # 5 Gigabytes

        type = "striped"
        stripe_count = 1  # linear

        stripes = [
          "pv1", 0
        ]
      }
    }
  }
}
```

### Properties

The metadata is textual and uses properties, where a property is defined as:

```text
<identifier> = <value>
```

The identifier contains an unique name of the property and &lt;value&gt; is one of the
following types:

| Value | Description |
| --- | --- |
| [0-9]+ | An integer |
| "..." | A string |
| ["...", "...", ...] | A list (or array) of strings |

> Note that white space, such as space and new line characters, seem to be ignored.

The # character is used for comments. A comment continues to the end-of-line.

> Note that for now it is assumed that the # character is not allowed to be used in any of the
> values.

### Top level properties

| Value | Description |
| --- | --- |
| "contents" | The contents of the metadata area, which contains the string "Text Format Volume Group" |
| "creation_host" | The hostname of the system on which metadata area was created, which can contain a trailing comment that contains the output equivalent to "uname -a" |
| "creation_time" | The creation time of the metadata area, which contains an interger containing the number of seconds since January 1, 1970 00:00:00 UTC and can contain a trailing comment that contains the creation time as a ctime (function) string in UTC |
| "description" | Unknown (Description of the metadata area?) |
| [&lt;volume group name&gt;](#metadata_names) | [Volume group object](#metadata_volume_group_object) |
| "version" | The metadata area version, which contains an integer value of 1 |

> Note it is assumed that there can only be 1 volume group object.

### Volume group and logical volume names {#metadata_names}

* Allowed characters: "a-z", "A-Z", "0-9", "\_", "+", ".", and "-" (hypen)
* A volume group name should not start with a hypen
* A logical volume name can start with a hypen, but it is reserved for logical volumes that have
  been removed but are stil being tracked by the format

> Note that some implementations use lv_ as the prefix for a logical volume, however this is not
> enforced by the format.

### Volume group object {#metadata_volume_group_object}

The volume group object contains the following properties:

| Value | Description |
| --- | --- |
| "extent_size" | The size of an extent, in number of sectors |
| "flags" | [Flags](#flags) |
| "format" | Optional format identifier, such as "lvm2" |
| "id" | Volume group identifier (VG UUID), which contains an ASCII string in the following format: fg1fKZ-xoHz-CfAD-yQPx-l2HL-Y7kA-9kJ9LD |
| "logical_volumes" | [Logical volumes object](#metadata_logical_volumes_object) |
| "max_lv" | Maximum number of logical volumes |
| "max_pv" | Maximum number of physical volumes |
| "metadata_copies" | Unknown (The number of metadata copies?) |
| "physical_volumes" | [Physical volumes object](#metadata_physical_volumes_object) |
| "seqno" | Metadata sequence number |
| "status" | The [status flags](#status_flags), which contains a list of strings |

### Physical volumes object {#metadata_physical_volumes_object}

The physical volumes object contains the following properties:

| Value | Description |
| --- | --- |
| pv# | Individual [physical volume object](#metadata_physical_volume_object), where # is a place holder for a the physical volume number e.g. pv0. 0 appears to be the first number that is used |

### Physical volume object {#metadata_physical_volume_object}

Each physical volume object contains the following properties:

| Value | Description |
| --- | --- |
| "device" | The device filename, which contains an ASCII string, e.g. /dev/dm-0 |
| "device_id" | Unknown (device identifier "/tmp/lvm.raw") |
| "device_id_type" | Unknown (device type "loop_file") |
| "dev_size" | The physical volume size including non-usable space, in number of sectors |
| "flags" | [Flags](#flags) |
| "id" | Physical volume identifier (PV UUID), which contains an ASCII string in the following format: 9LBcEB-7PQT-GIlL-I0Kx-rtzr-ynju-SL983W |
| "pe_count" | The number of (allocated) extents in the physical volume |
| "pe_start" | The start extent, which contains an offset in bytes relative from the start of the physical volume |
| "status" | The [status flags](#status_flags), which contains a list of strings |

### Logical volumes object {#metadata_logical_volumes_object}

The logical volumes object contains the following properties:

| Value | Description |
| --- | --- |
| [&lt;logical volume name&gt;](#metadata_names) | Individual [logical volume object](#metadata_logical_volume_object), where &lt;name&gt; is a place holder for a the logical volume name |

### Logical volume object {#metadata_logical_volume_object}

Each logical volume object contains the following properties:

| Value | Description |
| --- | --- |
| "creation_host" | The hostname of the system on which the logical volume was created |
| "creation_time" | The creation time of the metadata area, which contains an interger containing the number of seconds since January 1, 1970 00:00:00 UTC and can contain a trailing comment that contains the creation time as a ctime (function) string in UTC |
| "flags" | [Flags](#flags) |
| "id | Physical volume identifier (PV UUID), which contains an ASCII string in the following format: 9LBcEB-7PQT-GIlL-I0Kx-rtzr-ynju-SL983W |
| "segment_count" | The number of segments |
| "segment#" | Individual [segment objects](#metadata_segment_object), where # is a place holder for the segment number e.g. segment1. 1 appears to be the first number that is used |
| "status" | [Status flags](#status_flags), which contains a list of strings |

### Segment object {#metadata_segment_object}

Each segment object contains the following properties:

| Value | Description |
| --- | --- |
| "extent_count" | The number of extents in the segment (or current logical extent) |
| "start_extent" | The start extent of the segment, which contains an offset in number of extents relative from the start of the segment |
| "stripe_count" | The number of stripes in the segment, where 1 represents linear striping |
| "stripes" | [Stripes list](#metadata_stripes_list) |
| "type" | [Segment type](#metadata_segment_types) |

### Segment types {#metadata_segment_types}

| Value | Description |
| --- | --- |
| cache | Segment that is used as a cache, e.g. SSD for a HDD |
| cache-pool | Segment used as a cache for both data and metadata |
| error | Segment that fails read and write operations, intended for testing |
| free | Unallocated segment |
| integrity | Segment with integrity checksum |
| linear | Linear physical volume |
| mirror | Segment that mirrors data across multiple physical volumes (comparable to RAID1) |
| raid0 | Segment that contains a stripe of data (RAID0) |
| raid0_meta | RAID0 physical volume that contains data and metadata |
| raid1 | Segment that mirrors data across multiple physical volumes (RAID1) |
| raid10 | Segment that contains a stripe of data (RAID0) that mirrors data across multiple physical volumes (RAID1) |
| raid10_near | RAID10 segment that copies data sectors near each other |
| raid4 | Segment with stripes across multiple physical volumes with parity data (RAID4) |
| raid5 | Segment with stripes across multiple physical volumes with parity data (RAID5) |
| raid5_la | RAID5 variant that uses left asymmetric layout |
| raid5_ls | RAID5 variant that uses left symmetric layout |
| raid5_n | RAID5 variant with dedicated parity layout |
| raid5_ra | RAID5 variant that uses right asymmetric layout |
| raid5_rs | RAID5 variant that uses right symmetric layout |
| raid6 | Segment with stripes across multiple physical volumes with dual parity data (RAID6) |
| raid6_la_6 | RAID6 variant that uses left asymmetric layout |
| raid6_ls_6 | RAID6 variant that uses left symmetric layout |
| raid6_n_6 | RAID6 variant with dedicated parity layout |
| raid6_nc | RAID6 variant that uses no-change layout |
| raid6_nr | RAID6 variant that uses non-rotating layout |
| raid6_ra_6 | RAID6 variant that uses right asymmetric layout |
| raid6_rs_6 | RAID6 variant that uses right symmetric layout |
| raid6_zr | RAID6 variant that uses zero-restart layout |
| snapshot | Segment that contains a snapshot |
| striped | Segment that contains a stripe of data (comparable to RAID0) |
| thin | Segment that allocated on write |
| thin-pool | Segment that manages a shared pool of data and metadata |
| vdo | Virtual Data Optimizer (VDO) physical volume |
| vdo-pool | Virtual Data Optimizer (VDO) pool |
| writecache | Segment that is used as a write-only cache |
| zero | Segment that is sparse (filled with 0-byte values) |

> Note that a comparable list can be retrieved using `lvm segtypes`.

### Stripes list {#metadata_stripes_list}

```text
stripes = [
<physical volume name>, <start extent number>
]
```

Where:

* &lt;physical volume name&gt; is a string containing the physical volume name e.g. "pv0".
* &lt;start extent number&gt; the segment start extent number relative from the start of the data
  area.

```python
start_extent_offset = (
    (start_extent_number * extent_size * sector_size) + physical_volume_data_area_start_offset
)
```

### Status flags {#status_flags}

| Value | Description |
| --- | --- |
| ALLOCATABLE | Is allocatable (physical volume only) |
| RESIZEABLE | Can be re-sized (volume group only) |
| READ | Can be read |
| VISIBLE | Is visible (logical volume only). Hidden if not set. |
| WRITE | Can be written |

### Flags {#flags}

TODO: complete section

### Comments

Textual metadata such as:

```text
# Generated by LVM2 version 2.02.39 (2008-06-27): Sat Jan 17 11:45:29 2009
```

## References

<!-- rumdl-disable MD013 -->

* [LVM HOWTO](https://tldp.org/HOWTO/LVM-HOWTO)
* [RHEL 5 - Logical Volume Manager Administration](https://docs.redhat.com/en/documentation/Red_Hat_Enterprise_Linux/5/html/Logical_Volume_Manager_Administration/lvm_metadata.html)

<!-- rumdl-enable MD013 -->
