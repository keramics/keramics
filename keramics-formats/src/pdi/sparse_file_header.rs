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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<16>"),
        field(name = "format_version", data_type = "u32"),
        field(name = "number_of_heads", data_type = "u32"),
        field(name = "number_of_cylinders", data_type = "u32"),
        field(name = "sectors_per_block", data_type = "u32"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "number_of_sectors", data_type = "u64"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "data_start_sector", data_type = "u32"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "features_start_sector", data_type = "u64"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Parallels Disk Image (PDI) sparse file header.
pub struct PdiSparseFileHeader {
    /// Format version.
    pub format_version: u32,

    /// Sectors per block.
    pub sectors_per_block: u32,

    /// Number of blocks.
    pub number_of_blocks: u32,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Data start sector.
    pub data_start_sector: u32,
}

impl PdiSparseFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            number_of_blocks: 0,
            sectors_per_block: 0,
            number_of_sectors: 0,
            data_start_sector: 0,
        }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 64 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..16] != PDI_SPARSE_FILE_HEADER_SIGNATURE1
            && &data[0..16] != PDI_SPARSE_FILE_HEADER_SIGNATURE2
        {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.format_version = bytes_to_u32_le!(data, 16);
        self.sectors_per_block = bytes_to_u32_le!(data, 28);
        self.number_of_blocks = bytes_to_u32_le!(data, 32);
        self.number_of_sectors = bytes_to_u64_le!(data, 36);
        self.data_start_sector = bytes_to_u32_le!(data, 48);

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
            0x57, 0x69, 0x74, 0x68, 0x6f, 0x75, 0x74, 0x46, 0x72, 0x65, 0x65, 0x53, 0x70, 0x61,
            0x63, 0x65, 0x02, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x68, 0x06, 0x00, 0x00,
            0x00, 0x08, 0x00, 0x00, 0x9a, 0x01, 0x00, 0x00, 0x00, 0xd0, 0x0c, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x70, 0x64, 0x32, 0x32, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = PdiSparseFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format_version, 2);
        assert_eq!(test_struct.sectors_per_block, 2048);
        assert_eq!(test_struct.number_of_blocks, 410);
        assert_eq!(test_struct.number_of_sectors, 839680);
        assert_eq!(test_struct.data_start_sector, 2048);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = PdiSparseFileHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..63]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = PdiSparseFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = PdiSparseFileHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.format_version, 2);
        assert_eq!(test_struct.sectors_per_block, 2048);
        assert_eq!(test_struct.number_of_blocks, 410);
        assert_eq!(test_struct.number_of_sectors, 839680);
        assert_eq!(test_struct.data_start_sector, 2048);

        Ok(())
    }
}
