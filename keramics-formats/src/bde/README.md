# bde

The bde module provides read-only support for the
[BitLocker Drive Encryption (BDE)](https://keramics.github.io/bde.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1.0 (Windows Vista), 2.0 (Windows 7 and later), To Go, Used Disk Space Only encryption |
| Encryption methods | AES-CBC, AES-CBC with Elephant Diffuser, AES-XTS |
| Unlock credentials | Passphrase (password), Recovery password |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Unlock credentials | clear key, external key (start-up or recovery key), FKEV and/or TWEAK key data, SID-based, TPM |
| | Partial encrypted volumes (pre Used Disk Space Only encryption) |
