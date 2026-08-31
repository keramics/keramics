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
use keramics_types::bytes_to_u32_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "start_block_number", data_type = "u32"),
        field(name = "partition_type", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// SGI disklabel (sgilabel) partition entry.
pub struct SgiDiskLabelPartitionEntry {
    /// The entry index.
    pub entry_index: u8,

    /// The number of blocks.
    pub number_of_blocks: u32,

    /// The start block number.
    pub start_block_number: u32,

    /// The partition type.
    pub partition_type: u32,
}

impl SgiDiskLabelPartitionEntry {
    /// Creates a new partition entry.
    pub fn new() -> Self {
        Self {
            entry_index: 0,
            number_of_blocks: 0,
            start_block_number: 0,
            partition_type: 0,
        }
    }

    /// Reads the partition entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 12 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.number_of_blocks = bytes_to_u32_be!(data, 0);
        self.start_block_number = bytes_to_u32_be!(data, 4);
        self.partition_type = bytes_to_u32_be!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = SgiDiskLabelPartitionEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_blocks, 0);
        assert_eq!(test_struct.start_block_number, 0);
        assert_eq!(test_struct.partition_type, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = SgiDiskLabelPartitionEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }
}
