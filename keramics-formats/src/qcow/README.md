# qcow

The qcow module provides read-only support for the
[QEMU Copy-On-Write (QCOW) image file format](https://keramics.github.io/qcow.html).

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1, 2, 3 |
| Image types | Differential (backing file), Dynamic-size |
| Compression | zlib |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Compression | zstd |
| Encryption | AES-128-CBC, LUKS |
| | Snapshots |
| | Data file |
| | Extended L2 (table) entries |
