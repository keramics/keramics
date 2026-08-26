# apfs

The apfs module provides read-only support for the
[Apple File System (APFS)](https://keramics.github.io/apfs.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 2 |
| Compression | [decmpfs](https://github.com/keramics/keramics/tree/main/keramics-formats/src/decmpfs/README.md) LZFSE (methods 11 and 12), LZVN (methods 7 and 8), "raw" (methods 9 and 10), zlib (methods 3 and 4) |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 1 |
| Compression | [decmpfs](https://github.com/keramics/keramics/tree/main/keramics-formats/src/decmpfs/README.md) LZBitmap (methods 13 and 14) |
| Encryption | Software and hardware-backed (T2) |
| | Fusion drive (NX_INCOMPAT_FUSION) |
| | Snapshots |
