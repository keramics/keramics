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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<8>"),
        field(name = "size", data_type = "u16"),
        field(name = "format_version", data_type = "u16"),
        field(name = "unknown1", data_type = "u16"),
        field(name = "unknown2", data_type = "u16"),
        field(name = "encrypted_volume_size", data_type = "u64"),
        field(name = "unknown3", data_type = "[u8; 4]"),
        field(name = "boot_record_number_of_sectors", data_type = "u32"),
        field(name = "metadata_block_offset1", data_type = "u64", format = "hex"),
        field(name = "metadata_block_offset2", data_type = "u64", format = "hex"),
        field(name = "metadata_block_offset3", data_type = "u64", format = "hex"),
        field(name = "boot_record_offset", data_type = "u64", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) metadata block header.
pub struct BdeMetadataBlockHeader {
    /// Format version.
    pub format_version: u16,

    /// Volume size.
    pub volume_size: u64,

    /// Encrypted volume size.
    pub encrypted_volume_size: u64,

    /// Boot record number of sectors.
    pub boot_record_number_of_sectors: u32,

    /// Metadata block offset 1.
    pub metadata_block_offset1: u64,

    /// Metadata block offset 2.
    pub metadata_block_offset2: u64,

    /// Metadata block offset 3.
    pub metadata_block_offset3: u64,

    /// Boot record offset.
    pub boot_record_offset: u64,
}

impl BdeMetadataBlockHeader {
    /// Creates a new metadata block header.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            volume_size: 0,
            encrypted_volume_size: 0,
            boot_record_number_of_sectors: 0,
            metadata_block_offset1: 0,
            metadata_block_offset2: 0,
            metadata_block_offset3: 0,
            boot_record_offset: 0,
        }
    }

    /// Reads the metadata block header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 64 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..8] != BDE_FILE_SYSTEM_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.format_version = bytes_to_u16_le!(data, 10);

        if self.format_version > 1 {
            let status1: u16 = bytes_to_u16_le!(data, 12);
            let status2: u16 = bytes_to_u16_le!(data, 14);

            self.encrypted_volume_size = bytes_to_u64_le!(data, 16);

            if status1 == 4 && status2 == 4 {
                self.volume_size = self.encrypted_volume_size;
            }
            self.boot_record_number_of_sectors = bytes_to_u32_le!(data, 28);
        }
        self.metadata_block_offset1 = bytes_to_u64_le!(data, 32);
        self.metadata_block_offset2 = bytes_to_u64_le!(data, 40);
        self.metadata_block_offset3 = bytes_to_u64_le!(data, 48);

        if self.format_version > 1 {
            self.boot_record_offset = bytes_to_u64_le!(data, 56);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x2d, 0x46, 0x56, 0x45, 0x2d, 0x46, 0x53, 0x2d, 0x24, 0x00, 0x02, 0x00, 0x04, 0x00,
            0x04, 0x00, 0x00, 0x00, 0xef, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60,
            0x94, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataBlockHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format_version, 2);
        assert_eq!(test_struct.volume_size, 65994752);
        assert_eq!(test_struct.encrypted_volume_size, 65994752);
        assert_eq!(test_struct.boot_record_number_of_sectors, 16);
        assert_eq!(test_struct.metadata_block_offset1, 0x021f0000);
        assert_eq!(test_struct.metadata_block_offset2, 0x02946000);
        assert_eq!(test_struct.metadata_block_offset3, 0x0309b000);
        assert_eq!(test_struct.boot_record_offset, 0x02200000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataBlockHeader::new();
        let result = test_struct.read_data(&test_data[0..63]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[3] = 0xff;

        let mut test_struct = BdeMetadataBlockHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
