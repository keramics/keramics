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
use keramics_types::{bytes_to_u16_be, bytes_to_u32_be};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "level", data_type = "u16"),
        field(name = "number_of_records", data_type = "u16"),
        field(name = "previous_btree_block_number", data_type = "u32"),
        field(name = "next_btree_block_number", data_type = "u32"),
        group(
            size_condition = ">= 56",
            field(name = "block_number", data_type = "u64"),
            field(name = "log_sequence_number", data_type = "u64"),
            field(name = "block_type_identifier", data_type = "Uuid"),
            field(name = "owner_allocation_group", data_type = "u32"),
            field(name = "checksum", data_type = "u32", format = "hex"),
        )
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) B-Tree node header 32-bit.
pub struct XfsBtreeNodeHeader32bit {
    /// Signature.
    pub signature: Vec<u8>,

    /// Level.
    pub level: u16,

    /// Number of records.
    pub number_of_records: u16,

    /// Previous B-tree block number.
    pub previous_btree_block_number: u32,

    /// Next B-tree block number.
    pub next_btree_block_number: u32,
}

impl XfsBtreeNodeHeader32bit {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            signature: Vec::new(),
            level: 0,
            number_of_records: 0,
            previous_btree_block_number: 0,
            next_btree_block_number: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size != 16 && data_size != 56 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.signature = data[0..4].to_vec();
        self.level = bytes_to_u16_be!(data, 4);
        self.number_of_records = bytes_to_u16_be!(data, 6);
        self.previous_btree_block_number = bytes_to_u32_be!(data, 8);
        self.next_btree_block_number = bytes_to_u32_be!(data, 12);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x49, 0x41, 0x42, 0x33, 0x00, 0x00, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBtreeNodeHeader32bit::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.signature, &test_data[0..4]);
        assert_eq!(test_struct.level, 0);
        assert_eq!(test_struct.number_of_records, 1);
        assert_eq!(test_struct.next_btree_block_number, 0xffffffff);
        assert_eq!(test_struct.next_btree_block_number, 0xffffffff);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBtreeNodeHeader32bit::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
