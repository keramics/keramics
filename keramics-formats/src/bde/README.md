# bde

The bde module provides read-only support for the
[BitLocker Drive Encryption (BDE)](https://keramics.github.io/bde.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1 (Windows Vista), 2 (Windows 7 and later), To Go |
| Encryption methods | AES-CBC, AES-XTS |
| Unlock credentials | Passphrase (password) |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Encryption methods | AES-CBC with Elephant Diffuser |
| Unlock credentials | clear key, recovery password, external key (start-up or recovery key), FKEV and/or TWEAK key data, SID-based, TPM |
| | Used Disk Space Only encryption |
