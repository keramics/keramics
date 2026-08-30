/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may
 * obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

/// XFS superblock signature
pub(crate) const XFS_SUPERBLOCK_SIGNATURE: &[u8] = b"XFSB";

/// XFS inode information signature
pub(super) const XFS_INODE_INFORMATION_SIGNATURE: &[u8] = b"XAGI";

// XFS free block information signature
// pub(super) const XFS_FREE_BLOCK_INFORMATION_SIGNATURE: &[u8] = b"XAGF";

/// XFS extent B-tree node signature
pub(super) const XFS_EXTENT_TREE_SIGNATURE: &[u8] = b"BMAP";

/// XFS extent B-tree node version 5 signature
pub(super) const XFS_EXTENT_TREE_V5_SIGNATURE: &[u8] = b"BMA3";

/// XFS inode B-tree node signature
pub(super) const XFS_INODE_TREE_SIGNATURE: &[u8] = b"IABT";

/// XFS inode B-tree node version 5 signature
pub(super) const XFS_INODE_TREE_V5_SIGNATURE: &[u8] = b"IAB3";

/// XFS inode signature
pub(super) const XFS_INODE_SIGNATURE: &[u8] = b"IN";

/// XFS file mode types
pub const XFS_FILE_MODE_TYPE_FIFO: u16 = 0x1000;
pub const XFS_FILE_MODE_TYPE_CHARACTER_DEVICE: u16 = 0x2000;
pub const XFS_FILE_MODE_TYPE_DIRECTORY: u16 = 0x4000;
pub const XFS_FILE_MODE_TYPE_BLOCK_DEVICE: u16 = 0x6000;
pub const XFS_FILE_MODE_TYPE_REGULAR_FILE: u16 = 0x8000;
pub const XFS_FILE_MODE_TYPE_SYMBOLIC_LINK: u16 = 0xa000;
pub const XFS_FILE_MODE_TYPE_SOCKET: u16 = 0xc000;

/// XFS fork types
pub const XFS_FORK_TYPE_DEVICE: u8 = 0;
pub const XFS_FORK_TYPE_INLINE_DATA: u8 = 1;
pub const XFS_FORK_TYPE_EXTENTS: u8 = 2;
pub const XFS_FORK_TYPE_BTREE: u8 = 3;
