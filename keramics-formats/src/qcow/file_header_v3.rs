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

use std::io;

use keramics_types::{bytes_to_u32_be, bytes_to_u64_be};
use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "[u8; 4]", format = "hex"),
        field(name = "format_version", data_type = "u32"),
        field(name = "backing_file_name_offset", data_type = "u64"),
        field(name = "backing_file_name_size", data_type = "u32"),
        field(name = "number_of_cluster_block_bits", data_type = "u32"),
        field(name = "media_size", data_type = "u64"),
        field(name = "encryption_method", data_type = "u32"),
        field(name = "level1_table_number_of_references", data_type = "u32"),
        field(name = "level1_table_offset", data_type = "u64"),
        field(name = "reference_count_table_offset", data_type = "u64"),
        field(name = "reference_count_table_clusters", data_type = "u32"),
        field(name = "number_of_snapshots", data_type = "u32"),
        field(name = "snapshots_offset", data_type = "u64"),
        field(name = "incompatible_feature_flags", data_type = "u64"),
        field(name = "compatible_feature_flags", data_type = "u64"),
        field(name = "auto_clear_feature_flags", data_type = "u64"),
        field(name = "reference_count_order", data_type = "u32"),
        field(name = "header_size", data_type = "u32"),
        field(name = "compression_method", data_type = "u8"),
        field(name = "unknown1", data_type = "[u8; 7]"),
    ),
    method(name = "debug_read_data")
)]
/// QEMU Copy-On-Write (QCOW) file header version 3.
pub struct QcowFileHeaderV3 {
    pub backing_file_name_offset: u64,
    pub backing_file_name_size: u32,
    pub number_of_cluster_block_bits: u32,
    pub media_size: u64,
    pub encryption_method: u32,
    pub level1_table_number_of_references: u32,
    pub level1_table_offset: u64,
    pub number_of_snapshots: u32,
    pub snapshots_offset: u64,
    pub header_size: u32,
    pub compression_method: u8,
}

impl QcowFileHeaderV3 {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            backing_file_name_offset: 0,
            backing_file_name_size: 0,
            number_of_cluster_block_bits: 0,
            media_size: 0,
            encryption_method: 0,
            level1_table_number_of_references: 0,
            level1_table_offset: 0,
            number_of_snapshots: 0,
            snapshots_offset: 0,
            header_size: 0,
            compression_method: 0,
        }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 104 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != QCOW_FILE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported QCOW file header version 3 signature"),
            ));
        }
        let format_version: u32 = bytes_to_u32_be!(data, 4);

        if format_version != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported format version: {}", format_version),
            ));
        }
        let supported_flags: u64 = 1;

        let incompatible_feature_flags: u64 = bytes_to_u64_be!(data, 72);

        if incompatible_feature_flags & !(supported_flags) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported incompatible feature flags: 0x{:016x}",
                    incompatible_feature_flags
                ),
            ));
        }
        let compatible_feature_flags: u64 = bytes_to_u64_be!(data, 80);

        if compatible_feature_flags & !(supported_flags) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported compatible feature flags: 0x{:016x}",
                    compatible_feature_flags
                ),
            ));
        }
        self.backing_file_name_offset = bytes_to_u64_be!(data, 8);
        self.backing_file_name_size = bytes_to_u32_be!(data, 16);
        self.number_of_cluster_block_bits = bytes_to_u32_be!(data, 20);
        self.media_size = bytes_to_u64_be!(data, 24);
        self.encryption_method = bytes_to_u32_be!(data, 32);
        self.level1_table_number_of_references = bytes_to_u32_be!(data, 36);
        self.level1_table_offset = bytes_to_u64_be!(data, 40);
        self.number_of_snapshots = bytes_to_u32_be!(data, 60);
        self.snapshots_offset = bytes_to_u64_be!(data, 64);
        self.header_size = bytes_to_u32_be!(data, 100);
        self.compression_method = data[104];

        if self.number_of_cluster_block_bits <= 8 || self.number_of_cluster_block_bits > 63 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid number of cluster block bits: {} value out of bounds",
                    self.number_of_cluster_block_bits
                ),
            ));
        }
        if self.header_size != 104 && self.header_size != 112 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported header size: {}", self.header_size),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowFileHeaderV3::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data[0..103]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[4] = 0xff;

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_number_of_cluster_block_bits() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[20] = 0xff;

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_incompatible_feature_flags() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[72] = 0xff;

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_compatible_feature_flags() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[80] = 0xff;

        let mut test_struct = QcowFileHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
