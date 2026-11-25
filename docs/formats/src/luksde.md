# Linux Unified Key Setup (LUKS) Disk Encryption format

Linux Unified Key Setup (LUKS) Disk Encryption is commonly used by Linux to encrypt storage media
volumes.

## Overview

There are 2 versions of the Linux Unified Key Setup (LUKS) Disk Encryption format, each with a
separate layout.

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | N/A |
| Character strings | ASCII string with an end-of-string character |

### Layout version 1

A LUKS version 1 encrypted volume consist of:

* [volume header](#volume_header_v1)
* 8 x [key slots](#key_slot_v1)
* split master key material
* encrypted (volume) data

The total reserved size of the LUKS metadata (volume header and split master key material) seems to
commonly be 2 MiB.

The number of bytes per sector is 512.

### Layout version 2

A LUKS version 2 encrypted volume consist of:

* metadata area
  * [volume header](#volume_header_v2)
  * [JSON area](#json_area)
* backup metadata area
  * backup volume header
  * backup JSON area
* keyslots area
* encrypted (volume) data

The number of bytes per sector is stored in the JSON metadata and can be 512, 1024, 2048, or 4096.

## Keys

To encrypt storage media LUKS Disk Encryption uses different kind of keys.

### Master Key (MK)

The Master Key (MK) is derived from the Split Master Key (SMK). The size of the MK is dependent on
the master key size value in the volume header (format version 1) or keyslot metadata (format
version 2). Commonly the MK is 128-bit, 256-bit, or 512-bit (for XTS mode) in size. The MK is used
to de/encrypt the encrypted (volume) data.

### Split Master Key (SMK)

The Split Master Key (SMK) is stored encrypted with a specific user key (UK) in the split master
key material. The size of the key material and hence the SMK is the size of the Master Key (MK)
times the number of stripes.

In format version 2, the raw keyslot area is always encrypted in 512-byte sectors (independent of
the data segment sector size), using the cipher specified in the keyslot "area.encryption" property
and sector numbers starting at 0.

The MK is determined from the SMK using the anti-forensic (AF) diffuser using the hashing method.

The resulting MK can be validated with the master key validation hash stored in the volume header
(format version 1, fixed 20 bytes) or a JSON [digest object](#metadata_digests_object) (format
version 2, variable size matching the hashing method). The validation hash can be calculated using
the PBKDF2 algorithm with:

* The [hashing method](#hashing_method) stored in the volume header (format version 1) or digest
  object (format version 2).
* The number of iterations as stored in the volume header (format version 1) or digest object
  (format version 2).
* A salt, as stored in the volume header (format version 1) or Base64-encoded in the digest object
  (format version 2).
* The master key as the input data.

### User Key (UK)

The User Key (UK) is derived from the user password. The UK is used to de/encrypt the corresponding
split master key material.

In format version 1, the user key is calculated using the PBKDF2 algorithm with:

* The [hashing method](#hashing_method) stored in the volume header.
* The number of iterations as stored in the corresponding key slot.
* A salt, as stored in the corresponding key slot.
* The password string as the input data (bytes).
* An output key size that is the same as that of the Master Key (MK).

In format version 2, each keyslot specifies its own key derivation function in the
[key derivation object](#metadata_keyslot_kdf_object), which can be PBKDF2, Argon2i, or Argon2id
(RFC 9106, version 0x13), with an output key size defined by the keyslot "area.key_size" property.

## Encryption methods

LUKS supports multiple encryption methods, different encryption chaining modes and initialization
vector modes.

### Initialization vector modes

#### The null initialization vector mode

In the null initialization vector mode the initialization vector (IV) is filled with 0‑byte values.

#### The plain initialization vector modes

In the plain and plain64 initialization vector mode the initialization vector (IV) is filled with
respectively a 32-bit or 64-bit little-endian representation of the corresponding sector number
padded with 0-byte values.

The sector number is relative to the start of the data not relative to the start of the volume
header.

#### The encrypted sector-salt initialization vector (ESSIV) mode

In the encrypted sector-salt initialization vector (ESSIV) mode the initialization vector (IV) is
determined by:

1. hashing the encryption key with hashing method defined in the initialization vector mode options.
1. encrypting the little-endian representation of the corresponding sector number padded with
   0-byte values with the hash of the encryption key.

> Note that the sector number is relative to the start of the data not relative to the start of the
> volume header.

#### The benbi initialization vector mode

In the benbi initialization vector mode the initialization vector (IV) is filled with a 64-bit
big-endian representation of the corresponding cipher block (or narrow block)-count (starting at 1)
padded with 0-byte values.

The sector number is relative to the start of the data not relative to the start of the volume
header.

The cipher block-count is calculated as:

```python
cipher_block_count = (sector_number << (log2(bytes_per_sector) - log2(iv_size))) + 1
```

Benbi is presumably the abbreviation of big-endian numeric block index, or equivalent.

#### The lmk initialization vector mode

TODO: complete section

### AES-CBC

Decryption uses:

* AES-CBC with Master Key (MK) decryption of sector data
* The initialization vector of the AES-CBC is dependent on the initialization vector mode defined
  in the volume header. In recent versions of Linux, AES-CBC is combined with the ESSIV
  initialization vector mode by default.
* The initialization vector is 16 bytes in size.

### AES-ECB

Decryption uses:

* AES-ECB with Master Key (MK) decryption of sector data
* No initialization vector is used.
* The initialization vector is 16 bytes in size.

### AES-XTS

Decryption uses:

* AES-XTS (XEX-based tweaked-codebook mode with ciphertext stealing) with the Master Key (MK) split
  into two equal halves: the first half is the data encryption key and the second half is the tweak
  encryption key (for example, a 64-byte MK provides two 256-bit AES keys, displayed as
  AES-512-XTS).
* The 16-byte initialization vector (tweak) is derived from the sector number using the configured
  initialization vector mode (commonly "plain64").
* AES-XTS is the default encryption mode in LUKS version 2.

### Anubis

TODO: complete section

Default encryption mode is cbc-plain
Size of initialization vector?

### Blowfish

TODO: complete section

Default encryption mode is cbc-plain
Size of initialization vector?

### Cast5

TODO: complete section

RFC 2144
Size of initialization vector?

### Cast6

TODO: complete section

RFC 2612
Default encryption mode is cbc-plain
Size of initialization vector?

### Serpent

TODO: complete section

Default encryption mode is cbc-plain
Size of initialization vector?

### Twofish

TODO: complete section

Default encryption mode is cbc-plain
Size of initialization vector?

## Volume header

### Volume header - format version 1 {#volume_header_v1}

The volume header - format version 1 is 4096 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 6 | "LUKS\xba\xbe" | Signature |
| 6 | 2 | 1 | Format version |
| 8 | 32 | | [Encryption method](#encryption_method) (Cipher name), which contains an ASCII string with an end-of-string character |
| 40 | 32 | | [Encryption mode](#encryption_mode) (Cipher mode), which contains an ASCII string with an end-of-string character |
| 72 | 32 | | [Hashing method](#hashing_method), which contains an ASCII string with an end-of-string character |
| 104 | 4 | | Encrypted data start sector |
| 108 | 4 | | Master key size, in number of bytes |
| 112 | 20 | | Master key validation hash |
| 132 | 32 | | Master key derivation salt |
| 164 | 4 | | Master key derivation number of iterations |
| 168 | 40 | | Volume identifier, which contains an ASCII string with an end-of-string character that consists of a lower-case UUID |
| 208 | 8 x 48 | | Array of [key slots](#key_slot_v1) |
| 592 | 3504 | | Unknown (empty values) |

The hashing method is used for the user key calculation and the anti-forensic (AF) diffuser.

### Volume header - format version 2 {#volume_header_v2}

The volume header - format version 2 (or binary header) is 4096 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 6 | "LUKS\xba\xbe" | Signature |
| 6 | 2 | 2 | Format version |
| 8 | 8 | | Metadata area size, which consists of the size of the volume header and JSON area |
| 16 | 8 | | Epoch (or sequence identifier) |
| 24 | 48 | | Volume label, which contains an ASCII string with an end-of-string character |
| 72 | 32 | | Metadata area checksum method (or algorithm), which contains an ASCII string with an end-of-string character |
| 104 | 64 | | Salt |
| 168 | 40 | | Volume identifier, which contains an ASCII string with an end-of-string character that consists of a lower-case UUID |
| 208 | 48 | | Unknown (subsystem), which contains an ASCII string with an end-of-string character |
| 256 | 8 | | Metadata area offset, which is relative from the start of the storage media in which the LUKS volume is stored |
| 264 | 184 | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |
| 448 | 64 | | Metadata area checksum |
| 512 | 7 x 512 = 3584 | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |

The metadata area checksum is calculated over the entire metadata area (the 4096-byte binary header
with the 64-byte checksum field at offset 448 filled with 0-byte values, followed by the JSON area
of `metadata_area_size - 4096` bytes) using the hashing algorithm specified in the metadata area
checksum method field. The resulting digest is stored in the first bytes of the 64-byte checksum
field, with the remaining bytes filled with 0-byte values.

When both the primary and backup metadata areas are valid, the metadata area with the higher epoch
(sequence identifier) is active. If the primary metadata area fails validation, readers fall back
to the backup metadata area.

## JSON area {#json_area}

The JSON area is stored directly after the volume header and must be 4096-byte aligned. The JSON
area is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | ... | | JSON string, which contains an ASCII string with an end-of-string character |
| ... | ... | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |

### Example

```json
{
  "keyslots": {
    "0": {
      "type": "luks2",
      "key_size": 32,
      "af": {
        "type": "luks1",
        "stripes": 4000,
        "hash": "sha1"
      },
      "area": {
        "type": "raw",
        "offset": "32768",
        "size": "131072",
        "encryption": "aes-ecb",
        "key_size": 32
      },
      "kdf": {
        "type": "argon2i",
        "time": 6,
        "memory": 1048576,
        "cpus": 4,
        "salt": "X3OghBqUPLPkYuaFlSu4w/4VsRlRNDBzN+IW5Y5JQSU="
      }
    }
  },
  "tokens": {},
  "segments": {
    "0": {
      "type": "crypt",
      "offset": "16777216",
      "size": "dynamic",
      "iv_tweak": "0",
      "encryption": "aes-ecb",
      "sector_size": 512
    }
  },
  "digests": {
    "0": {
      "type": "pbkdf2",
      "keyslots": [
        "0"
      ],
      "segments": [
        "0"
      ],
      "hash": "sha1",
      "iterations": 154931,
      "salt": "wxT97+jYHKhAat3rZb6XXuwXVRn3DM7tvGy8+ZukM38=",
      "digest": "WHT1SoOLP3tummIDhiNTxP39dfw="
    }
  },
  "config": {
    "json_size": "12288",
    "keyslots_size": "16744448"
  }
}
```

### Top level properties

| Value | Description |
| --- | --- |
| "config" | [Config object](#metadata_config_object) |
| "digests" | One or more [digests object](#metadata_digests_object) |
| "keyslots" | One or more [keyslots object](#metadata_keyslots_object) |
| "segments" | One or more [segments object](#metadata_segments_object) |
| "tokens" | Zero or more [tokens object](#metadata_tokens_object) |

### Config object {#metadata_config_object}

| Value | Description |
| --- | --- |
| "flags" | Optional list of persistent configuration flag strings |
| "json_size" | String containing the JSON area size in bytes (excluding the 4096-byte binary header) |
| "keyslots_size" | String containing the binary keyslots area size in bytes |
| "requirements" | Optional object containing a "mandatory" list of required feature strings |

### Digests object {#metadata_digests_object}

Maps numeric string identifiers (such as "0") to digest objects that verify the derived Master Key
(MK) for specific keyslots and data segments.

| Value | Description |
| --- | --- |
| "digest" | Base64-encoded expected validation digest |
| "hash" | [Hashing method](#hashing_method) used by PBKDF2 (present when "type" is "pbkdf2") |
| "iterations" | Number of PBKDF2 iterations (present when "type" is "pbkdf2") |
| "keyslots" | List of numeric keyslot identifier strings validated by this digest |
| "salt" | Base64-encoded salt |
| "segments" | List of numeric segment identifier strings whose volume key is validated by this digest |
| "type" | Digest type, such as "pbkdf2" |

### Keyslots object {#metadata_keyslots_object}

Maps numeric string identifiers (such as "0") to zero or more
[keyslot objects](#metadata_keyslot_object).

### Keyslot object {#metadata_keyslot_object}

| Value | Description |
| --- | --- |
| "af" | [Anti-forensics (diffuser) object](#metadata_keyslot_af_object), present when "type" is "luks2" |
| "area" | [Key slot area object](#metadata_keyslot_area_object) |
| "direction" | Re-encryption direction ("forward" or "backward"), present when "type" is "reencrypt" |
| "kdf" | [Key derivation object](#metadata_keyslot_kdf_object), present when "type" is "luks2" |
| "key_size" | Master key size in bytes |
| "mode" | Re-encryption mode ("reencrypt", "encrypt", or "decrypt"), present when "type" is "reencrypt" |
| "priority" | Optional unlock priority, where 0 represents "ignore", 1 "normal" (default), and 2 "high" |
| "type" | Keyslot type, which can be "luks2" or "reencrypt" |

#### Anti-forensics (diffuser) object {#metadata_keyslot_af_object}

| Value | Description |
| --- | --- |
| "hash" | [Hashing method](#hashing_method) used by the anti-forensic diffuser (for type "luks1") |
| "stripes" | Number of anti-forensic stripes (typically 4000, for type "luks1") |
| "type" | Anti-forensics type, which can be "luks1" |

#### Keyslot area object {#metadata_keyslot_area_object}

For "raw" keyslot areas, the split key material occupies `key_size * af.stripes` bytes starting at
"offset" and is always encrypted in 512-byte sectors using "encryption" and a user key of length
"key_size".

| Value | Description |
| --- | --- |
| "encryption" | Combined encryption method and mode string (such as "aes-xts-plain64"), for type "raw" |
| "hash" | Hashing method, for types "checksum" and "datashift-checksum" |
| "key_size" | User key size in bytes used to encrypt the keyslot area, for type "raw" |
| "offset" | String containing the byte offset of the keyslot area from the start of the volume |
| "sector_size" | Sector size in bytes, for types "checksum" and "datashift-checksum" |
| "shift_size" | Shift size in bytes, for types "datashift", "datashift-checksum", and "datashift-journal" |
| "size" | String containing the reserved size of the keyslot area in bytes |
| "type" | Area type, which can be "raw", "checksum", "none", "journal", "datashift", "datashift-checksum", or "datashift-journal" |

#### Key derivation object {#metadata_keyslot_kdf_object}

| Value | Description |
| --- | --- |
| "cpus" | Parallelism (number of threads/lanes), for types "argon2i" and "argon2id" |
| "hash" | [Hashing method](#hashing_method), for type "pbkdf2" |
| "iterations" | Number of PBKDF2 iterations, for type "pbkdf2" |
| "memory" | Memory cost in KiB, for types "argon2i" and "argon2id" |
| "salt" | Base64-encoded salt |
| "time" | Time cost (number of passes/iterations), for types "argon2i" and "argon2id" |
| "type" | Key derivation function type, which can be "pbkdf2", "argon2i", or "argon2id" |

### Segments object {#metadata_segments_object}

Maps numeric string identifiers (such as "0") to segment objects describing the encrypted or
unencrypted data regions of the volume.

| Value | Description |
| --- | --- |
| "encryption" | Combined encryption method and mode string (such as "aes-xts-plain64"), for type "crypt" |
| "flags" | Optional list of segment flag strings |
| "integrity" | Optional [segment integrity object](#metadata_segment_integrity_object), for type "crypt" |
| "iv_tweak" | String containing the starting 64-bit sector number tweak added to each sector's IV, for type "crypt" |
| "offset" | String containing the byte offset of the segment from the start of the volume |
| "sector_size" | Encryption sector size in bytes (512, 1024, 2048, or 4096), for type "crypt" |
| "size" | String containing the segment size in bytes, or "dynamic" to extend to the end of the volume |
| "type" | Segment type, which can be "linear" or "crypt" |

#### Segment integrity object {#metadata_segment_integrity_object}

| Value | Description |
| --- | --- |
| "journal_encryption" | Journal encryption algorithm (or "none") |
| "journal_integrity" | Journal integrity algorithm (or "none") |
| "key_size" | Integrity key size in bytes |
| "type" | Authenticated encryption / integrity algorithm (such as "hmac(sha256)") |

### Tokens object {#metadata_tokens_object}

Maps numeric string identifiers (such as "0") to optional token objects that describe external key
retrieval mechanisms (for example, kernel keyring or hardware tokens).

| Value | Description |
| --- | --- |
| "key_description" | Kernel keyring key description, for type "luks2-keyring" |
| "keyslots" | List of numeric keyslot identifier strings associated with this token |
| "type" | Token type, such as "luks2-keyring" |

## Backup metadata area

To make recovery easier the backup metadata area starts at a fixed offset:

| Offset | Maximum JSON area size |
| --- | --- |
| 16384 (0x004000) | 12 KiB |
| 32768 (0x008000) | 28 KiB |
| 65536 (0x010000) | 60 KiB |
| 131072 (0x020000) | 124 KiB |
| 262144 (0x040000) | 252 KiB |
| 524288 (0x080000) | 508 KiB |
| 1048576 (0x100000) | 1020 KiB |
| 2097152 (0x200000) | 2044 KiB |
| 4194304 (0x400000) | 4092 KiB |

### Backup volume header - format version 2

The backup (or secondary) volume header - format version 2 is the same as the
[Volume header - format version 2](#volume_header_v2) with a different signature: "SKUL\xba\xbe".

## Keyslots

### Key slot - format version 1 {#key_slot_v1}

The key slot - format version 1 is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | State (of key slot), where 0x0000dead represents inactive (dead) and 0x00ac71f3 represents active |
| 4 | 4 | | Key material number of iterations |
| 8 | 32 | | Key material salt |
| 40 | 4 | | Key material start sector |
| 44 | 4 | | Key material number of (anti-forensic) stripes |

## String identifiers

LUKS uses various string values as identifiers.

> Note that it is assumed that these identifiers are case insensitive.

### Encryption method {#encryption_method}

Known values of encryption method are:

| Value | Description |
| --- | --- |
| arc4 | Alleged RC4 (ARC4) |
| aes | Advanced Encryption Standard (AES) |
| anubis | Anubis |
| blowfish | Blowfish |
| cast5 | Cast5 (RFC 2144) |
| cast6 | Cast6 (RFC 2612) |
| serpent | Serpent |
| tnepres | Reversed variant of Serpent |
| twofish | Twofish |

### Encryption mode {#encryption_mode}

The encryption mode consists of a string in the format:

```text
chaining_mode[-initialization_vector_mode[:initialization_vector_options]]
```

#### Chaining mode

Known values of chaining mode are:

| Value | Description |
| --- | --- |
| cbc | Cipher-block chaining (CBC) |
| ecb | Electronic codebook (ECB), which should not have a initialization vector mode set |
| xts | XEX-based tweaked-codebook mode with ciphertext stealing (XTS) |

> Note that it is assumed that these identifiers are case insensitive.

TODO: determine ctr and lrw

#### Initialization vector mode and options

Known values of initialization vector mode and options are:

| Value | Description |
| --- | --- |
| benbi | The initialization vector is the 64-bit big-endian cipher block (or narrow block)-count (starting at 1) |
| essiv | Encrypted sector-salt initialization vector (ESSIV). The "essiv" initialization vector mode requires a hash algorithm to be defined as an initialization vector option. This is specified in the form "essiv:hash", e.g. "essiv:sha256" |
| lmk | Compatible implementation of the block chaining mode used by the Loop-AES block device encryption system |
| null | The initialization vector is always zero |
| plain | The initialization vector is the 32-bit little-endian version of the sector number, padded with zeros if necessary |
| plain64 | The initialization vector is the 64-bit little-endian version of the sector number, padded with zeros if necessary |
| plumb | Unknown |

### Hashing method {#hashing_method}

Known values of hashing method are:

| Value | Description |
| --- | --- |
| ripemd160 | RIPEMD-160 |
| sha1 | SHA-1 |
| sha224 | SHA-224 |
| sha256 | SHA-256 |
| sha384 | SHA-384 |
| sha512 | SHA-512 |
| wd256 | Unknown |

The hashing method must at least produce 20 bytes of hash data. Therefore hashing methods like
ghash and MD5 are unsupported.

## Format edge cases and corruption scenarios

### Uninitialized encrypted volume data

Running "cryptsetup luksFormat" will not initialize the encrypted volume data, the data is
initialized on write. The uninitialized encrypted data is treated as-is on decryption.

## References

* [LUKS On-Disk Format Specification](https://gitlab.com/cryptsetup/cryptsetup/-/wikis/LUKS-standard/on-disk-format.pdf),
  by Clemens Fruhwirth
* [LUKS2 On-Disk Format Specification](https://gitlab.com/cryptsetup/cryptsetup/blob/master/docs/on-disk-format-luks2.pdf),
  by Milan Broz
