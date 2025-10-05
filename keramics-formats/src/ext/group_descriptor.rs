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

use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

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
struct Ext2GroupDescriptor {}

impl Ext2GroupDescriptor {
    /// Creates a new group descriptor.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the group descriptor from a buffer.
    pub fn read_data(
        &self,
        group_descriptor: &mut ExtGroupDescriptor,
        data: &[u8],
    ) -> io::Result<()> {
        if data.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported ext group descriptor data size"),
            ));
        }
        group_descriptor.inode_table_block_number = bytes_to_u32_le!(data, 8) as u64;
        group_descriptor.checksum = bytes_to_u16_le!(data, 30);

        Ok(())
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "block_bitmap_block_number_lower", data_type = "u32")),
        member(field(name = "inode_bitmap_block_number_lower", data_type = "u32")),
        member(field(name = "inode_table_block_number_lower", data_type = "u32")),
        member(field(name = "number_of_unallocated_blocks_lower", data_type = "u16")),
        member(field(name = "number_of_unallocated_inodes_lower", data_type = "u16")),
        member(field(name = "number_of_directories_lower", data_type = "u16")),
        member(field(name = "block_group_flags", data_type = "u16", format = "hex")),
        member(field(name = "exclude_bitmap_block_number", data_type = "u32")),
        member(field(
            name = "block_bitmap_checksum_lower",
            data_type = "u16",
            format = "hex"
        )),
        member(field(
            name = "inode_bitmap_checksum_lower",
            data_type = "u16",
            format = "hex"
        )),
        member(field(name = "number_of_unused_inodes_lower", data_type = "u16")),
        member(field(name = "checksum", data_type = "u16", format = "hex")),
        member(group(
            size_condition = "> 32",
            field(name = "block_bitmap_block_number_upper", data_type = "u32"),
            field(name = "inode_bitmap_block_number_upper", data_type = "u32"),
            field(name = "inode_table_block_number_upper", data_type = "u32"),
            field(name = "number_of_unallocated_blocks_upper", data_type = "u16"),
            field(name = "number_of_unallocated_inodes_upper", data_type = "u16"),
            field(name = "number_of_directories_upper", data_type = "u16"),
            field(name = "number_of_unused_inodes_upper", data_type = "u16"),
            field(name = "exclude_bitmap_block_number_upper", data_type = "u32"),
            field(
                name = "block_bitmap_checksum_upper",
                data_type = "u16",
                format = "hex"
            ),
            field(
                name = "inode_bitmap_checksum_upper",
                data_type = "u16",
                format = "hex"
            ),
            field(name = "padding1", data_type = "[u8; 4]"),
        )),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext4) group descriptor.
struct Ext4GroupDescriptor {}

impl Ext4GroupDescriptor {
    /// Creates a new group descriptor.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the group descriptor from a buffer.
    pub fn read_data(
        &self,
        group_descriptor: &mut ExtGroupDescriptor,
        data: &[u8],
    ) -> io::Result<()> {
        let data_size: usize = data.len();
        if data_size != 32 && data_size != 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported ext group descriptor data size"),
            ));
        }
        let lower_32bit: u32 = bytes_to_u32_le!(data, 8);
        group_descriptor.inode_table_block_number = lower_32bit as u64;
        if data_size >= 44 {
            let upper_32bit: u32 = bytes_to_u32_le!(data, 40);
            group_descriptor.inode_table_block_number |= (upper_32bit as u64) << 32;
        }
        group_descriptor.checksum = bytes_to_u16_le!(data, 30);

        Ok(())
    }
}

/// Extended File System group descriptor.
pub struct ExtGroupDescriptor {
    /// Inode table block number.
    pub inode_table_block_number: u64,

    /// Checksum.
    pub checksum: u16,
}

impl ExtGroupDescriptor {
    /// Creates a new group descriptor.
    pub fn new() -> Self {
        Self {
            inode_table_block_number: 0,
            checksum: 0,
        }
    }

    /// Reads the group descriptor for debugging.
    pub fn debug_read_data(&self, format_version: u8, data: &[u8]) -> String {
        if format_version == 4 {
            Ext4GroupDescriptor::debug_read_data(data)
        } else {
            Ext2GroupDescriptor::debug_read_data(data)
        }
    }

    /// Reads the group descriptor from a buffer.
    pub fn read_data(&mut self, format_version: u8, data: &[u8]) -> io::Result<()> {
        if format_version == 4 {
            let group_descriptor: Ext4GroupDescriptor = Ext4GroupDescriptor::new();
            group_descriptor.read_data(self, data)
        } else {
            let group_descriptor: Ext2GroupDescriptor = Ext2GroupDescriptor::new();
            group_descriptor.read_data(self, data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_ext2() -> Vec<u8> {
        return vec![
            0x12, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x58, 0x0f,
            0xf0, 0x03, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    fn get_test_data_ext4_32bit() -> Vec<u8> {
        return vec![
            0x22, 0x00, 0x00, 0x00, 0x32, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0xc9, 0x0a,
            0xf0, 0x03, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x28, 0x60, 0x74,
            0xf0, 0x03, 0x63, 0x33,
        ];
    }

    fn get_test_data_ext4_64bit() -> Vec<u8> {
        return vec![
            0x22, 0x00, 0x00, 0x00, 0x32, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0xc9, 0x0a,
            0xf0, 0x03, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x28, 0x60, 0x74,
            0xf0, 0x03, 0x63, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xad, 0xe4, 0xe8, 0x8c, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data_ext2() -> io::Result<()> {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        test_struct.read_data(2, &test_data)?;

        assert_eq!(test_struct.inode_table_block_number, 20);

        Ok(())
    }

    #[test]
    fn test_read_data_ext2_with_unsupported_data_size() {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        let result = test_struct.read_data(2, &test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_ext4_32bit() -> io::Result<()> {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext4_32bit();
        test_struct.read_data(4, &test_data)?;

        assert_eq!(test_struct.inode_table_block_number, 66);

        Ok(())
    }

    #[test]
    fn test_read_data_ext4_32bit_with_unsupported_data_size() {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext4_32bit();
        let result = test_struct.read_data(4, &test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_ext4_64bit() -> io::Result<()> {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext4_64bit();
        test_struct.read_data(4, &test_data)?;

        assert_eq!(test_struct.inode_table_block_number, 66);

        Ok(())
    }

    #[test]
    fn test_read_data_ext4_64bit_with_unsupported_data_size() {
        let mut test_struct = ExtGroupDescriptor::new();

        let test_data: Vec<u8> = get_test_data_ext4_64bit();
        let result = test_struct.read_data(4, &test_data[0..63]);
        assert!(result.is_err());
    }
}
