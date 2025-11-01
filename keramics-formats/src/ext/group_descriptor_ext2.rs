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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

use super::group_descriptor::ExtGroupDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "block_bitmap_block_number", data_type = "u32"),
        field(name = "inode_bitmap_block_number", data_type = "u32"),
        field(name = "inode_table_block_number", data_type = "u32"),
        field(name = "number_of_unallocated_blocks", data_type = "u16"),
        field(name = "number_of_unallocated_inodes", data_type = "u16"),
        field(name = "number_of_directories", data_type = "u16"),
        field(name = "padding1", data_type = "[u8; 2]"),
        field(name = "unknown1", data_type = "[u8; 12]"),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext2 and ext3) group descriptor.
pub struct Ext2GroupDescriptor {}

impl Ext2GroupDescriptor {
    /// Reads the group descriptor from a buffer.
    pub fn read_data(
        group_descriptor: &mut ExtGroupDescriptor,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() != 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported group descriptor data size"
            ));
        }
        group_descriptor.inode_table_block_number = bytes_to_u32_le!(data, 8) as u64;
        group_descriptor.checksum = bytes_to_u16_le!(data, 30);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x12, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x58, 0x0f,
            0xf0, 0x03, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        Ext2GroupDescriptor::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.inode_table_block_number, 20);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = Ext2GroupDescriptor::read_data(&mut test_struct, &test_data[0..31]);
        assert!(result.is_err());
    }
}
