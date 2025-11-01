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
use keramics_types::{bytes_to_u32_be, bytes_to_u64_be};

use super::constants::*;
use super::file_header::QcowFileHeader;

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
    ),
    method(name = "debug_read_data")
)]
/// QEMU Copy-On-Write (QCOW) file header version 2.
pub struct QcowFileHeaderV2 {}

impl QcowFileHeaderV2 {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the file header from a buffer.
    pub fn read_data(file_header: &mut QcowFileHeader, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 72 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported QCOW file header version 2 data size"
            ));
        }
        if data[0..4] != QCOW_FILE_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported QCOW file header version 2 signature"
            ));
        }
        let format_version: u32 = bytes_to_u32_be!(data, 4);

        if format_version != 2 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format version: {}",
                format_version
            )));
        }
        file_header.backing_file_name_offset = bytes_to_u64_be!(data, 8);
        file_header.backing_file_name_size = bytes_to_u32_be!(data, 16);
        file_header.number_of_cluster_block_bits = bytes_to_u32_be!(data, 20);
        file_header.media_size = bytes_to_u64_be!(data, 24);
        file_header.encryption_method = bytes_to_u32_be!(data, 32);
        file_header.level1_table_number_of_references = bytes_to_u32_be!(data, 36);
        file_header.level1_table_offset = bytes_to_u64_be!(data, 40);
        file_header.number_of_snapshots = bytes_to_u32_be!(data, 60);
        file_header.snapshots_offset = bytes_to_u64_be!(data, 64);

        if file_header.number_of_cluster_block_bits <= 8
            || file_header.number_of_cluster_block_bits > 63
        {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of cluster block bits: {} value out of bounds",
                file_header.number_of_cluster_block_bits
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowFileHeader::new();
        QcowFileHeaderV2::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.backing_file_name_offset, 0);
        assert_eq!(test_struct.backing_file_name_size, 0);
        assert_eq!(test_struct.number_of_cluster_block_bits, 16);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.encryption_method, 0);
        assert_eq!(test_struct.level1_table_number_of_references, 1);
        assert_eq!(test_struct.level1_table_offset, 196608);
        assert_eq!(test_struct.number_of_snapshots, 0);
        assert_eq!(test_struct.snapshots_offset, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowFileHeader::new();
        let result = QcowFileHeaderV2::read_data(&mut test_struct, &test_data[0..71]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = QcowFileHeader::new();
        let result = QcowFileHeaderV2::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[4] = 0xff;

        let mut test_struct = QcowFileHeader::new();
        let result = QcowFileHeaderV2::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_number_of_cluster_block_bits() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[20] = 0xff;

        let mut test_struct = QcowFileHeader::new();
        let result = QcowFileHeaderV2::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
