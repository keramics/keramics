# decmpfs

The decmpfs module provides read-only support for the
[Apple File System Compression (decmpfs)](https://keramics.github.io/decmpfs.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Compression | LZFSE (methods 11 and 12), LZVN (methods 7 and 8), "raw" (methods 9 and 10), zlib (methods 3 and 4) |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Compression | LZBitmap (methods 13 and 14) |
