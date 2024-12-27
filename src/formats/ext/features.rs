/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::checksums::ReversedCrc32Context;

use super::constants::*;
use super::superblock::ExtSuperblock;

/// Extended File System (ext) features.
pub struct ExtFeatures {
    /// Compatible feature flags.
    pub compatible_feature_flags: u32,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u32,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u32,

    /// Metadata checksum seed.
    metadata_checksum_seed: Option<u32>,
}

impl ExtFeatures {
    /// Creates new features.
    pub fn new() -> Self {
        Self {
            compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            metadata_checksum_seed: None,
        }
    }

    /// Initializes the features.
    pub fn initialize(&mut self, superblock: &ExtSuperblock) {
        self.compatible_feature_flags = superblock.compatible_feature_flags;
        self.incompatible_feature_flags = superblock.incompatible_feature_flags;
        self.read_only_compatible_feature_flags = superblock.read_only_compatible_feature_flags;

        if superblock.read_only_compatible_feature_flags
            & EXT_READ_ONLY_COMPATIBLE_FEATURE_FLAG_METADATA_CHECKSUM
            != 0
        {
            let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0x82f63b78, 0);
            crc32_context.update(&superblock.file_system_identifier);
            let checksum: u32 = crc32_context.finalize();

            self.metadata_checksum_seed = Some(checksum);
        }
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u8 {
        if self.compatible_feature_flags & 0x00000200 != 0
            || self.incompatible_feature_flags & 0x0001f7c0 != 0
            || self.read_only_compatible_feature_flags & 0x00000378 != 0
        {
            4
        } else if self.compatible_feature_flags & 0x00000004 != 0
            || self.incompatible_feature_flags & 0x0000000c != 0
        {
            3
        } else {
            2
        }
    }

    /// Retrieves the group descriptor size.
    pub fn get_group_descriptor_size(&self) -> u32 {
        if self.incompatible_feature_flags & EXT_INCOMPATIBLE_FEATURE_FLAG_64BIT_SUPPORT != 0 {
            64
        } else {
            32
        }
    }

    /// Retrieves the metadata checksum seed.
    pub fn get_metadata_checksum_seed(&self) -> Option<u32> {
        self.metadata_checksum_seed
    }

    /// Determines if the meta block groups feature is used.
    pub fn has_meta_block_groups(&self) -> bool {
        self.incompatible_feature_flags & EXT_INCOMPATIBLE_FEATURE_FLAG_HAS_META_BLOCK_GROUPS != 0
    }

    /// Determines if the sparse superblock feature is used.
    pub fn has_sparse_superblock(&self) -> bool {
        self.read_only_compatible_feature_flags
            & EXT_READ_ONLY_COMPATIBLE_FEATURE_FLAG_SPARSE_SUPERBLOCK
            != 0
    }

    /// Determines if the sparse superblock version 2 feature is used.
    pub fn has_sparse_superblock2(&self) -> bool {
        self.compatible_feature_flags & EXT_COMPATIBLE_FEATURE_FLAG_SPARSE_SUPERBLOCK2 != 0
    }

    /// Checks if there are unsupported features.
    pub fn is_unsupported(&self) -> bool {
        let supported_flags: u32 = 0x00000002 // EXT2_FEATURE_INCOMPAT_FILETYPE
            | 0x00000004 // EXT3_FEATURE_INCOMPAT_RECOVER
            | 0x00000008 // EXT3_FEATURE_INCOMPAT_JOURNAL_DEV
            | 0x00000040 // EXT4_FEATURE_INCOMPAT_EXTENTS
            | 0x00000080 // EXT4_FEATURE_INCOMPAT_64BIT
            | 0x00000200 // EXT4_FEATURE_INCOMPAT_FLEX_BG
            | 0x00000400 // EXT4_FEATURE_INCOMPAT_EA_INODE
            | 0x00002000 // EXT4_FEATURE_INCOMPAT_CSUM_SEED,
            | 0x00008000 // EXT4_FEATURE_INCOMPAT_INLINE_DATA
            | 0x00010000 // EXT4_FEATURE_INCOMPAT_ENCRYPT
            | 0x00020000; // EXT4_FEATURE_INCOMPAT_CASEFOLD

        self.incompatible_feature_flags & !(supported_flags) != 0
    }
}
