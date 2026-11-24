# exfat

The exfat module provides read-only support for the
[Extensible File Allocation Table (exFAT)](https://keramics.github.io/exfat.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Sector sizes | 512, 1024, 2048, 4096 bytes |
| | Case-insensitive path lookup (case folding mappings) |
| | Contiguous and fragmented data streams |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| | TexFAT (Transaction-Safe Extended FAT) |
