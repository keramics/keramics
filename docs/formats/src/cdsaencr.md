# Mac OS Encrypted Encoding

Mac OS uses encrypted encoding (CEncryptedEncoding) to encrypt various formats, such as:

* [Mac OS sparse bundle](sparsebundle.md)
* [Mac OS sparse image](sparseimage.md)
* [Universal Disk Image Format (UDIF)](udif.md)

## Overview

There are 2 known versions of Encrypted Encoding.

### Encrypted Encoding version 1

A version 1 encrypted container consist of:

* Encrypted data
* [Encrypted container footer](#encypted_container_footer) at the end of the file

Format version 1 supports the following key protectors:

* Passphrase

### Encrypted Encoding version 2

A version 2 encrypted container consist of:

* [Encrypted container header](#encypted_container_header) at the start of the file
* Key protectors
* Unknown (empty values), probably reserved for the key protectors
* Encrypted data, typically at offset 122368

Version 2 supports the following key protectors:

* Passphrase
* Public key
* Unknown (keybag)

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |
| Date and time values | N/A |
| Character strings | N/A |

## Encrypted container

### Encrypted container footer {#encypted_container_footer}

The encrypted container footer is 1276 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 16 | | Container identifier (UUID), used in Mac OS keychain as account identifier |
| 16 | 4 | | Block size, in number of bytes |
| 20 | 4 | | Key protector [encryption method](#algorithm_identifiers) |
| 24 | 4 | | Key protector [padding type](#padding_types) |
| 28 | 4 | | Key protector [encryption mode](#encryption_modes) |
| 32 | 4 | | Key protector key size, in number of bits |
| 36 | 4 | | Key protector initialization vector size |
| 40 | 4 | | [Key derivation method](#algorithm_identifiers) |
| 44 | 4 | | Unknown |
| 48 | 4 | | Key derivation number of iterations |
| 52 | 4 | | Key derivation salt size, in number of bytes |
| 56 | 32 | | Key derivation salt |
| 88 | 4 | | Block initialization vector size |
| 92 | 4 | | Block [encryption mode](#encryption_modes) |
| 96 | 4 | | Block [encryption method](#algorithm_identifiers) |
| 100 | 4 | | Block key size, in number of bits |
| 104 | 32 | | Unknown (Wrapped block (or master) data encryption key (DEK) initialization vector?) |
| 136 | 4 | | Wrapped block (or master) data encryption key (DEK) size |
| 140 | 256 | | Wrapped block (or master) data encryption key (DEK) |
| 396 | 4 | | [HMAC method](#algorithm_identifiers) |
| 400 | 4 | | HMAC key size, in number of bits |
| 404 | 32 | | Unknown (Wrapped block HMAC initialization vector?) |
| 436 | 4 | | Wrapped block HMAC key size |
| 440 | 256 | | Wrapped block HMAC key |
| 696 | 4 | | Integrity [encryption method](#algorithm_identifiers) |
| 700 | 4 | | Integrity key size, in number of bits |
| 704 | 32 | | Unknown (Wrapped integrity key initialization vector?) |
| 736 | 4 | | Wrapped integrity key size |
| 740 | 256 | | Wrapped integrity key |
| 996 | 4 | | Unknown (data size) |
| 1000 | 256 | | Unknown (data) |
| 1256 | 4 | | Data fork offset, where the offset is relative from the start of the container |
| 1260 | 4 | | Data fork size, in number of bytes |
| 1264 | 4 | 1 | Encrypted Encoding format version |
| 1268 | 8 | "cdsaencr" | Signature |

> Note that "cdsaencr" presumably is short for Common Data Security Architecture (CDSA) encryption.
> Common Security Services Manager (CSSM) is part of CDSA.

Key data can be obtained from the wrapped key data using the following approach (presumably based
on RFC 3537):

* Use the specified key derivation method, e.g. PDBKDF2, with salt and number of iterations to
  determine the key encryption key (KEK) based on a passphrase.
* Pad the initialization vector [0x4a, 0xdd, 0xa2, 0x2c, 0x79, 0xe8, 0x21, 0x05] with 0-byte values
  if necessesary, e.g. if initialization vector is 8 bytes but the encryption method (AES) requires
  an initialization vector of 16 bytes.
* Decrypt the wrapped key data using the encryption method and mode, e.g. DES3-CBC, with the number
  of bits of the KEK (defined by encryption key size) and the initialization vector if applicable.
* Remove the padding, specified by the padding type.
* Reverse the resulting intermediate key data.

The intermediate key data is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | | Initialization vector |
| 8 | ... | | Wrapped key data |

* Pad the initialization vector of the intermediate key data with 0-byte values if necessesary.
* Decrypt the wrapped key data (of the intermediate key data) using the encryption method and mode,
  e.g. DES3-CBC, with the number of bits of the KEK (defined by encryption key size) and the
  initialization vector (of the intermediate key data) if applicable.
* Remove the padding, specified by the padding type.

The decypted key data is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | 0 | Signature |
| 4 | ... | | Key data |

### Encrypted container header {#encypted_container_header}

The encrypted container header is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 8 | "encrcdsa" | Signature |
| 8 | 4 | 2 | Encrypted Encrypted Encoding format version |
| 12 | 4 | | Block initialization vector size, in number of bytes |
| 16 | 4 | | Block [encryption mode](#encryption_modes) |
| 20 | 4 | | Block [encryption method](#algorithm_identifiers) |
| 24 | 4 | | Block key size, in number of bits |
| 28 | 4 | | [HMAC method](#algorithm_identifiers) |
| 32 | 4 | | HMAC key size, in number of bits |
| 36 | 16 | | Container identifier (UUID), used in Mac OS keychain as account identifier |
| 52 | 4 | | Block size, in number of bytes |
| 56 | 8 | | Data fork size, in number of bytes |
| 64 | 8 | | Data fork offset, where the offset is relative from the start of the container |
| 72 | 4 | | Number of key protector descriptors |
| 76 | ... | | Array of [key protector descriptors](#encrypted_key_protector_descriptor) |

#### Key protector descriptor {#encrypted_key_protector_descriptor}

The key protector descriptor is 20 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Unlock type](#encrypted_unlock_types) |
| 4 | 8 | | Data offset, where the offset is relative from the start of the container |
| 12 | 8 | | Data size |

#### Unlock types {#encrypted_unlock_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 1 | CSSM_APPLE_UNLOCK_TYPE_KEY_DIRECT | Master key wrapped by passphrase, stored as [passphrase wrapped key](#passphrase_wrapped_key) |
| 2 | CSSM_APPLE_UNLOCK_TYPE_WRAPPED_PRIVATE | Master key wrapped by a public key, stored as [public key wrapped key](#public_key_wrapped_key) |
| 3 | CSSM_APPLE_UNLOCK_TYPE_KEYBAG | Master key wrapped by keybag |

#### Passphrase wrapped key {#passphrase_wrapped_key}

The passphrase wrapped key is 616 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | [Key derivation method](#algorithm_identifiers) |
| 4 | 8 | | Key derivation number of iterations |
| 12 | 4 | | Key derivation salt size, in number of bytes |
| 16 | 32 | | Key derivation salt |
| 48 | 4 | | Encryption initialization vector size, in number of bytes |
| 52 | 32 | | Encryption initialization vector |
| 84 | 4 | | Encryption key size, in number of bits |
| 88 | 4 | | [Encryption method](#algorithm_identifiers) |
| 92 | 4 | | [Padding type](#padding_types) |
| 96 | 4 | | [Encryption mode](#encryption_modes) |
| 100 | 4 | | Wrapped key data size |
| 104 | 64 | | Wrapped key data |
| 168 | 448 | | Unknown (empty values) |

Key data can be obtained from the wrapped key data using the following approach:

* Use the specified key derivation method, e.g. PDBKDF2, with salt and number of iterations to
  determine the key encryption key (KEK) based on a passphrase.
* Pad the initialization vector with 0-byte values if necessesary, e.g. if initialization vector
  is 8 bytes but the encryption method (AES) requires an initialization vector of 16 bytes.
* Decrypt the wrapped key data using the encryption method and mode, e.g. DES3-CBC, with the number
  of bits of the KEK (defined by encryption key size) and the initialization vector if applicable.
* Remove the padding, specified by the padding type.

The decypted key data is of variable size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | ... | | Block (or master) data encryption key (DEK) |
| ... | ... | | Block HMAC key |
| ... | 5 | "CKIE\x00" | Signature |

#### Public key wrapped key {#public_key_wrapped_key}

TODO: complete section

The public key wrapped key is 564 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 4 | | Public key hash size |
| 4 | 20 | | Public key hash |
| 24 | 4 | | Unknown |
| 28 | 4 | | Unknown |
| 32 | 4 | | Unknown |
| 36 | 4 | | Unknown (encryption method?) |
| 40 | 4 | | Unknown |
| 44 | 4 | | Unknown |
| 48 | 4 | | Unknown (size) |
| 52 | 256 | | Unknown |
| 308 | 256 | | Unknown (empty values) |

#### Keybag wrapped key {#keybag_wrapped_key}

TODO: complete section

### Algorithm identifiers {#algorithm_identifiers}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | CSSM_ALGID_NONE | No algorithm (none) |
| 1 | CSSM_ALGID_CUSTOM | Custom algorithm |
| 2 | CSSM_ALGID_DH | Diffie Hellman key exchange |
| 3 | CSSM_ALGID_PH | Pohlig Hellman key exchange |
| 4 | CSSM_ALGID_KEA | Key Exchange Algorithm |
| 5 | CSSM_ALGID_MD2 | MD2 |
| 6 | CSSM_ALGID_MD4 | MD4 |
| 7 | CSSM_ALGID_MD5 | MD5 |
| 8 | CSSM_ALGID_SHA1 | SHA-1 |
| 9 | CSSM_ALGID_NHASH | N-Hash |
| 10 | CSSM_ALGID_HAVAL | HAVAL |
| 11 | CSSM_ALGID_RIPEMD | RIPE-MD |
| 12 | CSSM_ALGID_IBCHASH | IBC-Hash |
| 13 | CSSM_ALGID_RIPEMAC | RIPE-MAC |
| 14 | CSSM_ALGID_DES | DES |
| 15 | CSSM_ALGID_DESX | DESX |
| 16 | CSSM_ALGID_RDES | RDES |
| 17 | CSSM_ALGID_3DES_3KEY_EDE (or CSSM_ALGID_3DES_3KEY) | Triple-DES with 3 keys applied encrypt, decrypt, encrypt (EDE) |
| 18 | CSSM_ALGID_3DES_2KEY_EDE (or CSSM_ALGID_3DES_2KEY) | Triple-DES with 2 keys applied encrypt, decrypt, encrypt (EDE), with the first key used for the first and last operation |
| 19 | CSSM_ALGID_3DES_1KEY_EEE | Triple-DES with 1 keys applied encrypt, encrypt, encrypt (EEE), with the first key used for all operation |
| 20 | CSSM_ALGID_3DES_3KEY_EEE | Triple-DES with 3 keys applied encrypt, encrypt, encrypt (EEE) |
| 21 | CSSM_ALGID_3DES_2KEY_EEE | Triple-DES with 2 keys applied encrypt, encrypt, encrypt (EEE), with the first key used for the first and last operation |
| 22 | CSSM_ALGID_IDEA | IDEA |
| 23 | CSSM_ALGID_RC2 | RC2 |
| 24 | CSSM_ALGID_RC5 | RC5 |
| 25 | CSSM_ALGID_RC4 | RC4 |
| 26 | CSSM_ALGID_SEAL | SEAL |
| 27 | CSSM_ALGID_CAST | CAST |
| 28 | CSSM_ALGID_BLOWFISH | Blowfish |
| 29 | CSSM_ALGID_SKIPJACK | Skipjac |
| 30 | CSSM_ALGID_LUCIFER | Lucifer |
| 31 | CSSM_ALGID_MADRYGA | Madryga |
| 32 | CSSM_ALGID_FEAL | FEAL |
| 33 | CSSM_ALGID_REDOC | REDOC 2 |
| 34 | CSSM_ALGID_REDOC3 | REDOC 3 |
| 35 | CSSM_ALGID_LOKI | LOKI |
| 36 | CSSM_ALGID_KHUFU | KHUFU |
| 37 | CSSM_ALGID_KHAFRE | KHAFRE |
| 38 | CSSM_ALGID_MMB | MMB |
| 39 | CSSM_ALGID_GOST | GOST |
| 40 | CSSM_ALGID_SAFER | SAFER (K-40, K-64, K-128) |
| 41 | CSSM_ALGID_CRAB | CRAB |
| 42 | CSSM_ALGID_RSA | RSA |
| 43 | CSSM_ALGID_DSA | DSA |
| 44 | CSSM_ALGID_MD5WithRSA | MD5/RSA |
| 45 | CSSM_ALGID_MD2WithRSA | MD2/RSA |
| 46 | CSSM_ALGID_ElGamal | ElGamal |
| 47 | CSSM_ALGID_MD2Random | MD2-based random numbers |
| 48 | CSSM_ALGID_MD5Random | MD5-based random numbers |
| 49 | CSSM_ALGID_SHARandom | SHA-based random numbers |
| 50 | CSSM_ALGID_DESRandom | DES-based random numbers |
| 51 | CSSM_ALGID_SHA1WithRSA | SHA-1/RSA |
| 52 | CSSM_ALGID_CDMF | CDMF |
| 53 | CSSM_ALGID_CAST3 | CAST3 |
| 54 | CSSM_ALGID_CAST5 | CAST5 |
| 55 | CSSM_ALGID_GenericSecret | Generic secret |
| 56 | CSSM_ALGID_ConcatBaseAndKey | Concatenate base key with key |
| 57 | CSSM_ALGID_ConcatKeyAndBase | Concatenate key with base key |
| 58 | CSSM_ALGID_ConcatBaseAndData | Concatenate base key with data |
| 59 | CSSM_ALGID_ConcatDataAndBase | Concatenate data with base key |
| 60 | CSSM_ALGID_XORBaseAndData | XOR base key with data |
| 61 | CSSM_ALGID_ExtractFromKey | Extract key from base key |
| 62 | CSSM_ALGID_SSL3PreMasterGen | SSL 3 with 48 byte pre-master key |
| 63 | CSSM_ALGID_SSL3MasterDerive | Derive an SSL 3 key from a pre-master key |
| 64 | CSSM_ALGID_SSL3KeyAndMacDerive | Derive SSL3 key and MAC |
| 65 | CSSM_ALGID_SSL3MD5_MAC | SSL 3 with MD5 MAC |
| 66 | CSSM_ALGID_SSL3SHA1_MAC | SSL 3 with SHA-1 MAC  |
| 67 | CSSM_ALGID_PKCS5_PBKDF1_MD5 | PKCS5 key derivation using PBKDF1 with MD5 |
| 68 | CSSM_ALGID_PKCS5_PBKDF1_MD2 | PKCS5 key derivation using PBKDF1 with MD2 |
| 69 | CSSM_ALGID_PKCS5_PBKDF1_SHA1 | PKCS5 key derivation using PBKDF1 with SHA-1 |
| 70 | CSSM_ALGID_WrapLynks | Spyrus LYNKS DES based wrapping scheme with checksum |
| 71 | CSSM_ALGID_WrapSET_OAEP | SET key wrapping |
| 72 | CSSM_ALGID_BATON | Fortezza BATON |
| 73 | CSSM_ALGID_ECDSA | Elliptic Curve DSA |
| 74 | CSSM_ALGID_MAYFLY | Fortezza MAYFLY |
| 75 | CSSM_ALGID_JUNIPER | Fortezza JUNIPER |
| 76 | CSSM_ALGID_FASTHASH | Fortezza FASTHASH |
| 77 | CSSM_ALGID_3DES | Generix 3DES |
| 78 | CSSM_ALGID_SSL3MD5 | SSL 3 with MD5 |
| 79 | CSSM_ALGID_SSL3SHA1 | SSL 3 with SHA-1 |
| 80 | CSSM_ALGID_FortezzaTimestamp | Fortezza with timestamp |
| 81 | CSSM_ALGID_SHA1WithDSA | SHA-1 with DSA |
| 82 | CSSM_ALGID_SHA1WithECDSA | SHA-1 with Elliptic Curve DSA |
| 83 | CSSM_ALGID_DSA_BSAFE | DSA with BSAFE Key |
| 84 | CSSM_ALGID_ECDH | Elliptic Curve DiffieHellman Key Exchange |
| 85 | CSSM_ALGID_ECMQV | Elliptic Curve MQV key exchange |
| 86 | CSSM_ALGID_PKCS12_SHA1_PBE | PKCS12 SHA-1 PBE key derivation |
| 87 | CSSM_ALGID_ECNRA | Elliptic Curve Nyberg-Rueppel |
| 88 | CSSM_ALGID_SHA1WithECNRA | SHA-1 with Elliptic Curve Nyberg-Rueppel |
| 89 | CSSM_ALGID_ECES | Elliptic Curve Encryption Scheme |
| 90 | CSSM_ALGID_ECAES | Elliptic Curve Authenticate Encryption Scheme |
| 91 | CSSM_ALGID_SHA1HMAC | SHA1-MAC |
| 92 | CSSM_ALGID_FIPS186Random | FIPS186 Random |
| 93 | CSSM_ALGID_ECC | Elliptic Curve Encryption (ECC) |
| 94 | CSSM_ALGID_MQV | Discrete-Log MQV key exchange |
| 95 | CSSM_ALGID_NRA | Discrete-Log Nyberg-Rueppel Signature scheme |
| 96 | CSSM_ALGID_IntelPlatformRandom | Intel Platform Random Number Generator |
| 97 | CSSM_ALGID_UTC | Date and time value in the form: "YYYYMMDDhhmmss" |
| 98 | CSSM_ALGID_HAVAL3 | HAVAL3 Digest |
| 99 | CSSM_ALGID_HAVAL4 | HAVAL4 Digest |
| 100 | CSSM_ALGID_HAVAL5 | HAVAL5 Digest |
| 101 | CSSM_ALGID_TIGER | TIGER Digest |
| 102 | CSSM_ALGID_MD5HMAC | HMAC-MD5 |
| 103 | CSSM_ALGID_PKCS5_PBKDF2 | PKCS5 key derivation using PBKDF2 with SHA-1 (PBKDF2-HMAC-SHA1) |
| 104 | CSSM_ALGID_RUNNING_COUNTER | Running hardware counter |
| | | |
| 0x80000000 | CSSM_ALGID_VENDOR_DEFINED | Vendor defined algorithm |
| 0x80000001 | CSSM_ALGID_AES | Advanced Encryption Standard (AES) |

### Padding types {#padding_types}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | CSSM_PADDING_NONE | No padding |
| 1 | CSSM_PADDING_CUSTOM | Unknown |
| 2 | CSSM_PADDING_ZERO | Pad with 0 |
| 3 | CSSM_PADDING_ONE | Pad with 1 |
| 4 | CSSM_PADDING_ALTERNATE | Unknown |
| 5 | CSSM_PADDING_FF | Unknown (Pad with 0xff?) |
| 6 | CSSM_PADDING_PKCS5 | Pad using Public-Key Cryptography Standard (PKCS) 5 (RFC 2898) |
| 7 | CSSM_PADDING_PKCS7 | Pad using Public-Key Cryptography Standard (PKCS) 7 (RFC 2315) |
| 8 | CSSM_PADDING_CIPHERSTEALING | Unknown |
| 9 | CSSM_PADDING_RANDOM | Unknown |
| 10 | CSSM_PADDING_PKCS1 | Pad using Public-Key Cryptography Standard (PKCS) 1 (RFC 2437) |

### Encryption modes {#encryption_modes}

| Value | Identifier | Description |
| --- | --- | --- |
| 0 | CSSM_ALGMODE_NONE | Unknown (Null algorithm mode) |
| 1 | CSSM_ALGMODE_CUSTOM | Unknown (Custom mode) |
| 2 | CSSM_ALGMODE_ECB | Electronic CodeBook (ECB) mode, without padding |
| 3 | CSSM_ALGMODE_ECBPad | Electronic CodeBook (ECB) mode with padding |
| 4 | CSSM_ALGMODE_CBC | Cipher Block Chaining (CBC) mode, without padding |
| 5 | CSSM_ALGMODE_CBC_IV8 | Cipher Block Chaining (CBC) mode with 8 byte initialization vector, without padding |
| 6 | CSSM_ALGMODE_CBCPadIV8 | Cipher Block Chaining (CBC) mode with 8 byte initialization vector, with padding |
| 7 | CSSM_ALGMODE_CFB | Cipher feedback (CFB) mode |
| 8 | CSSM_ALGMODE_CFB_IV8 | Cipher feedback (CFB) mode with 8 byte initialization vector |
| 9 | CSSM_ALGMODE_CFBPadIV8 | Cipher feedback (CFB) mode with 8 byte initialization vector, with padding |
| 10 | CSSM_ALGMODE_OFB | Output FeedBack (OFB) mode |
| 11 | CSSM_ALGMODE_OFB_IV8 | Output FeedBack (OFB) mode mode with 8 byte initialization vector |
| 12 | CSSM_ALGMODE_OFBPadIV8 | Output FeedBack (OFB) mode with 8 byte initialization vector, with padding |
| 13 | CSSM_ALGMODE_COUNTER | Counter mode |
| 14 | CSSM_ALGMODE_BC | Block Chaining mode |
| 15 | CSSM_ALGMODE_PCBC | Propagating Cipher Block Chaining (CBC) mode |
| 16 | CSSM_ALGMODE_CBCC | Cipher Block Chaining (CBC) with checksum mode |
| 17 | CSSM_ALGMODE_OFBNLF | Output FeedBack (OFB) with non-linear function mode |
| 18 | CSSM_ALGMODE_PBC | Plaintext Block Chaining (PBC) mode |
| 19 | CSSM_ALGMODE_PFB | Plaintext FeedBack (PFB) mode |
| 20 | CSSM_ALGMODE_CBCPD | Cipher Block Chaining (CBC) if Plaintext Difference mode |
| 21 | CSSM_ALGMODE_PUBLIC_KEY | Public key mode |
| 22 | CSSM_ALGMODE_PRIVATE_KEY | Private key mode |
| 23 | CSSM_ALGMODE_SHUFFLE | Fortezza shuffle mode |
| 24 | CSSM_ALGMODE_ECB64 | 64 byte Electronic CodeBook (ECB) mode |
| 25 | CSSM_ALGMODE_CBC64 | 64 byte Cipher Block Chaining (CBC) mode |
| 26 | CSSM_ALGMODE_OFB64 | 64 byte Output FeedBack (OFB) mode |
| 28 | CSSM_ALGMODE_CFB32 | 32 byte Cipher feedback (CFB) mode |
| 29 | CSSM_ALGMODE_CFB16 | 16 byte Cipher feedback (CFB) mode |
| 30 | CSSM_ALGMODE_CFB8 | 8 byte Cipher feedback (CFB) mode |
| 31 | CSSM_ALGMODE_WRAP | Unknown |
| 32 | CSSM_ALGMODE_PRIVATE_WRAP | Unknown |
| 33 | CSSM_ALGMODE_RELAYX | Unknown |
| 34 | CSSM_ALGMODE_ECB128 | 128 byte Electronic CodeBook (ECB) mode |
| 35 | CSSM_ALGMODE_ECB96 | 96 byte Electronic CodeBook (ECB) mode |
| 36 | CSSM_ALGMODE_CBC128 | 128 byte Cipher Block Chaining (CBC) mode |
| 37 | CSSM_ALGMODE_OAEP_HASH | Unknown (Algorithm mode for SET key wrapping?) |
| 38 | CSSM_ALGMODE_PKCS1_EME_V15 | Public-Key Cryptography Standard (PKCS) 1 version 1.5 |
| 39 | CSSM_ALGMODE_PKCS1_EME_OAEP | Public-Key Cryptography Standard (PKCS) 1 version 2.0 |
| 40 | CSSM_ALGMODE_PKCS1_EMSA_V15 | Unknown |
| 41 | CSSM_ALGMODE_ISO_9796 | Unknown |
| 42 | CSSM_ALGMODE_X9_31 | Unknown |

### Encrypted block data

The encrypted block data can be decrypted using the following approach:

* Calculate the specified block HMAC, e.g. HMAC-SHA-1, with the block HMAC key and the block number
  stored as a 32-bit big-endian value, where 0 represents the first block. This HMAC is used as the
  initialization vector for decryption.
* Decrypt the encrypted data using the block encryption method and mode, e.g. AES-CBC, with the
  number of bits of the block DEK (defined by the block encryption key size) and the initialization
  vector if applicable.
