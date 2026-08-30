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

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterEncoding;
use keramics_layout_map::LayoutMap;
use keramics_types::{ByteString, bytes_to_u16_be, bytes_to_u32_be, bytes_to_u64_be};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "block_size", data_type = "u32"),
        field(name = "number_of_blocks", data_type = "u64"),
        field(name = "number_of_realtime_blocks", data_type = "u64"),
        field(name = "number_of_realtime_extents", data_type = "u64"),
        field(name = "file_system_identifier", data_type = "Uuid"),
        field(name = "journal_block_number", data_type = "u64"),
        field(name = "root_directory_inode_number", data_type = "u64"),
        field(name = "realtime_bitmap_extents_inode_number", data_type = "u64"),
        field(name = "realtime_bitmap_summary_inode_number", data_type = "u64"),
        field(name = "realtime_extents_size", data_type = "u32"),
        field(name = "allocation_group_size", data_type = "u32"),
        field(name = "number_of_allocation_groups", data_type = "u32"),
        field(name = "realtime_bitmap_size", data_type = "u32"),
        field(name = "journal_size", data_type = "u32"),
        field(name = "version_and_feature_flags", data_type = "u16", format = "hex"),
        field(name = "bytes_per_sector", data_type = "u16"),
        field(name = "inode_size", data_type = "u16"),
        field(name = "number_of_inodes_per_block", data_type = "u16"),
        field(name = "volume_label", data_type = "ByteString<12>"),
        field(name = "block_size_log2", data_type = "u8"),
        field(name = "bytes_per_sector_log2", data_type = "u8"),
        field(name = "inode_size_log2", data_type = "u8"),
        field(name = "number_of_inodes_per_block_log2", data_type = "u8"),
        field(name = "allocation_group_size_log2", data_type = "u8"),
        field(name = "number_of_realtime_extents_log2", data_type = "u8"),
        field(name = "creation_flag", data_type = "u8"),
        field(name = "inodes_percentage", data_type = "u8"),
        field(name = "number_of_inodes", data_type = "u64"),
        field(name = "number_of_free_inodes", data_type = "u64"),
        field(name = "number_of_free_data_blocks", data_type = "u64"),
        field(name = "number_of_free_realtime_extents", data_type = "u64"),
        field(name = "user_quota_inode_number", data_type = "u64"),
        field(name = "group_quota_inode_number", data_type = "u64"),
        field(name = "quota_flags", data_type = "u16", format = "hex"),
        field(name = "miscellaneous_flags", data_type = "u8", format = "hex"),
        field(name = "unknown1", data_type = "u8"),
        field(name = "inode_chunk_alignment_size", data_type = "u32"),
        field(name = "raid_unit_size", data_type = "u32"),
        field(name = "raid_width", data_type = "u32"),
        field(name = "directory_block_size_log2", data_type = "u8"),
        field(name = "journal_device_bytes_per_sector_log2", data_type = "u8"),
        field(name = "journal_device_bytes_per_sector", data_type = "u16"),
        field(name = "journal_device_raid_unit_size", data_type = "u32"),
        field(name = "secondary_feature_flags", data_type = "u32", format = "hex"),
        field(
            name = "secondary_feature_flags_copy",
            data_type = "u32",
            format = "hex"
        ),
        field(name = "compatible_feature_flags", data_type = "u32", format = "hex"),
        field(
            name = "read_only_compatible_feature_flags",
            data_type = "u32",
            format = "hex"
        ),
        field(name = "incompatible_feature_flags", data_type = "u32", format = "hex"),
        field(
            name = "journal_incompatible_feature_flags",
            data_type = "u32",
            format = "hex"
        ),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "unknown2", data_type = "u32"),
        field(name = "project_quota_inode_number", data_type = "u32"),
        field(name = "log_sequence_number", data_type = "u64"),
        field(name = "metadata_identifier", data_type = "Uuid"),
        field(name = "realtime_reverse_mapping_tree_inode_number", data_type = "u64"),
        field(name = "unknown3", data_type = "[u8; 244]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// X File System (XFS) superblock
pub struct XfsSuperblock {
    /// Block size.
    pub block_size: u32,

    /// Root directory (absolute) inode number.
    pub root_directory_inode_number: u64,

    /// Allocation group size.
    pub allocation_group_size: u32,

    /// Number of allocation groups.
    pub number_of_allocation_groups: u32,

    /// Format version.
    pub format_version: u16,

    /// Feature flags.
    pub feature_flags: u16,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Inode size.
    pub inode_size: u16,

    /// Number of inodes per block.
    pub inodes_per_block: u16,

    /// Volume label.
    pub volume_label: ByteString,

    /// Directory block size.
    pub directory_block_size: u32,

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

    /// Number of relative block number bits.
    pub number_of_relative_block_number_bits: u32,

    /// Number of relative inode number bits.
    pub number_of_relative_inode_number_bits: u32,
}

impl XfsSuperblock {
    const SUPPORTED_BLOCK_SIZES: [u32; 8] = [512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];

    const SUPPORTED_BYTES_PER_SECTOR: [u16; 6] = [512, 1024, 2048, 4096, 8192, 16384];

    const SUPPORTED_INODE_SIZE: [u16; 4] = [256, 512, 1024, 2048];

    /// Creates a new superblock.
    pub fn new(encoding: &CharacterEncoding) -> Self {
        Self {
            block_size: 0,
            root_directory_inode_number: 0,
            allocation_group_size: 0,
            number_of_allocation_groups: 0,
            format_version: 0,
            feature_flags: 0,
            bytes_per_sector: 0,
            inode_size: 0,
            inodes_per_block: 0,
            volume_label: ByteString::new_with_encoding(encoding),
            directory_block_size: 0,
            secondary_feature_flags: 0,
            compatible_feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            journal_incompatible_feature_flags: 0,
            number_of_relative_block_number_bits: 0,
            number_of_relative_inode_number_bits: 0,
        }
    }

    /// Reads the superblock from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..4] != XFS_SUPERBLOCK_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let value_16bit: u16 = bytes_to_u16_be!(data, 100);

        self.format_version = value_16bit & 0x000f;

        if self.format_version == 0 || self.format_version > 5 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format version: {}",
                self.format_version
            )));
        }
        self.block_size = bytes_to_u32_be!(data, 4);

        if !Self::SUPPORTED_BLOCK_SIZES.contains(&self.block_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported block size: {}",
                self.block_size
            )));
        }
        self.feature_flags = value_16bit & 0xfff0;

        self.root_directory_inode_number = bytes_to_u64_be!(data, 56);
        self.allocation_group_size = bytes_to_u32_be!(data, 84);

        if self.allocation_group_size < 5 || self.allocation_group_size > (i32::MAX as u32) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported allocation group size value out of bounds: {}",
                self.allocation_group_size
            )));
        }
        self.number_of_allocation_groups = bytes_to_u32_be!(data, 88);

        self.bytes_per_sector = bytes_to_u16_be!(data, 102);

        if !Self::SUPPORTED_BYTES_PER_SECTOR.contains(&self.bytes_per_sector) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported bytes per sector: {}",
                self.bytes_per_sector
            )));
        }
        self.inode_size = bytes_to_u16_be!(data, 104);

        if !Self::SUPPORTED_INODE_SIZE.contains(&self.inode_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported inode size: {}",
                self.inode_size
            )));
        }
        self.inodes_per_block = bytes_to_u16_be!(data, 106);

        self.volume_label.read_data(&data[108..120]);

        let value_32bit: u32 = data[124] as u32;

        if value_32bit == 0 || value_32bit >= 32 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported allocation group size log2 value out of bounds: {}",
                value_32bit
            )));
        }
        self.number_of_relative_block_number_bits = value_32bit;

        let value_32bit: u32 = data[123] as u32;

        if value_32bit == 0 || value_32bit >= 32 - self.number_of_relative_block_number_bits {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported inodes per block log2 value out of bounds: {}",
                value_32bit
            )));
        }
        self.number_of_relative_inode_number_bits =
            self.number_of_relative_block_number_bits + value_32bit;

        if 1 << value_32bit != (self.inodes_per_block as u32) {
            return Err(keramics_core::error_trace_new!(
                "Mismatch between number of inodes per block and log2 values"
            ));
        }
        let value_32bit: u32 = data[192] as u32;

        if value_32bit >= 32 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported directory block size log2 value out of bounds: {}",
                value_32bit
            )));
        }
        self.directory_block_size = 1 << value_32bit;

        if self.directory_block_size > u32::MAX / self.block_size {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported directory block size out of bounds: {}",
                self.directory_block_size
            )));
        }
        self.directory_block_size *= self.block_size;

        self.secondary_feature_flags = bytes_to_u32_be!(data, 200);

        let value_32bit: u32 = bytes_to_u32_be!(data, 204);

        if self.secondary_feature_flags != value_32bit {
            return Err(keramics_core::error_trace_new!(format!(
                "Secondary feature flags: 0x{:08x} does not match copy: 0x{:08x}",
                self.secondary_feature_flags, value_32bit
            )));
        }
        if self.format_version >= 5 {
            self.compatible_feature_flags = bytes_to_u32_be!(data, 208);
            self.read_only_compatible_feature_flags = bytes_to_u32_be!(data, 212);
            self.incompatible_feature_flags = bytes_to_u32_be!(data, 216);
            self.journal_incompatible_feature_flags = bytes_to_u32_be!(data, 220);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        vec![
            0x58, 0x46, 0x53, 0x42, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x1a, 0x06, 0xf8, 0x61, 0x55, 0x10, 0x42, 0x99, 0x86, 0xaa,
            0x62, 0x9a, 0x2d, 0x10, 0x94, 0xe4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x3f, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x02, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x07, 0xd1, 0xb4, 0xb5, 0x02, 0x00, 0x02, 0x00, 0x00, 0x08, 0x78, 0x66, 0x73, 0x5f,
            0x74, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x09, 0x09, 0x03, 0x0c, 0x00,
            0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x18, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x8a, 0x00, 0x00, 0x01, 0x8a, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0xe3, 0x00, 0x00, 0x00, 0x00,
            0xb5, 0xbd, 0x18, 0xbf, 0x00, 0x00, 0x00, 0x04, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.root_directory_inode_number, 16128);
        assert_eq!(test_struct.allocation_group_size, 4096);
        assert_eq!(test_struct.number_of_allocation_groups, 1);
        assert_eq!(test_struct.format_version, 5);
        assert_eq!(test_struct.feature_flags, 0xb4b0);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.inode_size, 512);
        assert_eq!(test_struct.inodes_per_block, 8);
        assert_eq!(test_struct.volume_label, ByteString::from("xfs_test"));
        assert_eq!(test_struct.directory_block_size, 4096);
        assert_eq!(test_struct.secondary_feature_flags, 0x0000018a);
        assert_eq!(test_struct.compatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x0000000f);
        assert_eq!(test_struct.incompatible_feature_flags, 0x000000e3);
        assert_eq!(test_struct.journal_incompatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.number_of_relative_block_number_bits, 12);
        assert_eq!(test_struct.number_of_relative_inode_number_bits, 15);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_block_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[4] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
    // TODO: add tests with invalid allocation_group_size

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[101] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_bytes_per_sector() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[102] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_inode_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[104] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    // TODO: add tests with invalid allocation_group_size_log2
    // TODO: add tests with invalid directory_block_size_log2

    #[test]
    fn test_read_data_with_invalid_secondary_feature_flags_copy() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[204] = 0xff;

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = XfsSuperblock::new(&CharacterEncoding::Utf8);
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.root_directory_inode_number, 16128);
        assert_eq!(test_struct.allocation_group_size, 4096);
        assert_eq!(test_struct.number_of_allocation_groups, 1);
        assert_eq!(test_struct.format_version, 5);
        assert_eq!(test_struct.feature_flags, 0xb4b0);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.inode_size, 512);
        assert_eq!(test_struct.inodes_per_block, 8);
        assert_eq!(test_struct.volume_label, ByteString::from("xfs_test"));
        assert_eq!(test_struct.directory_block_size, 4096);
        assert_eq!(test_struct.secondary_feature_flags, 0x0000018a);
        assert_eq!(test_struct.compatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x0000000f);
        assert_eq!(test_struct.incompatible_feature_flags, 0x000000e3);
        assert_eq!(test_struct.journal_incompatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.number_of_relative_block_number_bits, 12);
        assert_eq!(test_struct.number_of_relative_inode_number_bits, 15);

        Ok(())
    }
}
