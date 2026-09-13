# keramics-encryption

Encryption support for Keramics.

Cryptographic functions provided by this project are primarily intended for data format analysis,
and are **not** intended for other purposes.

[docs.rs](https://docs.rs/keramics_encryption)

Supported decryption methods:

* AES (CBC, CCM, ECB and XTS)
* Blowfish (CBC and ECB)
* DES3 (CBC and ECB)
* RC4

Supported Message Authentication Code (HMAC) methods:

* HMAC-MD5
* HMAC-SHA-1
* HMAC-SHA-2 (HMAC-SHA-224, HMAC-SHA-256, HMAC-SHA-384, HMAC-SHA-512)

Supported key derivation methods:

* PBKDF2-HMAC-SHA1
* PBKDF2-HMAC-SHA2 (PBKDF2-HMAC-SHA-224, PBKDF2-HMAC-SHA-256, PBKDF2-HMAC-SHA-384,
  PBKDF2-HMAC-SHA-512)

Supported other cryptographic methods:

* AES key wrap
* PKCS7

## License

Licensed under [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
