# gpt

The gpt module provides read-only support for the
[GUID Partition Table (GPT)](https://keramics.github.io/gpt.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Sector sizes | 512, 1024, 2048, 4096 bytes |
| | CRC-32 checksum verification |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| | Backup partition table fallback and comparison |
