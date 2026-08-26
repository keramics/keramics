# linuxlvm

The linuxlvm module provides read-only support for the
[Linux Logical Volume Manager (LVM) format](https://keramics.github.io/linuxlvm.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Configurations | Single physical volume |
| Segment types | striped  |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Configurations | Multiple physical volumes, Mulitiple segments |
| Segment types | cache, error, integrity, linear, mirror, raid0, raid1, raid4, raid5, raid6, raid10, snapshot, thin, vdo, zero |
| | Snapshots |
