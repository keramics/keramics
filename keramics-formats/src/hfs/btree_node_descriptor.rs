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

use super::enums::HfsBtreeNodeType;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "next_node_number", data_type = "u32"),
        field(name = "previous_node_number", data_type = "u32"),
        field(name = "node_type", data_type = "u8"),
        field(name = "node_level", data_type = "u8"),
        field(name = "number_of_records", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 2]"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS) B-tree node descriptor.
pub struct HfsBtreeNodeDescriptor {
    /// Next node number.
    pub next_node_number: u32,

    /// Previous node number.
    pub previous_node_number: u32,

    /// Node type.
    pub node_type: HfsBtreeNodeType,

    /// Node level.
    pub node_level: u8,

    /// Number of records.
    pub number_of_records: u16,
}

impl HfsBtreeNodeDescriptor {
    /// Creates a new B-tree node descriptor.
    pub fn new() -> Self {
        Self {
            next_node_number: 0,
            previous_node_number: 0,
            node_type: HfsBtreeNodeType::LeafNode,
            node_level: 0,
            number_of_records: 0,
        }
    }

    /// Reads the B-tree node descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 14 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.next_node_number = bytes_to_u32_be!(data, 0);
        self.previous_node_number = bytes_to_u32_be!(data, 4);
        self.node_type = match data[8] {
            0x00 => HfsBtreeNodeType::IndexNode,
            0x01 => HfsBtreeNodeType::HeaderNode,
            0x02 => HfsBtreeNodeType::MapNode,
            0xff => HfsBtreeNodeType::LeafNode,
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported node type value: {}",
                    data[8]
                )));
            }
        };
        self.node_level = data[9];
        self.number_of_records = bytes_to_u16_be!(data, 10);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x03, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsBtreeNodeDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.next_node_number, 0);
        assert_eq!(test_struct.previous_node_number, 0);
        assert_eq!(test_struct.node_type, HfsBtreeNodeType::HeaderNode);
        assert_eq!(test_struct.node_level, 0);
        assert_eq!(test_struct.number_of_records, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsBtreeNodeDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..13]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_node_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[8] = 0xaa;

        let mut test_struct = HfsBtreeNodeDescriptor::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
