# mbr

The mbr module provides read-only support for the
[Master Boot Record (MBR)](https://keramics.github.io/mbr.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Partition types | Primary partitions, Extended (EBR) logical partitions |
| Sector sizes | 512, 1024, 2048, 4096 bytes |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| | Multiple extended partition entries per boot record |
