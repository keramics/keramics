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
use keramics_types::bytes_to_u64_be;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "number_of_blocks", data_type = "BitField128<21>"),
        field(name = "physical_block_number", data_type = "BitField128<52>"),
        field(name = "logical_block_number", data_type = "BitField128<54>"),
        field(name = "uninitialized_flag", data_type = "BitField128<1>"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) packed extent.
pub struct XfsPackedExtent {
    /// Number of blocks.
    pub number_of_blocks: u32,

    /// Physical block number.
    pub physical_block_number: u64,

    /// Logical block number.
    pub logical_block_number: u64,

    /// Uninitialized flag.
    pub uninitialized_flag: u8,
}

impl XfsPackedExtent {
    /// Creates a new packed extent.
    pub fn new() -> Self {
        Self {
            number_of_blocks: 0,
            physical_block_number: 0,
            logical_block_number: 0,
            uninitialized_flag: 0,
        }
    }

    /// Reads the packed extent from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let value_128bit_upper: u64 = bytes_to_u64_be!(data, 0);
        let value_128bit_lower: u64 = bytes_to_u64_be!(data, 8);

        self.number_of_blocks = (value_128bit_lower & 0x1fffff) as u32;
        self.physical_block_number = (value_128bit_lower >> 21) | (value_128bit_upper & 0x1ff);
        self.logical_block_number = (value_128bit_upper >> 9) & 0x3fffffffffffff;
        self.uninitialized_flag = (value_128bit_upper >> 63) as u8;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfb, 0xc0,
            0x00, 0x01,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsPackedExtent::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_blocks, 1);
        assert_eq!(test_struct.physical_block_number, 2014);
        assert_eq!(test_struct.logical_block_number, 0);
        assert_eq!(test_struct.uninitialized_flag, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = XfsPackedExtent::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
