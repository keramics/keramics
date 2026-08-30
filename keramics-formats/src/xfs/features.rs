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

use super::superblock::XfsSuperblock;

/// X File System (XFS) features.
pub struct XfsFeatures {
    /// Format version.
    pub format_version: u16,

    /// Feature flags.
    pub feature_flags: u16,

    /// Secondary feature flags.
    pub secondary_feature_flags: u32,

    /// Compatible feature flags.
    pub compatible_feature_flags: u32,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u32,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u32,

    /// Journal incompatible feature flags.
    pub journal_incompatible_feature_flags: u32,
}

impl XfsFeatures {
    /// Creates new features.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            feature_flags: 0,
            secondary_feature_flags: 0,
            compatible_feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            journal_incompatible_feature_flags: 0,
        }
    }

    /// Determines if 64-bit number of data extents and 32-bit number of attributes extents are used.
    pub fn has_64bit_number_of_extents(&self) -> bool {
        self.incompatible_feature_flags & 0x00000020 != 0
    }

    /// Determines if bigtime date and time values are used.
    pub fn has_bigtime(&self) -> bool {
        self.incompatible_feature_flags & 0x00000008 != 0
    }

    /// Determines if version 2 directories are used.
    pub fn has_directory_v2(&self) -> bool {
        self.feature_flags & 0x2000 != 0
    }

    /// Determines if directory entries contain a file type value.
    pub fn has_file_type(&self) -> bool {
        self.format_version >= 5 || self.secondary_feature_flags & 0x00000200 != 0
    }

    /// Initializes the features.
    pub fn initialize(&mut self, superblock: &XfsSuperblock) {
        self.format_version = superblock.format_version;
        self.feature_flags = superblock.feature_flags;
        self.secondary_feature_flags = superblock.secondary_feature_flags;
        self.compatible_feature_flags = superblock.compatible_feature_flags;
        self.read_only_compatible_feature_flags = superblock.read_only_compatible_feature_flags;
        self.incompatible_feature_flags = superblock.incompatible_feature_flags;
        self.journal_incompatible_feature_flags = superblock.journal_incompatible_feature_flags;
    }

    /// Checks if there are unsupported features.
    pub fn is_unsupported(&self) -> bool {
        let supported_flags: u16 = match self.format_version {
            2 => 0x0010,
            3 => {
                0x0010  // XFS_SB_VERSION_ATTRBIT
                | 0x0020 // XFS_SB_VERSION_NLINKBIT
            }
            4 | 5 => {
                0x0010  // XFS_SB_VERSION_ATTRBIT
                    | 0x0020  // XFS_SB_VERSION_NLINKBIT
                    | 0x0080  // XFS_SB_VERSION_ALIGNBIT
                    | 0x0100  // XFS_SB_VERSION_DALIGNBIT
                    | 0x0400  // XFS_SB_VERSION_LOGV2BIT
                    | 0x0800  // XFS_SB_VERSION_SECTORBIT
                    | 0x1000  // XFS_SB_VERSION_EXTFLGBIT
                    | 0x2000  // XFS_SB_VERSION_DIRV2BIT
                    | 0x4000  // XFS_SB_VERSION_BORGBIT
                    | 0x8000 // XFS_SB_VERSION_MOREBITSBIT
            }
            _ => 0x0000,
        };
        if self.feature_flags & !(supported_flags) != 0 {
            return true;
        }
        if self.format_version >= 5 {
            let supported_flags: u32 = 0x00000001  // XFS_SB_FEAT_INCOMPAT_FTYPE
                | 0x00000002  // XFS_SB_FEAT_INCOMPAT_SPINODES
                | 0x00000008  // XFS_SB_FEAT_INCOMPAT_BIGTIME
                | 0x00000020  // XFS_SB_FEAT_INCOMPAT_NREXT64
                | 0x00000040  // XFS_SB_FEAT_INCOMPAT_EXCHRANGE
                | 0x00000080; // XFS_SB_FEAT_INCOMPAT_PARENT

            if self.incompatible_feature_flags & !(supported_flags) != 0 {
                return true;
            }
            let supported_flags: u32 = 0x00000000;

            if self.journal_incompatible_feature_flags & !(supported_flags) != 0 {
                return true;
            }
        }
        false
    }
}
