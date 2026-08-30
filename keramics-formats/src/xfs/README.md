# xfs

The xfs module provides read-only support for the
[X File System (XFS)](https://keramics.github.io/xfs.html) format.

Supported features:

| Category | Feature(s) |
| --- | --- |
| Superblock format version | 1, 2, 3, 4, 5 |
| Directory format version | 1, 2, 3 |
| Inode format version | 1, 2, 3 |
| Extended attributes format version | 2, 3 |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Feature flags | XFS_SB_VERSION_QUOTABIT |
| Incompatible feature flags| XFS_SB_FEAT_INCOMPAT_META_UUID, XFS_SB_FEAT_INCOMPAT_NEEDSREPAIR, XFS_SB_FEAT_INCOMPAT_METADIR, XFS_SB_FEAT_INCOMPAT_ZONED, XFS_SB_FEAT_INCOMPAT_ZONE_GAPS |
