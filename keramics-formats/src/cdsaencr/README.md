# cdsaencr

The cdsaencr module provides read-only support for the
[Mac OS encrypted encoding](https://keramics.github.io/cdsaencr.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1, 2 |
| Encryption methods | AES (AES-CBC), DES3 (DES3-CBC) |
| Key derivation methods | PBKDF2-SHA-1 |
| HMAC methods | MHAC-SHA-1 |
| Padding methods | Pad with 0, Pad with 1, PKCS7 |
| Unlock credentials | Passphrase |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Encryption methods | Others |
| Key derivation methods | Others |
| HMAC methods | Others |
| Padding methods | Others including: PKCS1, PKCS5 |
| Unlock credentials | Keybag, Master keys, Public key |
