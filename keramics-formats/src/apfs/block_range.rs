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
        field(name = "block_number", data_type = "u64"),
        field(name = "number_of_blocks", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) block range.
pub struct ApfsBlockRange {
    /// Block number.
    pub block_number: u64,

    /// Number of blocks.
    pub number_of_blocks: u64,
}

impl ApfsBlockRange {
    /// Creates a new block range.
    pub fn new() -> Self {
        Self {
            block_number: 0,
            number_of_blocks: 0,
        }
    }

    /// Reads the block range from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.block_number = bytes_to_u64_le!(data, 0);
        self.number_of_blocks = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsBlockRange::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_number, 1);
        assert_eq!(test_struct.number_of_blocks, 6);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsBlockRange::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
