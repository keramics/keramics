# qcow

The qcow module provides read-only support for the
[QEMU Copy-On-Write (QCOW) image file format](https://keramics.github.io/qcow.html).

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1, 2, 3 |
| Image types | Differential (backing file), Dynamic-size |
| Compression | zlib |
| Encryption | AES-128-CBC |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Incompatible feature flags | QCOW2_INCOMPAT_CORRUPT, QCOW2_INCOMPAT_DATA_FILE, QCOW2_INCOMPAT_COMPRESSION, QCOW2_INCOMPAT_EXTL2 |
| Compression | zstd |
| Encryption | Linux Unified Key Setup (LUKS) |
| | Snapshots |
