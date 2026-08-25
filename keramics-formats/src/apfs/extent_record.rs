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
use keramics_types::bytes_to_u64_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "extent_size", data_type = "BitField64<60>"),
        field(name = "flags", data_type = "BitField64<4>"),
        field(name = "physical_block_number", data_type = "u64"),
        field(name = "encryption_identifier", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) extent record.
#[derive(Clone)]
pub struct ApfsExtentRecord {
    /// Data size.
    pub extent_size: u64,

    /// Flags.
    pub flags: u8,

    /// Physical block number.
    pub physical_block_number: u64,

    /// Encryption identifier.
    pub encryption_identifier: u64,
}

impl ApfsExtentRecord {
    /// Creates a new extent record.
    pub fn new() -> Self {
        Self {
            extent_size: 0,
            flags: 0,
            physical_block_number: 0,
            encryption_identifier: 0,
        }
    }

    /// Reads the extent record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 24 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let value_64bit: u64 = bytes_to_u64_le!(data, 0);

        self.extent_size = value_64bit & 0x0fffffffffffffff;
        self.flags = (value_64bit >> 60) as u8;
        self.physical_block_number = bytes_to_u64_le!(data, 8);
        self.encryption_identifier = bytes_to_u64_le!(data, 16);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsExtentRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.extent_size, 4096);
        assert_eq!(test_struct.flags, 0x0000);
        assert_eq!(test_struct.physical_block_number, 93);
        assert_eq!(test_struct.encryption_identifier, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsExtentRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_record_data_size() {
        let mut test_struct = ApfsExtentRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }
}
