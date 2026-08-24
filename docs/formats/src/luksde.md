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

* volume header
* 8 x key slots
* split master key material
* encrypted (volume) data

The total reserved size of the LUKS metadata (volume header and split master key material) seems to
commonly be 2 MiB.

The number of bytes per sector is 512.

### Layout version 2

A LUKS version 2 encrypted volume consist of:

* metadata area
  * volume header
  * JSON area
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
the master key size value in the volume header. Commonly the MK is 128-bit or 256-bit of size. The
MK is used to de/encrypt the encrypted (volume) data.

### Split Master Key (SMK)

The Split Master Key (SMK) is stored encrypted with a specific user key (UK) in the split master
key material. The size of the key material and hence the SMK is the size of the Master Key (MK)
times the number of stripes.

The MK is determined from the SMK using the anti-forensic (AF) diffuser using the hashing method.

The resulting MK can be validated with the master key validation hash stored in the volume header.
The validation hash can be calculated using the PBKDF2 algorithm with:

* The [hashing method](#hashing_method) stored in the volume header (format version 1) or metadata
  (format version 2).
* The number of iterations as stored in the volume header.
* A salt, as stored in the volume header.
* The master key as the input data.

### User Key (UK)

The User Key (UK) is derived from the user password. The UK is used to de/encrypt the corresponding
split master key material.

The user key is calculated using the PBKDF2 algorithm with:

* The [hashing method](#hashing_method) stored in the volume header (format version 1) or metadata
  (format version 2).
* The number of iterations as stored in the corresponding key slot.
* A salt, as stored in the corresponding key slot.
* The password string as the input data (bytes).
* A (output) key size that is the same as that of the Master Key (MK).

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

Int the encrypted sector-salt initialization vector (ESSIV) mode the initialization vector (IV) is
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
cipher_block_count = (sector_number << (log2(bytes_per_sectory) - log2(iv_size))) + 1
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
* The initialization vector is 16 bytes of size.

### AES-ECB

Decryption uses:

* AES-ECB with Master Key (MK) decryption of sector data
* No initialization vector is used.
* The initialization vector is 16 bytes of size.

### AES-XTS

TODO: complete section

* The initialization vector is 16 bytes of size.

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

### Volume header - format version 1

The volume header is 4096 bytes in size and consists of:

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
| 208 | 8 x 48 | | Array of [key slots](#key_slot) |
| 592 | 3504 | | Unknown (empty values) |

The hashing method is used for the user key calculation and the anti-forensic (AF) diffuser.

### Volume header - format version 2 {#volume_header_v2}

The volume header (or binary header) is 4096 bytes in size and consists of:

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
| 256 | 8 | | Header offset, which is relative from the start of the volume |
| 264 | 184 | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |
| 448 | 64 | | Metadata area checksum |
| 512 | 7 x 512 = 3584 | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |

### JSON area

The JSON area is stored directly after the volume header and must be 4096-byte aligned. The JSON
area is variable of size and constists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | ... | | JSON string, which contains an ASCII string with an end-of-string character |
| ... | ... | | Unknown (padding), which according to "LUKS2 On-Disk Format Specification" this must be filled with 0-byte values |

#### Example

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

#### Top level properties

| Value | Description |
| --- | --- |
| "config" | [Config object](#metadata_config_object) |
| "digests" | [Digests object](#metadata_digests_object) |
| "keyslots" | [Keyslots object](#metadata_keyslots_object) |
| "segments" | [Segments object](#metadata_segments_object) |
| "tokens" | [Tokens object](#metadata_tokens_object) |

#### Config object {#metadata_config_object}

| Value | Description |
| --- | --- |
| "flags" | List of strings |
| "json_size" | String containing an integer |
| "keyslots_size" | String containing an integer |

#### Digests object {#metadata_digests_object}

TODO: complete section

#### Keyslots object {#metadata_keyslots_object}

Contains zero or more [keyslot object](#metadata_keyslot_object).

#### Keyslot object {#metadata_keyslot_object}

TODO: complete section

| Value | Description |
| --- | --- |
| "af" | Anti-forensics (diffuser) object |
| "area" | |
| "kdf" | Key derivation object |
| "key_size" | |
| "priority" | |
| "type" | |

#### Segments object {#metadata_segments_object}

TODO: complete section

#### Tokens object {#metadata_tokens_object}

TODO: complete section

### Backup metadata area

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

#### Backup volume header - format version 2

The backup (or secondary) volume header - format version 2 is the same as the
[Volume header - format version 2](#volume_header_v2) with a different signature: "SKUL\xba\xbe".

### Keyslots area

TODO: complete section

### Encryption method {#encryption_method}

The encryption mode consists of a string in the form:

```text
cipher
```

Where known values of cipher are:

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

> Note that it is assumed that these identifiers are case insensitive.

### Encryption mode {#encryption_mode}

The encryption mode consists of a string in the form:

```text
chaining_mode[-initialization_vector_mode[:initialization_vector_options]]
```

Where known values of chaining mode are:

| Value | Description |
| --- | --- |
| cbc | Cipher-block chaining (CBC) |
| ecb | Electronic codebook (ECB), which should not have a initialization vector mode set |
| xts | XEX-based tweaked-codebook mode with ciphertext stealing (XTS) |

> Note that it is assumed that these identifiers are case insensitive.

TODO: determine ctr and lrw

And known values of initialization vector mode are:

| Value | Description |
| --- | --- |
| benbi | The initialization vector is the 64-bit big-endian cipher block (or narrow block)-count (starting at 1) |
| essiv | Encrypted sector-salt initialization vector (ESSIV). The "essiv" initialization vector mode requires a hash algorithm to be defined as an initialization vector option. This is specified in the form "essiv:hash", e.g. "essiv:sha256" |
| lmk | Compatible implementation of the block chaining mode used by the Loop-AES block device encryption system |
| null | The initialization vector is always zero |
| plain | The initialization vector is the 32-bit little-endian version of the sector number, padded with zeros if necessary |
| plain64 | The initialization vector is the 64-bit little-endian version of the sector number, padded with zeros if necessary |
| plumb | Unknown |

> Note that it is assumed that these identifiers are case insensitive.

### Hashing method {#hashing_method}

| Value | Description |
| --- | --- |
| ripemd160 | RIPEMD-160 |
| sha1 | SHA-1 |
| sha224 | SHA-224 |
| sha256 | SHA-256 |
| sha512 | SHA-512 |
| wd256 | Unknown |

> Note that it is assumed that these identifiers are case insensitive.

The hashing method must at least produce 20 bytes of hash data. Therefore hashing methods like:
ghash, MD5 are unsupported.

### Key slot {#key_slot}

The key slot is 48 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | State (of key slot), where 0x0000dead represents inactive (dead) and 0x00ac71f3 represents active |
| 4 | 4 | | Key material number of iterations |
| 8 | 32 | | Key material salt |
| 40 | 4 | | Key material start sector |
| 44 | 4 | | Key material number of (anti-forensic) stripes |

## Format edge cases and corruption scenarios

### Uninitialized encrypted volume data

Running "cryptsetup luksFormat" will not initialize the encrypted volume data, the data is
initialized on write. The uninitialized encrypted data is treated as-is on decryption.

## References

* [LUKS On-Disk Format Specification](https://gitlab.com/cryptsetup/cryptsetup/-/wikis/LUKS-standard/on-disk-format.pdf),
  by Clemens Fruhwirth
* [LUKS2 On-Disk Format Specification](https://gitlab.com/cryptsetup/cryptsetup/blob/master/docs/on-disk-format-luks2.pdf),
  by Milan Broz
