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

use super::group_descriptor_ext2::Ext2GroupDescriptor;
use super::group_descriptor_ext4::Ext4GroupDescriptor;

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
    pub fn read_data(&mut self, format_version: u8, data: &[u8]) -> Result<(), ErrorTrace> {
        if format_version == 4 {
            Ext4GroupDescriptor::read_data(self, data)
        } else {
            Ext2GroupDescriptor::read_data(self, data)
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
    fn test_read_data_ext2() -> Result<(), ErrorTrace> {
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
    fn test_read_data_ext4_32bit() -> Result<(), ErrorTrace> {
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
    fn test_read_data_ext4_64bit() -> Result<(), ErrorTrace> {
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
