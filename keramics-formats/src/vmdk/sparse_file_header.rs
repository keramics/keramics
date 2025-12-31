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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "format_version", data_type = "u32"),
        field(name = "flags", data_type = "u32"),
        field(name = "maximum_data_number_of_sectors", data_type = "u64"),
        field(name = "sectors_per_grain", data_type = "u64"),
        field(name = "descriptor_start_sector", data_type = "u64"),
        field(name = "descriptor_size", data_type = "u64"),
        field(name = "number_of_grain_table_entries", data_type = "u32"),
        field(name = "secondary_grain_directory_start_sector", data_type = "u64"),
        field(name = "primary_grain_directory_start_sector", data_type = "u64"),
        field(name = "metadata_size", data_type = "u64"),
        field(name = "is_dirty", data_type = "u8"),
        field(name = "character_values", data_type = "[u8; 4]"),
        field(name = "compression_method", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 433]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// VMware Virtual Disk (VMDK) sparse file header.
pub struct VmdkSparseFileHeader {
    /// Format version.
    pub format_version: u32,

    /// Flags.
    pub flags: u32,

    /// Maximum data number of sectors.
    pub maximum_data_number_of_sectors: u64,

    /// Sectors per grain.
    pub sectors_per_grain: u64,

    /// Descriptor start sector.
    pub descriptor_start_sector: u64,

    /// Descriptor size.
    pub descriptor_size: u64,

    /// Number of grain table entries.
    pub number_of_grain_table_entries: u32,

    /// Secondary grain directory start sector.
    pub secondary_grain_directory_start_sector: u64,

    /// Primary grain directory start sector.
    pub primary_grain_directory_start_sector: u64,
}

impl VmdkSparseFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            flags: 0,
            maximum_data_number_of_sectors: 0,
            sectors_per_grain: 0,
            descriptor_start_sector: 0,
            descriptor_size: 0,
            number_of_grain_table_entries: 0,
            secondary_grain_directory_start_sector: 0,
            primary_grain_directory_start_sector: 0,
        }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..4] != VMDK_SPARSE_FILE_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        if &data[73..77] != [0x0a, 0x20, 0x0d, 0x0a] {
            return Err(keramics_core::error_trace_new!(
                "Unsupported character values"
            ));
        }
        self.format_version = bytes_to_u32_le!(data, 4);
        self.flags = bytes_to_u32_le!(data, 8);
        self.maximum_data_number_of_sectors = bytes_to_u64_le!(data, 12);
        self.sectors_per_grain = bytes_to_u64_le!(data, 20);
        self.descriptor_start_sector = bytes_to_u64_le!(data, 28);
        self.descriptor_size = bytes_to_u64_le!(data, 36);
        self.number_of_grain_table_entries = bytes_to_u32_le!(data, 44);
        self.secondary_grain_directory_start_sector = bytes_to_u64_le!(data, 48);
        self.primary_grain_directory_start_sector = bytes_to_u64_le!(data, 56);

        // sectors per grain & (sectors per grain - 1) is 0 when sectors per grain is a power of 2
        if self.sectors_per_grain < 8 || self.sectors_per_grain & (self.sectors_per_grain - 1) != 0
        {
            return Err(keramics_core::error_trace_new!(
                "Invalid sectors per grain value out of bounds"
            ));
        }
        let supported_flags: u32 = 0x00000001
            | VMDK_SPARSE_FILE_FLAG_USE_SECONDARY_GRAIN_DIRECTORY // 0x00000002
            | 0x00000004
            | VMDK_SPARSE_FILE_FLAG_HAS_GRAIN_COMPRESSION // 0x00010000
            | VMDK_SPARSE_FILE_FLAG_HAS_DATA_MARKERS; // 0x00020000

        if self.flags & !(supported_flags) != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported flags"));
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
        return vec![
            0x4b, 0x44, 0x4d, 0x56, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x20,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x0a, 0x20, 0x0d, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VmdkSparseFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format_version, 1);
        assert_eq!(test_struct.flags, 0x00000003);
        assert_eq!(test_struct.sectors_per_grain, 128);
        assert_eq!(test_struct.descriptor_start_sector, 1);
        assert_eq!(test_struct.descriptor_size, 20);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VmdkSparseFileHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VmdkSparseFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_flags() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[11] = 0xff;

        let mut test_struct = VmdkSparseFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_character_values() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[73] = 0xff;

        let mut test_struct = VmdkSparseFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_sectors_per_grain() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[20] = 0x00;

        let mut test_struct = VmdkSparseFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = VmdkSparseFileHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.format_version, 1);
        assert_eq!(test_struct.flags, 0x00000003);
        assert_eq!(test_struct.sectors_per_grain, 128);
        assert_eq!(test_struct.descriptor_start_sector, 1);
        assert_eq!(test_struct.descriptor_size, 20);

        Ok(())
    }
}
