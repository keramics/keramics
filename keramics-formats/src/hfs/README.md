# hfs

The hfs module provides read-only support for the
[Hierarchical File System (HFS)](https://keramics.github.io/hfs.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | HFS (standard), HFS+ (extended), HFSX |
| Encodings | MacRoman for HFS, Unicode 3.2 (Mac OS 10.3 and later) for HFS+/HFSX |
| Compression | [decmpfs](https://github.com/keramics/keramics/tree/main/keramics-formats/src/decmpfs/README.md) LZFSE (methods 11 and 12), LZVN (methods 7 and 8), "raw" (methods 9 and 10), zlib (methods 3 and 4) |
| | HFS-wrapped HFS+ |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Encodings | Non-MacRoman for HFS, Unicode 2.1 (Mac OS 8.1 through 10.2) for HFS+/HFSX |
| Compression | [decmpfs](https://github.com/keramics/keramics/tree/main/keramics-formats/src/decmpfs/README.md) LZBitmap (methods 13 and 14) |
