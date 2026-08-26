# vmdk

The vmdk module provides read-only support for the
[VMWare Virtual Disk Format (VMDK)](https://keramics.github.io/vmdk.html).

Supported features:

| Category | Feature(s) |
| --- | --- |
| Extent file types | Flat (or RAW), Sparse (VMDK (version 1, 2, 3)) |
| Compression | zlib for VMDK extent file type |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Extent file types | Sparse (COWD (version 1)), Physical device |
| | Data markers |
| | Delta links |
| | Changed block tracking (CBT) (or change tracking file) |
