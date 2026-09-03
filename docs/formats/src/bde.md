# BitLocker Drive Encryption (BDE) format

The BitLocker Drive Encryption (BDE) format is used by Microsoft Windows to encrypt volumes.

## Overview

There are multiple versions of BitLocker Drive Encryption (BDE):

* BitLocker Windows Vista and Windows 7; used to encrypt NTFS volumes on fixed storage media, like
  harddisks.
* BitLocker To Go; introduced in Windows 7 and used to encrypt removable drives, which typically
  contain FAT file systems.
* BitLocker Used Disk Space Only encryption

> Note that Windows treats NTFS volumes on removable drives are treated as NTFS volumes on fixed
> storage media.

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |
| Date and time values | FILETIME in UTC |
| Character strings | UCS-2 little-endian, which allows for unpaired Unicode surrogates such as "U+d800" and "U+dc00" |

BitLocker is known to use the following identifiers:

* 4967d63b-2e29-4ad8-8399-f6a339e3d001 (BitLocker and BitLocker To Go)
* 92a84d3b-dd80-4d0e-9e4e-b1e3284eaed8 (BitLocker Used Disk Space Only encryption)

## Metadata files

BitLocker exposes various files in the "\System Volume Information" directory of the unencrypted
volume that correspond to the BitLocker metadata areas.

The contents of the metadata files, on an unencrypted volume, consists of 0-byte values. It is
assumed that these files are used to prevent the BitLocker metadata to be overwritten.

> Note that not all tools zero out the metadata areas.

### Windows Vista

In Windows Vista the "\System Volume Information" directory contains the following BitLocker related
files:

* "FVE.{%GUID%}.[123]" maps the blocks that contain the FVE metadata; typically 16384 bytes in size.

### Windows 7

In Windows 7 the "\System Volume Information" directory contains the following BitLocker related
files:

* "FVE2.{%GUID%}" maps the block that contains the encrypted volume header; typically 8192 bytes in
  size.
* "FVE2.{%GUID%}.[123]" maps the blocks that contain the FVE metadata; typically 65536 bytes in
  size.

### To Go

BitLocker To Go uses a hybrid volume that has a encrypted and an unencrypted part. The unencrypted
part contains various files. Application files for the BitLocker To Go helper application; which
can also be found in:

```text
C:\Windows\BitLockerDiscoveryVolumeContents\
```

* "COV 0000. BL" maps the block that contains the BitLocker To Go GUID and the offsets to the
  metadata; typically 32768 bytes in size.
* "COV 0000. ER" maps the encrypted data.
* "PAD 0000. PD" maps padding.
* "PAD 0000. NG" unknown; typically 0 bytes in size.

It has been observed that the "COV 0000. ER" and "PAD 0000. NG" files can be split in multiple
4294934528 byte (4 GiB - 32768) on a FAT32 volume, such as "COV 0001. ER", "COV 0002. ER", ... or
"PAD 0001. NG", ...

The "PAD 0000. NG" are presumaly used to fill the root directory with entries so that no new files
may be created on the volume.

## Keys

To encrypt storage media BitLocker uses different kind of keys.

### Volume Master Key (VMK)

The Volume Master Key (VMK) is 256-bit of size and is stored in multiple FVE Volume Master Key
(VMK) structures. The VMK is stored encrypted with either the recovery key, external key, or the
TPM.

It is also possible that the VMK is stored unencrypted which is referred to as clear key.

### Full Volume Encryption Key (FVEK)

The Full Volume Encryption Key (FVEK) is stored encrypted with the Volume Master Key (VMK). The
size of the FVEK is dependent on the encryption method used:

* For AES 128-bit the key is 128-bit of size
* For AES 256-bit the key is 256-bit of size

When Elephant Diffuser is used the key data of the structure that hold the FVEK is always 512-bit
of size. The First 256-bit are reserved for the FVEK and the other 256-bit for the TWEAK key. Only
128-bit of the 256-bits are used when the encryption method is AES 128-bit.

### TWEAK key

The TWEAK is stored encrypted with the Volume Master Key (VMK). The size of the TWEAK key is
dependent on the encryption method used:

* For AES 128-bit the key is 128-bit of size
* For AES 256-bit the key is 256-bit of size

The TWEAK key is only present when Elephant Diffuser is used. The TWEAK key is stored in the key
data of the structure that hold the Full Volume Encryption Key (FVEK) is always 512-bit of size.
The First 256-bit are reserved for the FVEK and the other 256-bit for the TWEAK key. Only 128-bit
of the 256-bits are used when the encryption method is AES 128-bit.

### Recovery key

BitLocker provides for a recovery (or numerical) password to unlock the encrypted data. The
recovery password is used to determine a recovery key.

Example recovery password:

```text
471207-278498-422125-177177-561902-537405-468006-693451
```

A valid recovery password consists of 48 digits where every number is dividable by 11 with a
remainder of 0. The result of a division by 11 of a number is a 16-bit value. The individual 16-bit
values make up a 128-bit key.

The corresponding recovery key is calculated using the following approach, written partially in
pseudo C:

Initialize a structure consisting of:

```text
uint8_t last_sha256[ 32 ];
uint8_t initial_sha256[ 32 ];
uint8_t salt[ 16 ];
uint64_t count;
```

Initialize both the last SHA256 and the count to 0.

Calculate the SHA256 of the 128-bit key and update the initial SHA256 value.

The salt is stored on disk in the stretch key which is stored in the recovery key protected Volume
Master Key (VMK).

Loop for 1048576 (0x100000) times:

* calculate the SHA256 of the structure and update the last SHA256 value
* increment the count by 1

The last SHA256 value contains the 256-bit key which is recovery key that can unlock the recovery
key protected Volume Master Key (VMK).

### Clear key

The clear key is an unprotected 256-bit key stored on the volume to decrypt the VMK. It is used
when the encrypted volume is being decrypted.

### Startup key

The startup key (or external key) is stored in a file named "{%GUID%}.BEK". The GUID in the filename
equals the key identifier in the BitLocker metadata.

There can be multiple startup keys for a single BitLocker volume. Each key is identified a by a
different key identifier.

### User key

BitLocker To Go provides for a user password (or passphrase) to unlock the encrypted data. The user
password is used to determine a user key.

TODO: check if the password can be maximal 49 characters in size.

Convert the user password into a UTF16 little-endian string.

Initialize a structure consisting of:

```text
uint8_t last_sha256[ 32 ];
uint8_t initial_sha256[ 32 ];
uint8_t salt[ 16 ];
uint64_t count;
```

Initialize both the last SHA256 and the count to 0.

Calculate the SHA256 of the user password.

Calculate the SHA256 of the SHA256 of the user password, and set it as the initial SHA256 value.

The salt is stored on disk in the stretch key which is stored in the user key (or password)
protected Volume Master Key (VMK).

Loop for 1048576 (0x100000) times:

* calculate the SHA256 of the structure and update the last SHA256 value
* increment the count by 1

The last SHA256 value contains the 256-bit key which is user key that can unlock the user key (or
password) protected Volume Master Key (VMK).

## Encryption methods

BitLocker uses different kind of encryption methods. To encrypt the sector data it either uses
AES-CBC with or without Elephant Elephant Diffuser. To encrypt the key data BitLocker uses AES-CCM.

### AES-CBC

Both encryption and decryption use:

* AES-CBC with FVEK decryption of sector data

The initialization vector of the AES-CBC is the sector offset AES-ECB encrypted with the FVEK
stored as a 16-byte little-endian value. The sector offset is the offset of the sector relative
from the start of the volume.

### AES-CBC with Elephant Diffuser

Encryption:

* XOR with sector key
* Elephant Elephant Diffuser A
* Elephant Elephant Diffuser B
* AES-CBC with FVEK

Decryption:

* AES-CBC with FVEK
* Elephant Elephant Diffuser B
* Elephant Elephant Diffuser A
* XOR with sector key

The initialization vector of the AES-CBC is the sector offset AES-ECB encrypted with the FVEK
stored as a 16-byte little-endian value. The sector offset is the offset of the sector relative
from the start of the volume.

The sector key 32-byte of size and contains:

* the lower 16-byte contain a little-endian version of the offset of the sector, relative from the
  start of the volume, AES-ECB encrypted with the TWEAK key
* the upper 16-byte contain a 16-byte little-endian version of the offset of the sector, relative
  from the start of the volume, with the most upper bit set (or upper byte set to 0x80) AES-ECB
  encrypted with the TWEAK key

### AES-CCM

The key data is encrypted using AES-CCM with an initialization vector of 0.

### AES-XTS

The FVEK contains both XTS keys.

Both encryption and decryption use:

* AES-XTS with FVEK decryption of sector data

The initialization vector of the AES-XTS is the sector number stored as a 16-byte little-endian
value. The sector number is the offset of the sector relative from the start of the volume divided
by the sector size.

### Elephant Diffuser

The Elephant Diffuser A and B variants are described in "AES-CBC + Elephant diffuser - A Disk
Encryption Algorithm for Windows Vista".

### Virtual sector(s)

In BitLocker the certain sector(s) of the encrypted storage media are handled in a specific manner.
These are sectors to store:

* the unencrypted volume header
* the BitLocker metadata

#### BitLocker Windows Vista

In BitLocker Windows Vista the first sector of the unencrypted volume header sector is
reconstructed by replacing values in the BitLocker Volume header, namely

* replacing the "File system signature" with "NTFS\x20\x20\x20\x20"
* replacing the "FVE metadata block 1 cluster block number" with the "MTF mirror cluster block
  number"

The 15 sectors directly following the first sector are also unencrypted.

The sectors that contain the BDE metadata are shown as empty sectors; containing 0-byte values.

#### BitLocker Windows 7 and To Go

Both BitLocker Windows 7 and To Go store an encrypted version of the unencrypted first sectors in a
specific location. This location is defined in the
[FVE Volume header block](#fve_volume_header_block). It is commonly 8192 bytes an size, entailing
the first 16 sectors.

The sectors that contain the encrypted volume header and the BDE metadata are shown as empty
sectors; containing 0-byte values.

#### BitLocker Windows 10

In later versions of Bitlocker Windows 10 the [FVE Volume header block](#fve_volume_header_block)
no longer is present. The number of volume header sectors in the
[FVE metadata block header](#fve_metadata_block_header2) can be used to determine the volume header
size. It is commonly 8192 bytes an size, entailing the first 16 sectors.

## Volume header

### BitLocker Windows Vista

The BitLocker Windows Vista volume header is similar to NTFS volume header. The differences have
been emphasized in bold. The volume header is 512 bytes of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 3 | "\xeb\x52\x90" | Boot entry point |
| **3** | **8** | **"-FVE-FS-"** | **File system signature** |
| <td colspan="4">*DOS version 2.0 BIOS parameter block (BPB)*</td> |
| 11 | 2 | | Bytes per sector |
| 13 | 1 | | Sectors per cluster block |
| 14 | 2 | 0x00 | Reserved Sectors |
| 16 | 1 | 0x00 | Number of File Allocation Tables (FATs) |
| 17 | 2 | 0 | Root directory entries |
| 19 | 2 | | Total number of sectors (16-bit) |
| 21 | 1 | | Media descriptor |
| 22 | 2 | 0x00 | Sectors Per File Allocation Table (FAT) |
| <td colspan="4">*DOS version 3.4 BIOS parameter block (BPB)*</td> |
| 24 | 2 | 0x3f | Sectors per track |
| 26 | 2 | | Number of heads |
| 28 | 4 | | Number of hidden sectors |
| 32 | 4 | 0x00 | Total number of sectors (32-bit) |
| <td colspan="4">*NTFS version 8.0 BIOS parameter block (BPB) or extended BPB*</td> |
| 36 | 1 | 0x80 | Unknown (Disc unit number) |
| 37 | 1 | 0x00 | Unknown (Flags) |
| 38 | 1 | 0x80 | Unknown (BPB version signature byte) |
| 39 | 1 | 0x00 | Unknown (Reserved) |
| 40 | 8 | | Total number of sectors (64-bit) |
| 48 | 8 | | Master File Table (MFT) cluster block number |
| **56** | **8** | | **FVE metadata block 1 cluster block number** |
| 64 | 1 | | MFT entry size |
| 65 | 3 | | Unknown |
| 68 | 1 | | Index entry size |
| 69 | 3 | | Unknown |
| 72 | 8 | | NTFS volume serial number |
| 80 | 4 | 0x00 | Checksum |
| 84 | 426 | | Bootcode |
| 510 | 2 | 0x55 0xaa | Sector signature |

<!-- rumdl-enable MD033 MD056 -->

> Note that the number of sectors can be 1 less then the value indicated in the partition table.

### BitLocker Windows 7 and later

The BitLocker Windows 7 (and later) volume header less similar to NTFS volume header than the
BitLocker Windows Vista volume header. The differences between the versions have been emphasized in
bold. The volume header is 512 bytes of size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| **0** | **3** | **"\xeb\x58\x90"** | **Boot entry point** |
| 3 | 8 | "-FVE-FS-" | File system signature |
| <td colspan="4">*DOS version 2.0 BIOS parameter block (BPB)*</td> |
| 11 | 2 | | Bytes per sector |
| 13 | 1 | | Sectors per cluster block |
| 14 | 2 | 0x00 | Reserved Sectors |
| 16 | 1 | 0x00 | Number of File Allocation Tables (FATs) |
| 17 | 2 | 0 | Root directory entries |
| 19 | 2 | | Total number of sectors (16-bit) |
| 21 | 1 | | Media descriptor |
| 22 | 2 | 0x00 | Sectors Per File Allocation Table (FAT) |
| <td colspan="4">*DOS version 3.4 BIOS parameter block (BPB)*</td> |
| 24 | 2 | 0x3f | Sectors per track |
| 26 | 2 | | Number of heads |
| **28** | **4** | | **Number of hidden sectors**, which contains the volume start sector number |
| 32 | 4 | 0x00 | Total number of sectors (32-bit) |
| <td colspan="4">*Unknown*</td> |
| **36** | **4** | **0x1fe0** | **Sectors per file allocation table** |
| **40** | **2** | | **FAT Flags (Only used during a conversion from a FAT12/16 volume)** |
| **42** | **2** | | **Version (Defined as 0)** |
| **44** | **4** | | **Cluster number of root directory start** |
| **48** | **2** | **0x0001** | **Sector number of FS Information Sector** |
| **50** | **2** | **0x0006** | **Sector number of a copy of this boot sector (0 if no backup copy exists)** |
| **52** | **12** | | **Unknown (Reserved)** |
| **64** | **1** | **0x80** | **Physical Drive Number (see FAT12/16 BPB at offset 0x24)** |
| **65** | **1** | | **Unknown (Reserved) (see FAT12/16 BPB at offset 0x25)** |
| **66** | **1** | **0x29** | **Extended boot signature (see FAT12/16 BPB at offset 0x26)** |
| **67** | **4** | | **Volume serial number** |
| **71** | **11** | **"NO NAME\x20\x20\x20\x20"** | **Volume label** |
| **82** | **8** | **"FAT32\x20\x20\x20"** | **File system signature** |
| **90** | **70** | | **Bootcode** |
| **160** | **16** | | **BitLocker identifier**, which contains a GUID |
| **176** | **8** | | **FVE metadata block 1 offset**, which contains an offset relative to the start of the volume |
| **184** | **8** | | **FVE metadata block 2 offset**, which contains an offset relative to the start of the volume |
| **192** | **8** | | **FVE metadata block 3 offset**, which contains an offset relative to the start of the volume |
| **200** | **307** | | **Unknown (part of bootcode)** |
| **507** | **3** | | **Unknown** |
| 510 | 2 | 0x55 0xaa | Sector signature |

<!-- rumdl-enable MD033 MD056 -->

> Note that the number of sectors can be 1 less then the value indicated in the partition table.

### BitLocker To Go

BitLocker To Go on an NTFS volume is similar to BitLocker Windows 7. The BitLocker Windows To Go
volume header for a FAT volume is similar to FAT32 volume header. The differences have been
emphasized in bold. The volume header is 512 bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 3 | "\xeb\x58\x90" | Boot entry point |
| **3** | **8** | **"MSWIN4.1"** | **Signature** |
| <td colspan="4">*DOS version 2.0 BIOS parameter block (BPB)*</td> |
| 11 | 2 | | Bytes per sector |
| 13 | 1 | | Sectors per cluster block |
| 14 | 2 | 0x00 | Reserved Sectors |
| 16 | 1 | 0x00 | Number of File Allocation Tables (FATs) |
| 17 | 2 | 0 | Root directory entries |
| 19 | 2 | | Total number of sectors (16-bit) |
| 21 | 1 | | Media descriptor |
| 22 | 2 | 0x00 | Sectors Per File Allocation Table (FAT) |
| <td colspan="4">*DOS version 3.4 BIOS parameter block (BPB)*</td> |
| 24 | 2 | 0x3f | Sectors per track |
| 26 | 2 | | Number of heads |
| 28 | 4 | | Number of hidden sectors |
| 32 | 4 | | Total number of sectors (32-bit) |
| <td colspan="4">*Unknown*</td> |
| 36 | 4 | 0x1fe0 | Sectors per file allocation table |
| 40 | 2 | | FAT Flags (Only used during a conversion from a FAT12/16 volume) |
| 42 | 2 | | Version (Defined as 0) |
| 44 | 4 | | Cluster number of root directory start |
| 48 | 2 | 0x0001 | Sector number of FS Information Sector |
| 50 | 2 | 0x0006 | Sector number of a copy of this boot sector (0 if no backup copy exists) |
| 52 | 12 | | Unknown (Reserved) |
| 64 | 1 | 0x80 | Physical Drive Number (see FAT12/16 BPB at offset 0x24) |
| 65 | 1 | | Unknown (Reserved) (see FAT12/16 BPB at offset 0x25) |
| 66 | 1 | 0x29 | Extended boot signature (see FAT12/16 BPB at offset 0x26) |
| 67 | 4 | | Volume serial number |
| 71 | 11 | "NO NAME\x20\x20\x20\x20" | Volume label |
| 82 | 8 | "FAT32\x20\x20\x20" | File system signature |
| 90 | 334 | | Bootcode |
| **424** | **16** | | **BitLocker identifier**, which contains a GUID |
| **440** | **8** | | **FVE metadata block 1 offset**, which contains an offset relative to the start of the volume |
| **448** | **8** | | **FVE metadata block 2 offset**, which contains an offset relative to the start of the volume |
| **456** | **8** | | **FVE metadata block 3 offset**, which contains an offset relative to the start of the volume |
| 464 | 46 | | Unknown |
| 510 | 2 | 0x55 0xaa | Sector signature |

<!-- rumdl-enable MD033 MD056 -->

## FVE metadata block

A BitLocker volume contains 3 FVE metadata blocks. Each FVE metadata block consists of:

* a block header
* a metadata header
* an array of metadata entries
* padding (0-byte values) (seen in Windows 8)

### FVE metadata block header

#### FVE metadata block header version 1 - Windows Vista

The FVE metadata block header version 1 is 64 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "-FVE-FS-" | Signature |
| 8 | 2 | | Unknown (Size) |
| 10 | 2 | 1 | Version |
| 12 | 2 | | Unknown, which is commonly 0x04 |
| 14 | 2 | | Unknown, which is commonly 0x04 |
| 16 | 16 | 0 | Unknown (empty values) |
| 32 | 8 | | FVE metadata block 1 offset, which is relative to the start of the volume |
| 40 | 8 | | FVE metadata block 2 offset, which is relative to the start of the volume |
| 48 | 8 | | FVE metadata block 3 offset, which is relative to the start of the volume |
| 56 | 8 | | MFT mirror cluster block number |

#### FVE metadata block header version 2 – Windows 7 and later {#fve_metadata_block_header2}

The FVE metadata block header version 2 is 64 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "-FVE-FS-" | Signature |
| 8 | 2 | | Unknown (Size) |
| 10 | 2 | 2 | Version |
| 12 | 2 | | Unknown, which is commonly 0x04, but 0x05 has been observed in a partial decrypted volume (protection status?) |
| 14 | 2 | | Unknown, which is commonly 0x04, but 0x01 has been observed in a partial decrypted volume |
| 16 | 8 | | Encrypted volume size, in number of bytes |
| 24 | 4 | | Unknown |
| 28 | 4 | | Number of volume header sectors |
| 32 | 8 | | FVE metadata block 1 offset, which contains an offset relative to the start of the volume |
| 40 | 8 | | FVE metadata block 2 offset, which contains an offset relative to the start of the volume |
| 48 | 8 | | FVE metadata block 3 offset, which contains an offset relative to the start of the volume |
| **56** | **8** | | **Volume header offset**, which contains an offset relative to the start of the volume |

When decrypting BitLocker will decrypt from the back to the front. The encrypted volume size
therefore contains the number of bytes of the volume that are still encrypted (or need to be
decrypted).

### FVE metadata header (version 1)

The FVE metadata header (version 1) is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Metadata size, which includes the size value |
| 4 | 4 | 1 | Version |
| 8 | 4 | 48 | Metadata header size |
| 12 | 4 | | Metadata size copy |
| 16 | 16 | | Volume identifier, which contains a GUID |
| 32 | 4 | | Next nonce counter |
| 36 | 4 | | [Encryption method](#encryption_methods) |
| 40 | 8 | | Creation time, which contains a FILETIME |

> Note that it is currently unknown what the upper 16-bit of the encryption method value is used
> for. The MSB has been observed to be used or is this value actually 2x 16-bit values.

#### Encryption methods {#encryption_methods}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | | Unknown (Not encrypted/External Key) |
| | | |
| 0x1000 | | Unknown (Stretch key) |
| 0x1001 | | Unknown (Stretch key) |
| | | |
| 0x2000 | | Unknown (AES-CCM 256 bit encryption) |
| 0x2001 | | Unknown (AES-CCM 256 bit encryption) |
| 0x2002 | | Unknown (AES-CCM 256 bit encryption) |
| 0x2003 | | Unknown (AES-CCM 256 bit encryption) |
| 0x2004 | | Unknown (AES-CCM 256 bit encryption) |
| 0x2005 | | Unknown (AES-CCM 256 bit encryption) |
| | | |
| 0x8000 | | AES-CBC 128-bit encryption with Elephant Diffuser |
| 0x8001 | | AES-CBC 256-bit encryption with Elephant Diffuser |
| 0x8002 | | AES-CBC 128-bit encryption |
| 0x8003 | | AES-CBC 256-bit encryption |
| 0x8004 | | AES-XTS 128-bit encryption |
| 0x8005 | | Unknown (AES-XTS 256-bit encryption) |

### FVE metadata entry

The FVE metadata entry is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 2 | | Entry size, which includes the size value |
| 2 | 2 | | Entry type |
| 4 | 2 | | Value type |
| 6 | 2 | | Version (Seen: 1 and 3) |
| 8 | ... | | Data |

> Note that the version is typically 1 but 3 has been seen for VMK FVE metadata entry in
> combination with clear key.

#### FVE metadata entry types

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | | None, entry is a property |
| | | |
| 0x0002 | | Volume Master Key (VMK) |
| 0x0003 | | Full Volume Encryption Key (FVEK) |
| 0x0004 | | Unknown (Validation) |
| | | |
| 0x0006 | | Startup key |
| 0x0007 | | Description (Drive label), which contains computer name, volume name and date |
| | | |
| 0x000b | | Unknown (Backup of the Full Volume Encryption Key (FVEK)?) |
| | | |
| 0x000f | | Volume header block |

TODO: determine if the date format of the description is dependent on the locale, observed
"MM/DD/YYYY"

#### FVE metadata value types

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | | Erased |
| 0x0001 | | Key |
| 0x0002 | | String, which contains an UCS-2 little-endian string with end-of-string character |
| 0x0003 | | Stretch Key |
| 0x0004 | | Use Key |
| 0x0005 | | AES-CCM encrypted key |
| 0x0006 | | TPM encoded key |
| 0x0007 | | Validation |
| 0x0008 | | Volume master key |
| 0x0009 | | External key |
| 0x000a | | Update |
| 0x000b | | Error |
| | | |
| 0x000f | | Unknown (Offset and size), contains a tuple of 2 x 64-bit values |

### FVE key

The FVE Stretch encrypted key has value type 0x0001. It is variable in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Encryption method](#encryption_methods) |
| 4 | ... | | Key data |

### FVE Stretch encrypted key

The FVE Stretch encrypted key has value type 0x0003. It is variable in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Encryption method](#encryption_methods) |
| 4 | 16 | | Salt |
| 20 | ... | | FVE metadata entry, which contains an AES-CCM encrypted key |

### FVE AES-CCM encrypted key

The FVE AES-CCM encrypted key has value type 0x0005. It is variable in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Nonce date and time, which contains a FILETIME |
| 8 | 4 | | Nonce counter |
| 12 | ... | | AES-CCM encrypted data |

#### Unencrypted data

The unencrypted data is of variable size and consist of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Message Authentication Code (MAC) |
| <td colspan="4">*Key container*</td> |
| 16 | 4 | | Size, which does not include the size of the MAC |
| 20 | 2 | 1 | Unknown (Version) |
| 22 | 2 | | Unknown |
| 24 | 4 | | [Encryption method](#encryption_methods) |
| 28 | ... | | Unencrypted key data |

<!-- rumdl-enable MD033 MD056 -->

### FVE TPM encoded key

The FVE TPM encoded key has value type 0x0006. It is variable in size and consists of:

TODO: complete section

### FVE Validation

The FVE Validation has value type 0x0007. It is variable in size and consists of:

TODO: complete section

### FVE Volume Master Key (VMK)

The FVE Volume Master Key has value type 0x0008. It is variable in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Key identifier, which contains a GUID |
| 16 | 8 | | Last modification date and time, which contains a FILETIME |
| 24 | 2 | | Unknown |
| 26 | 2 | | [Protection type](#key_protection_types) |
| 28 | ... | | Properties, which contains an array of FVE metadata entries where the entry type is set to 0  |

The available properties depend on the VMK type.

The clear key protected VMK consists of:

* key (with 256-bit of key data)
* AES-CCM encrypted key

The recovery key protected VMK consists of:

* optional description string containing "DiskPassword\x00"
* stretch key
* AES-CCM encrypted key

The startup key protected VMK consists of:

* optional description string containing "ExternalKey\x00"
* stretch key
* AES-CCM encrypted key

The password protected VMK consists of:

* optional description string containing "ExternalKey\x00"
* stretch key
* AES-CCM encrypted key

#### Key protection types {#key_protection_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0x0000 | | VMK protected with clear key, which basically is an unprotected VMK |
| | | |
| 0x0100 | | VMK protected with TPM |
| 0x0200 | | VMK protected with startup key |
| | | |
| 0x0500 | | VMK protected with TPM and PIN |
| | | |
| 0x0800 | | VMK protected with recovery password |
| | | |
| 0x2000 | | VMK protected with password |

##### Notes

Key protector types defined by the GetKeyProtectorType function documentation:

```text
0 Unknown or other protector type
1 Trusted Platform Module (TPM)
2 External key
3 Numerical password
4 TPM And PIN
5 TPM And Startup Key
6 TPM And PIN And Startup Key
7 Public Key
8 Passphrase
9 TPM Certificate
10 CryptoAPI Next Generation (CNG) Protector
```

### FVE External Key

The FVE External Key has value type 0x0009. It is variable in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Key identifier, which contains a GUID |
| 16 | 8 | | Last modification date and time, which contains a FILETIME |
| 24 | ... | | Properties, which contains an array of FVE metadata entries where the entry type is set to 0 |

The available properties:

* optional description string containing "ExternalKey\x00"
* key

### FVE Volume header block {#fve_volume_header_block}

The FVE Volume header block has value type 0x000f. It is 16 or more bytes in size and consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Block offset |
| 8 | 8 | | Block size |
| <td colspan="4">*Unknown additional data*</td> |
| 16 | 2 | | Unknown (number of entries?) |
| 18 | 2 | | Unknown (size of additional data?) |
| 28 | ... | | Unknown (array of 14 byte sized entries) |
| ... | 2 | | Unknown (empty values) |

<!-- rumdl-enable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Unknown (seen combination of block offset +  block size) |
| 8 | 4 | | Unknown |
| 12 | 2 | | Unknown |

The FVE Volume header block seems to have been introduced in Windows 7. It specifies the location
in the encrypted volume where the unencrypted volume header is stored.

The FVE Volume header block is commonly 8192 bytes in size for Windows 7 and 5365760 bytes for a
BitLocker To Go.

## BitLocker External Key (BEK) file

A BitLocker External Key (BEK) file is commonly 156 bytes in size and consists of:

* a file header
* an array of metadata entries

### BEK file header (version 1)

The BEK file header is similar to the FVE metadata header (version 1). The BEK file header
(version 1) is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Metadata size, which includes the size value |
| 4 | 4 | 1 | Version |
| 8 | 4 | 48 | Metadata header size |
| 12 | 4 | | Unknown (Metadata size copy) |
| 16 | 16 | | Volume identifier, which contains a GUID |
| 32 | 4 | | Next nonce counter |
| 36 | 4 | | [Encryption method](#encryption_methods) |
| 40 | 8 | | Creation time, which contains a FILETIME |

The key identifier in the file must match the key identifier in the FVE Volume Master Key (VMK).

### BEK metadata entry (version 1)

The format of a BEK metadata entry (version 1) is similar to the format of a FVE metadata entry
(version 1).

The metadata in a BEK file consists of an FVE external key, which contains 256-bits of unprotected
key data.

The identifier of the VMK should match the identifier in the BEK file header.

## Notes

Seen on an unencrypted BDE volume (password was set shorted then 8 chars to bdetest).

### FVE-EOW block

The FVE-EOW block has ...

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "FVE-EOW\x00" | Signature |

### FVE-EOWBM block

The FVE-EOWBM block has ...

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 10 | "FVE-EOWBM\x00" | Signature |
| 10 | 2 | | Block size, which includes the signature and size value |
| 12 | 4 | | Unknown |

### FVE-EOWBR block

The FVE-EOWBR block has ...

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 10 | "FVE-EOWBR\x00" | Signature |
| 10 | 2 | | Block size, which includes the signature and size value |
| 12 | 4 | | Unknown |

### OLRDHEVF2 block

The OLRDHEVF2 block has ...

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 10 | "OLRDHEVF2\x00" | Signature |
| 10 | 2 | | Unknown |

## References

* [AES-CBC + Elephant diffuser - A Disk Encryption Algorithm for Windows Vista](http://download.microsoft.com/download/0/2/3/0238acaf-d3bf-4a6d-b3d6-0a0be4bbb36e/bitlockercipher200608.pdf),
  by N. Ferguson
