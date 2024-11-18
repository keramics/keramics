/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "entry_type", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "start_sector", data_type = "u64"),
        field(name = "number_of_sectors", data_type = "u64"),
        field(name = "data_offset", data_type = "u64", format = "hex"),
        field(name = "data_size", data_type = "u64"),
    ),
    method(name = "debug_read_data")
)]
/// Universal Disk Image Format (UDIF) block table entry.
pub struct UdifBlockTableEntry {
    /// Entry type.
    pub entry_type: u32,

    /// Start sector.
    pub start_sector: u64,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Data offset.
    pub data_offset: u64,

    /// Data size.
    pub data_size: u64,
}

impl UdifBlockTableEntry {
    /// Creates a new block table entry.
    pub fn new() -> Self {
        Self {
            entry_type: 0,
            start_sector: 0,
            number_of_sectors: 0,
            data_offset: 0,
            data_size: 0,
        }
    }

    /// Reads the block table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 40 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.entry_type = crate::bytes_to_u32_be!(data, 0);
        self.start_sector = crate::bytes_to_u64_be!(data, 8);
        self.number_of_sectors = crate::bytes_to_u64_be!(data, 16);
        self.data_offset = crate::bytes_to_u64_be!(data, 24);
        self.data_size = crate::bytes_to_u64_be!(data, 32);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x80, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x20, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data = get_test_data();

        let mut test_struct = UdifBlockTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entry_type, 0x80000005);
        assert_eq!(test_struct.start_sector, 0);
        assert_eq!(test_struct.number_of_sectors, 1);
        assert_eq!(test_struct.data_offset, 8205);
        assert_eq!(test_struct.data_size, 31);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = UdifBlockTableEntry::new();

        let test_data = get_test_data();
        let result = test_struct.read_data(&test_data[0..39]);
        assert!(result.is_err());
    }
}
