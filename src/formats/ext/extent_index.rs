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

use layout_map::LayoutMap;

use crate::{bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "logical_block_number", data_type = "u32"),
        field(name = "physical_block_number_lower", data_type = "u32"),
        field(name = "physical_block_number_upper", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 2]"),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext) extent index.
pub struct ExtExtentIndex {
    pub logical_block_number: u32,
    pub physical_block_number: u64,
}

impl ExtExtentIndex {
    /// Creates a new extent index.
    pub fn new() -> Self {
        Self {
            logical_block_number: 0,
            physical_block_number: 0,
        }
    }

    /// Reads the extent index from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.logical_block_number = bytes_to_u32_le!(data, 0);

        let lower_32bit: u32 = bytes_to_u32_le!(data, 4);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 8);
        self.physical_block_number = ((upper_16bit as u64) << 32) | (lower_32bit as u64);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtExtentIndex::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.logical_block_number, 0);
        assert_eq!(test_struct.physical_block_number, 7);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtExtentIndex::new();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }
}
