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

use keramics_types::bytes_to_u16_le;
use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<8>"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "number_of_entries", data_type = "u16"),
        field(name = "unknown2", data_type = "[u8; 20]"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) metadata table header.
pub struct VhdxMetadataTableHeader {
    /// Number of entries.
    pub number_of_entries: u16,
}

impl VhdxMetadataTableHeader {
    /// Creates a new metadata table header.
    pub fn new() -> Self {
        Self {
            number_of_entries: 0,
        }
    }

    /// Reads the metadata table header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported VHDX metadata table header data size"),
            ));
        }
        if data[0..8] != VHDX_METADATA_TABLE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported VHDX metadata table header signature"),
            ));
        }
        self.number_of_entries = bytes_to_u16_le!(data, 10);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxMetadataTableHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_entries, 5);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxMetadataTableHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VhdxMetadataTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
