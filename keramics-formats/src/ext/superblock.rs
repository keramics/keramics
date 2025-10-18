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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::ErrorTrace;
use keramics_datetime::PosixTime32;
use keramics_layout_map::LayoutMap;
use keramics_types::{ByteString, bytes_to_u16_le, bytes_to_u32_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "number_of_inodes", data_type = "u32"),
        field(name = "number_of_blocks_lower", data_type = "u32"),
        field(name = "number_of_reserved_blocks_lower", data_type = "u32"),
        field(name = "number_of_unallocated_blocks_lower", data_type = "u32"),
        field(name = "number_of_unallocated_inodes_lower", data_type = "u32"),
        field(name = "first_data_block_number", data_type = "u32"),
        field(name = "block_size", data_type = "u32"),
        field(name = "fragment_size", data_type = "u32"),
        field(name = "number_of_blocks_per_block_group", data_type = "u32"),
        field(name = "number_of_fragments_per_block_group", data_type = "u32"),
        field(name = "number_of_inodes_per_block_group", data_type = "u32"),
        field(name = "last_mount_time", data_type = "PosixTime32"),
        field(name = "last_written_time", data_type = "PosixTime32"),
        field(name = "mount_count", data_type = "u16"),
        field(name = "maximum_mount_count", data_type = "u16"),
        field(name = "signature", data_type = "[u8; 2]"),
        field(name = "file_system_state_flags", data_type = "u16", format = "hex"),
        field(name = "error_handling_status", data_type = "u16"),
        field(name = "minor_format_revision", data_type = "u16"),
        field(name = "last_consistency_check_time", data_type = "PosixTime32"),
        field(name = "consistency_check_interval", data_type = "u32"),
        field(name = "creator_operating_system", data_type = "u32"),
        field(name = "format_revision", data_type = "u32"),
        field(name = "reserved_block_user_identifier", data_type = "u16"),
        field(name = "reserved_block_group_identifier", data_type = "u16"),
        field(name = "first_non_reserved_inode", data_type = "u32"),
        field(name = "inode_size", data_type = "u16"),
        field(name = "block_group", data_type = "u16"),
        field(name = "compatible_feature_flags", data_type = "u32", format = "hex"),
        field(name = "incompatible_feature_flags", data_type = "u32", format = "hex"),
        field(
            name = "read_only_compatible_feature_flags",
            data_type = "u32",
            format = "hex"
        ),
        field(
            name = "file_system_identifier",
            data_type = "uuid",
            byte_order = "big"
        ),
        field(name = "volume_label", data_type = "ByteString<16>"),
        field(name = "last_mount_path", data_type = "ByteString<64>"),
        field(name = "algorithm_usage_bitmap", data_type = "u32"),
        field(name = "number_of_pre_allocated_blocks_per_file", data_type = "u8"),
        field(
            name = "number_of_pre_allocated_blocks_per_directory",
            data_type = "u8"
        ),
        field(name = "padding1", data_type = "[u8; 2]"),
        field(name = "journal_identifier", data_type = "uuid", byte_order = "big"),
        field(name = "journal_inode_number", data_type = "u32"),
        field(name = "journal_device", data_type = "u32"),
        field(name = "orphan_inode_list_head", data_type = "u32"),
        field(name = "htree_hash_seed", data_type = "[u8; 16]"),
        field(name = "default_hash_version", data_type = "u8"),
        field(name = "journal_backup_type", data_type = "u8"),
        field(name = "group_descriptor_size", data_type = "u16"),
        field(name = "default_mount_options", data_type = "u32"),
        field(name = "first_meta_block_group", data_type = "u32"),
        field(name = "file_system_creation_time", data_type = "PosixTime32"),
        field(name = "backup_journal_inodes", data_type = "[u32; 17]"),
        field(name = "number_of_blocks_upper", data_type = "u32"),
        field(name = "number_of_reserved_blocks_upper", data_type = "u32"),
        field(name = "number_of_unallocated_blocks_upper", data_type = "u32"),
        field(name = "minimum_inode_size", data_type = "u16"),
        field(name = "reserved_inode_size", data_type = "u16"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "read_stride", data_type = "u16"),
        field(name = "multi_mount_protection_update_interval", data_type = "u16"),
        field(name = "multi_mount_protection_block", data_type = "u64"),
        field(name = "raid_stripe_width", data_type = "u32"),
        field(name = "number_of_block_groups_per_flex_group", data_type = "u8"),
        field(name = "checksum_type", data_type = "u8"),
        field(name = "encryption_level", data_type = "u8"),
        field(name = "padding2", data_type = "u8"),
        field(name = "write_count", data_type = "u64"),
        field(name = "snapshot_inode_number", data_type = "u32"),
        field(name = "snapshot_sequential_identifier", data_type = "u32"),
        field(name = "snapshot_number_of_reserved_blocks", data_type = "u64"),
        field(name = "snapshot_inode_list", data_type = "u32"),
        field(name = "number_of_errors", data_type = "u32"),
        field(name = "first_error_time", data_type = "PosixTime32"),
        field(name = "first_error_inode_number", data_type = "u32"),
        field(name = "first_error_block_number", data_type = "u64"),
        field(name = "first_error_function", data_type = "ByteString<32>"),
        field(name = "first_error_function_line_number", data_type = "u32"),
        field(name = "last_error_time", data_type = "PosixTime32"),
        field(name = "last_error_inode_number", data_type = "u32"),
        field(name = "last_error_function_line_number", data_type = "u32"),
        field(name = "last_error_block_number", data_type = "u64"),
        field(name = "last_error_function", data_type = "ByteString<32>"),
        field(name = "mount_options", data_type = "ByteString<64>"),
        field(name = "user_quota_inode_number", data_type = "u32"),
        field(name = "group_quota_inode_number", data_type = "u32"),
        field(name = "overhead_number_of_clusters", data_type = "u32"),
        field(name = "backup_block_group1", data_type = "u32"),
        field(name = "backup_block_group2", data_type = "u32"),
        field(name = "encryption_algorithms", data_type = "u32"),
        field(name = "encryption_password_salt", data_type = "[u8; 16]"),
        field(name = "lost_and_found_inode_number", data_type = "u32"),
        field(name = "project_quota_inode_number", data_type = "u32"),
        field(name = "metadata_checksum_seed", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 8]"),
        field(name = "encoding", data_type = "u16"),
        field(name = "encoding_flags", data_type = "u16", format = "hex"),
        field(name = "orphan_file_inode_number", data_type = "u32"),
        field(name = "padding3", data_type = "[u8; 376]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// Extended File System (ext) superblock.
pub struct ExtSuperblock {
    /// Number of inodes.
    pub number_of_inodes: u32,

    /// Number of blocks.
    pub number_of_blocks: u64,

    /// Block size.
    pub block_size: u32,

    /// Number of blocks per block group.
    pub number_of_blocks_per_block_group: u32,

    /// Number of inodes per block group.
    pub number_of_inodes_per_block_group: u32,

    /// Last mount date and time.
    pub last_mount_time: PosixTime32,

    /// Last written date and time.
    pub last_written_time: PosixTime32,

    /// Format revision.
    pub format_revision: u32,

    /// Inode size.
    pub inode_size: u16,

    /// Compatible feature flags.
    pub compatible_feature_flags: u32,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u32,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u32,

    /// File system identifier.
    pub file_system_identifier: [u8; 16],

    /// Volume label.
    pub volume_label: ByteString,

    /// Last mount path.
    pub last_mount_path: ByteString,

    /// Group descriptor size.
    pub group_descriptor_size: u16,

    /// First meta block group.
    pub first_meta_block_group: u32,

    /// Number of block groups per flex group.
    pub number_of_block_groups_per_flex_group: u32,

    /// Metadata checksum seed.
    pub metadata_checksum_seed: Option<u32>,
}

impl ExtSuperblock {
    /// Creates a new superblock.
    pub fn new() -> Self {
        Self {
            number_of_inodes: 0,
            number_of_blocks: 0,
            block_size: 0,
            number_of_blocks_per_block_group: 0,
            number_of_inodes_per_block_group: 0,
            last_mount_time: PosixTime32::new(0),
            last_written_time: PosixTime32::new(0),
            format_revision: 0,
            inode_size: 0,
            compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            file_system_identifier: [0; 16],
            volume_label: ByteString::new(),
            last_mount_path: ByteString::new(),
            group_descriptor_size: 0,
            number_of_block_groups_per_flex_group: 0,
            first_meta_block_group: 0,
            metadata_checksum_seed: None,
        }
    }

    /// Reads the superblock from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 1024 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported ext superblock data size"
            ));
        }
        if data[56..58] != EXT_SUPERBLOCK_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported ext superblock signature"
            ));
        }
        self.format_revision = bytes_to_u32_le!(data, 76);

        if self.format_revision > 1 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format revision: {}",
                self.format_revision
            )));
        }
        self.compatible_feature_flags = bytes_to_u32_le!(data, 92);
        self.incompatible_feature_flags = bytes_to_u32_le!(data, 96);
        self.read_only_compatible_feature_flags = bytes_to_u32_le!(data, 100);

        self.number_of_inodes = bytes_to_u32_le!(data, 0);
        self.number_of_blocks = bytes_to_u32_le!(data, 4) as u64;
        if self.incompatible_feature_flags & EXT_INCOMPATIBLE_FEATURE_FLAG_64BIT_SUPPORT != 0 {
            self.number_of_blocks |= (bytes_to_u32_le!(data, 336) as u64) << 32;
        }
        if self.number_of_blocks == 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of blocks: {} value out of bounds",
                self.number_of_blocks
            )));
        }
        self.block_size = bytes_to_u32_le!(data, 24);

        if self.block_size > 32 - 10 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid block size: {} value out of bounds",
                self.block_size
            )));
        }
        self.block_size = 1024 << self.block_size;

        self.number_of_blocks_per_block_group = bytes_to_u32_le!(data, 32);

        if self.number_of_blocks_per_block_group == 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of blocks per block group: {} value out of bounds",
                self.number_of_blocks_per_block_group
            )));
        }
        self.number_of_inodes_per_block_group = bytes_to_u32_le!(data, 40);

        if self.incompatible_feature_flags & EXT_INCOMPATIBLE_FEATURE_FLAG_JOURNAL_DEVICE == 0 {
            if self.number_of_inodes_per_block_group == 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid number of inodes per block group: {} value out of bounds",
                    self.number_of_inodes_per_block_group
                )));
            }
        }
        self.last_mount_time = PosixTime32::from_le_bytes(&data[44..48]);
        self.last_written_time = PosixTime32::from_le_bytes(&data[48..52]);
        self.inode_size = bytes_to_u16_le!(data, 88);
        self.group_descriptor_size = bytes_to_u16_le!(data, 254);
        self.first_meta_block_group = bytes_to_u32_le!(data, 260);
        self.file_system_identifier.copy_from_slice(&data[104..120]);
        self.volume_label = ByteString::from(&data[120..136]);
        self.last_mount_path = ByteString::from(&data[136..200]);

        let number_of_block_groups_per_flex_group: u8 = data[372];
        if number_of_block_groups_per_flex_group >= 16 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of blocks per flex group: {} value out of bounds",
                number_of_block_groups_per_flex_group,
            )));
        }
        self.number_of_block_groups_per_flex_group =
            1 << (number_of_block_groups_per_flex_group as u32);

        if self.incompatible_feature_flags
            & EXT_INCOMPATIBLE_FEATURE_FLAG_HAS_METADATA_CHECKSUM_SEED
            != 0
        {
            let stored_checksum: u32 = bytes_to_u32_le!(data, 624);
            self.metadata_checksum_seed = Some(0xffffffff - stored_checksum);
        }
        if self.read_only_compatible_feature_flags
            & EXT_READ_ONLY_COMPATIBLE_FEATURE_FLAG_METADATA_CHECKSUM
            != 0
        {
            let checksum_type: u8 = data[373];
            if checksum_type != 1 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported checksum type: {}",
                    checksum_type
                )));
            }
            let stored_checksum: u32 = bytes_to_u32_le!(data, 1020);

            let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0x82f63b78, 0);
            crc32_context.update(&data[0..1020]);

            let mut calculated_checksum: u32 = crc32_context.finalize();
            calculated_checksum = 0xffffffff - calculated_checksum;

            if stored_checksum != 0 && stored_checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    stored_checksum, calculated_checksum
                )));
            }
        }
        Ok(())
    }

    /// Retrieves the block group size.
    pub fn get_block_group_size(&self) -> u64 {
        (self.number_of_blocks_per_block_group as u64) * (self.block_size as u64)
    }

    /// Retrieves the number of block groups.
    pub fn get_number_of_block_groups(&self) -> u64 {
        self.number_of_blocks
            .div_ceil(self.number_of_blocks_per_block_group as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x04, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x58, 0x0f,
            0x00, 0x00, 0xf0, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0xa3, 0x7b, 0xf9, 0x60, 0xa4, 0x7b, 0xf9, 0x60, 0x01, 0x00, 0xff, 0xff,
            0x53, 0xef, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0xa3, 0x7b, 0xf9, 0x60, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0b, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0xf0, 0x00, 0x50, 0xbb, 0x07, 0xee, 0x46, 0xa3,
            0x83, 0xa6, 0xa4, 0x05, 0xee, 0x0d, 0xb5, 0x1f, 0x65, 0x78, 0x74, 0x32, 0x5f, 0x74,
            0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x6d, 0x6e, 0x74,
            0x2f, 0x6b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa6, 0x0d,
            0x1b, 0x0a, 0x17, 0xa0, 0x4e, 0x3e, 0x8a, 0x1f, 0x7f, 0x4f, 0x89, 0x7e, 0x46, 0x4e,
            0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa3, 0x7b,
            0xf9, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtSuperblock::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_inodes, 1024);
        assert_eq!(test_struct.number_of_blocks, 4096);
        assert_eq!(test_struct.block_size, 1024);
        assert_eq!(test_struct.number_of_blocks_per_block_group, 8192);
        assert_eq!(test_struct.number_of_inodes_per_block_group, 1024);
        assert_eq!(test_struct.last_mount_time, PosixTime32::new(1626962851));
        assert_eq!(test_struct.last_written_time, PosixTime32::new(1626962852));
        assert_eq!(test_struct.format_revision, 1);
        assert_eq!(test_struct.inode_size, 128);
        assert_eq!(test_struct.compatible_feature_flags, 0x00000038);
        assert_eq!(test_struct.incompatible_feature_flags, 0x00000002);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x00000003);

        let expected_file_system_identifier: [u8; 16] = [
            0xf0, 0x00, 0x50, 0xbb, 0x07, 0xee, 0x46, 0xa3, 0x83, 0xa6, 0xa4, 0x05, 0xee, 0x0d,
            0xb5, 0x1f,
        ];
        assert_eq!(
            test_struct.file_system_identifier,
            expected_file_system_identifier
        );
        assert_eq!(test_struct.volume_label, ByteString::from("ext2_test"));
        assert_eq!(
            test_struct.last_mount_path,
            ByteString::from("/mnt/keramics")
        );
        assert_eq!(test_struct.group_descriptor_size, 0);
        assert_eq!(test_struct.number_of_block_groups_per_flex_group, 1);
        assert_eq!(test_struct.first_meta_block_group, 0);
        assert_eq!(test_struct.metadata_checksum_seed, None);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtSuperblock::new();
        let result = test_struct.read_data(&test_data[0..1023]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[56] = 0xff;

        let mut test_struct = ExtSuperblock::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_block_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[24] = 0xff;

        let mut test_struct = ExtSuperblock::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_revision_format() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[76] = 0xff;

        let mut test_struct = ExtSuperblock::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
