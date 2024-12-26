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

use crate::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "start_address_chs", data_type = "[u8; 3]"),
        field(name = "partition_type", data_type = "u8"),
        field(name = "end_address_chs", data_type = "[u8; 3]"),
        field(name = "start_address_lba", data_type = "u32"),
        field(name = "number_of_sectors", data_type = "u32"),
    ),
    method(name = "debug_read_data")
)]
/// Master Boot Record (MBR) partition entry.
pub struct MbrPartitionEntry {
    /// The partition index.
    pub index: usize,

    /// The partition flags.
    pub flags: u8,

    /// The partition type.
    pub partition_type: u8,

    /// The start LBA of the partition.
    pub start_address_lba: u32,

    /// The total number of sectors in the partition.
    pub number_of_sectors: u32,
}

impl MbrPartitionEntry {
    /// Creates a new partition entry.
    pub fn new() -> Self {
        Self {
            index: 0,
            flags: 0,
            partition_type: 0,
            start_address_lba: 0,
            number_of_sectors: 0,
        }
    }

    /// Reads the partition entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.flags = data[0];
        self.partition_type = data[4];
        self.start_address_lba = bytes_to_u32_le!(data, 8);
        self.number_of_sectors = bytes_to_u32_le!(data, 12);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x80, 0x20, 0x21, 0x00, 0x07, 0xdf, 0x13, 0x0c, 0x00, 0x08, 0x00, 0x00, 0x00, 0x20,
            0x03, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = MbrPartitionEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x80);
        assert_eq!(test_struct.partition_type, 7);
        assert_eq!(test_struct.start_address_lba, 2048);
        assert_eq!(test_struct.number_of_sectors, 204800);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = MbrPartitionEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
