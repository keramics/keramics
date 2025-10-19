/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

/// ext superblock signature: "\x53\xef"
pub(super) const EXT_SUPERBLOCK_SIGNATURE: [u8; 2] = [0x53, 0xef];

/// ext extents header signature: "\x0a\xf3"
pub(super) const EXT_EXTENTS_HEADER_SIGNATURE: [u8; 2] = [0x0a, 0xf3];

/// ext attribute inode or block header signature: "\x00\x00\x02\xea"
pub(super) const EXT_ATTRIBUTES_HEADER_SIGNATURE: [u8; 4] = [0x00, 0x00, 0x02, 0xea];

/// ext compatible feature flags
pub const EXT_COMPATIBLE_FEATURE_FLAG_SPARSE_SUPERBLOCK2: u32 = 0x00000200;

/// ext incompatible feature flags
pub const EXT_INCOMPATIBLE_FEATURE_FLAG_JOURNAL_DEVICE: u32 = 0x00000008;
pub const EXT_INCOMPATIBLE_FEATURE_FLAG_HAS_META_BLOCK_GROUPS: u32 = 0x00000010;
pub const EXT_INCOMPATIBLE_FEATURE_FLAG_64BIT_SUPPORT: u32 = 0x00000080;
pub const EXT_INCOMPATIBLE_FEATURE_FLAG_HAS_FLEX_BLOCK_GROUPS: u32 = 0x00000200;
pub const EXT_INCOMPATIBLE_FEATURE_FLAG_HAS_METADATA_CHECKSUM_SEED: u32 = 0x00002000;

/// ext read-only compatible feature flags
pub const EXT_READ_ONLY_COMPATIBLE_FEATURE_FLAG_SPARSE_SUPERBLOCK: u32 = 0x00000001;
pub const EXT_READ_ONLY_COMPATIBLE_FEATURE_FLAG_METADATA_CHECKSUM: u32 = 0x00000400;

/// ext inode flags
pub(super) const EXT_INODE_FLAG_COMPRESSED_DATA: u32 = 0x00000200;
pub(super) const EXT_INODE_FLAG_HAS_EXTENTS: u32 = 0x00080000;
pub(super) const EXT_INODE_FLAG_IS_EXTENDED_ATTRIBUTE_INODE: u32 = 0x00200000;
pub(super) const EXT_INODE_FLAG_INLINE_DATA: u32 = 0x10000000;

/// ext file mode types
pub const EXT_FILE_MODE_TYPE_FIFO: u16 = 0x1000;
pub const EXT_FILE_MODE_TYPE_CHARACTER_DEVICE: u16 = 0x2000;
pub const EXT_FILE_MODE_TYPE_DIRECTORY: u16 = 0x4000;
pub const EXT_FILE_MODE_TYPE_BLOCK_DEVICE: u16 = 0x6000;
pub const EXT_FILE_MODE_TYPE_REGULAR_FILE: u16 = 0x8000;
pub const EXT_FILE_MODE_TYPE_SYMBOLIC_LINK: u16 = 0xa000;
pub const EXT_FILE_MODE_TYPE_SOCKET: u16 = 0xc000;

/// ext root directory identifier (inode number).
pub(super) const EXT_ROOT_DIRECTORY_IDENTIFIER: u32 = 2;
