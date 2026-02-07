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
        field(name = "depth", data_type = "u16"),
        field(name = "root_node_number", data_type = "u32"),
        field(name = "number_of_data_records", data_type = "u32"),
        field(name = "first_leaf_node_number", data_type = "u32"),
        field(name = "last_leaf_node_number", data_type = "u32"),
        field(name = "node_size", data_type = "u16"),
        field(name = "maximum_key_size", data_type = "u16"),
        field(name = "number_of_nodes", data_type = "u32"),
        field(name = "number_of_unused_nodes", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "clump_size", data_type = "u32"),
        field(name = "file_type", data_type = "u8"),
        field(name = "key_comparion_method", data_type = "u8"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "unknown2", data_type = "[u8; 64]"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS) B-tree header record.
pub struct HfsBtreeHeaderRecord {
    /// Root node number.
    pub root_node_number: u32,

    /// First leaf node number.
    pub first_leaf_node_number: u32,

    /// Last leaf node number.
    pub last_leaf_node_number: u32,

    /// Node size.
    pub node_size: u16,

    /// Key comparision method.
    pub key_comparion_method: u8,
}

impl HfsBtreeHeaderRecord {
    const SUPPORTED_NODE_SIZES: [u16; 7] = [512, 1024, 2048, 4096, 8192, 16384, 32768];

    /// Creates a new B-tree header record.
    pub fn new() -> Self {
        Self {
            root_node_number: 0,
            first_leaf_node_number: 0,
            last_leaf_node_number: 0,
            node_size: 0,
            key_comparion_method: 0,
        }
    }

    /// Reads the B-tree node descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 106 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.root_node_number = bytes_to_u32_be!(data, 2);
        self.first_leaf_node_number = bytes_to_u32_be!(data, 10);
        self.last_leaf_node_number = bytes_to_u32_be!(data, 14);
        self.node_size = bytes_to_u16_be!(data, 18);

        if !Self::SUPPORTED_NODE_SIZES.contains(&self.node_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported node size: {}",
                self.node_size
            )));
        }
        self.key_comparion_method = data[37];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x08,
            0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x02, 0x04, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x01, 0xf7, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0xcf, 0x00, 0x00, 0x00, 0x06,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsBtreeHeaderRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.root_node_number, 3);
        assert_eq!(test_struct.first_leaf_node_number, 8);
        assert_eq!(test_struct.last_leaf_node_number, 1);
        assert_eq!(test_struct.node_size, 4096);
        assert_eq!(test_struct.key_comparion_method, 0xcf);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsBtreeHeaderRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..105]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_node_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[18] = 0xff;

        let mut test_struct = HfsBtreeHeaderRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
