# udif

The udif module provides read-only support for the
[Universal Disk Image Format (UDIF)](https://keramics.github.io/udif.html).

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format versions | 4 |
| Image types | Compressed, Encrypted, Split (or segmented), Uncompressed |
| Compression methods | ADC, bzip2, LZFSE, zlib |
| Encryption | [Mac OS encrypted encoding](https://github.com/keramics/keramics/tree/main/keramics-formats/src/cdsaencr/README.md) |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Compression methods | LZMA |
