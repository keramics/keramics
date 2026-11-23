# luksde

The luksde module provides read-only support for the
[Linux Unified Key Setup (LUKS) Disk Encryption](https://keramics.github.io/luksde.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1, 2 |
| Encryption methods | AES (AES-CBC, AES-ECB, AES-XTS) |
| Initialization Vector (IV) modes | benbi, ESSIV (SHA-1, SHA-224, SHA-256), null, plain, plain64 |
| Key derivation methods | Argon2 (argon2i, argon2id), PBKDF2 |
| Hashing methods | SHA-1, SHA-224, SHA-256, SHA-384, SHA-512 |
| Unlock credentials | Key data (master key), Passphrase |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Encryption methods | anubis, ARC4 (ARC4-CBC, ARC4-ECB), Blowfish (Blowfish-CBC, Blowfish-ECB), cast5, cast6, Serpent (Serpent-CBC, Serpent-ECB), twofish |
| Initialization Vector (IV) modes | lmk, plumb |
| Hashing methods | RIPEMD-160 |
| LUKS2 features | Authenticated encryption (integrity), linear segments, online re-encryption, tokens |
