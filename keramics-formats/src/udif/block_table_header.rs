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
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "format_version", data_type = "u32"),
        field(name = "start_sector", data_type = "u64"),
        field(name = "number_of_sectors", data_type = "u64"),
        field(name = "unknown1", data_type = "[u8; 40]"),
        field(name = "checksum_type", data_type = "u32"),
        field(name = "checksum_size", data_type = "u32"),
        field(name = "checksum", data_type = "[u8; 128]"),
        field(name = "number_of_entries", data_type = "u32"),
    ),
    method(name = "debug_read_data")
)]
/// Universal Disk Image Format (UDIF) block table header.
pub struct UdifBlockTableHeader {
    /// Start sector.
    pub start_sector: u64,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Number of entries.
    pub number_of_entries: u32,
}

impl UdifBlockTableHeader {
    /// Creates a new block table header.
    pub fn new() -> Self {
        Self {
            start_sector: 0,
            number_of_sectors: 0,
            number_of_entries: 0,
        }
    }

    /// Reads the block table header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 204 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported UDIF block table header data size"),
            ));
        }
        if data[0..4] != UDIF_BLOCK_TABLE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported UDIF block table header signature"),
            ));
        }
        let format_version: u32 = bytes_to_u32_be!(data, 4);

        if format_version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported format version: {}", format_version),
            ));
        }
        self.start_sector = bytes_to_u64_be!(data, 8);
        self.number_of_sectors = bytes_to_u64_be!(data, 16);
        self.number_of_entries = bytes_to_u32_be!(data, 200);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x6d, 0x69, 0x73, 0x68, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x20, 0x41, 0xf2, 0xfa, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifBlockTableHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.start_sector, 0);
        assert_eq!(test_struct.number_of_sectors, 1);
        assert_eq!(test_struct.number_of_entries, 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = UdifBlockTableHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..203]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = UdifBlockTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
