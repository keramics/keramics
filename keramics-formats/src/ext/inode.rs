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

use std::cmp::max;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::{DateTime, PosixTime32, PosixTime64Ns};

use super::attribute::ExtAttribute;
use super::block_numbers_tree::ExtBlockNumbersTree;
use super::block_range::ExtBlockRange;
use super::constants::*;
use super::extents_tree::ExtExtentsTree;
use super::inode_ext2::Ext2Inode;
use super::inode_ext3::Ext3Inode;
use super::inode_ext4::Ext4Inode;
use super::inode_extension_ext4::Ext4InodeExtension;

/// Extended File System inode.
pub struct ExtInode {
    /// File mode.
    pub file_mode: u16,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// Data size.
    pub data_size: u64,

    /// Access timestamp.
    pub(super) access_timestamp: i32,

    /// Access date and time.
    pub access_time: DateTime,

    /// Change timestamp.
    pub(super) change_timestamp: i32,

    /// Change date and time.
    pub change_time: DateTime,

    /// Modification timestamp.
    pub(super) modification_timestamp: i32,

    /// Modification date and time.
    pub modification_time: DateTime,

    /// Deletion date and time.
    pub deletion_time: DateTime,

    /// Number of links.
    pub number_of_links: u16,

    /// Number of block.
    pub number_of_blocks: u64,

    /// Flags.
    pub flags: u32,

    /// Data reference.
    pub data_reference: [u8; 60],

    /// Checksum.
    pub checksum: u32,

    /// Creation date and time.
    pub creation_time: Option<DateTime>,

    /// Block ranges.
    pub block_ranges: Vec<ExtBlockRange>,

    /// Attributes.
    pub attributes: Vec<ExtAttribute>,
}

impl ExtInode {
    /// Creates a new inode.
    pub fn new() -> Self {
        Self {
            file_mode: 0,
            owner_identifier: 0,
            group_identifier: 0,
            data_size: 0,
            access_timestamp: 0,
            access_time: DateTime::NotSet,
            change_timestamp: 0,
            change_time: DateTime::NotSet,
            modification_timestamp: 0,
            modification_time: DateTime::NotSet,
            deletion_time: DateTime::NotSet,
            number_of_links: 0,
            number_of_blocks: 0,
            flags: 0,
            data_reference: [0; 60],
            checksum: 0,
            creation_time: None,
            block_ranges: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Reads the inode for debugging.
    pub fn debug_read_data(&self, format_version: u8, data: &[u8]) -> String {
        let mut string_parts: Vec<String> = Vec::new();

        let string: String = match format_version {
            4 => Ext4Inode::debug_read_data(data),
            3 => Ext3Inode::debug_read_data(data),
            _ => Ext2Inode::debug_read_data(data),
        };
        string_parts.push(string);

        if data.len() > 128 {
            let string: String = Ext4InodeExtension::debug_read_data(&data[128..]);
            string_parts.push(string);
        }
        string_parts.join("")
    }

    /// Reads the inode from a buffer.
    pub fn read_data(&mut self, format_version: u8, data: &[u8]) -> Result<(), ErrorTrace> {
        match format_version {
            4 => {
                Ext4Inode::read_data(self, data)?;
            }
            3 => {
                Ext3Inode::read_data(self, data)?;
            }
            _ => {
                Ext2Inode::read_data(self, data)?;
            }
        }
        if data.len() > 128 {
            Ext4InodeExtension::read_data(self, &data[128..])?;
        }
        Ok(())
    }

    /// Reads the data reference.
    pub fn read_data_reference(
        &mut self,
        format_version: u8,
        data_stream: &DataStreamReference,
        block_size: u32,
    ) -> Result<(), ErrorTrace> {
        let file_mode_type: u16 = self.file_mode & 0xf000;

        if format_version == 4 && self.flags & EXT_INODE_FLAG_INLINE_DATA != 0 {
            // The data is stored inline in self.data_reference
            // Note that self.data_size can be larger than 60
        } else if file_mode_type == EXT_FILE_MODE_TYPE_SYMBOLIC_LINK && self.data_size < 60 {
            // The symbolic link target path is stored in self.data_reference
        }
        match file_mode_type {
            EXT_FILE_MODE_TYPE_CHARACTER_DEVICE | EXT_FILE_MODE_TYPE_BLOCK_DEVICE => {
                // The major and minor device numbers are stored in self.data_reference
            }
            _ => {
                // Note that the number of blocks stored in the inode does not always contain
                // the total number of blocks e.g. when the inode has leading sparse data.
                let number_of_blocks: u64 = max(
                    self.data_size.div_ceil(block_size as u64),
                    self.number_of_blocks,
                );

                if format_version == 4 && self.flags & EXT_INODE_FLAG_HAS_EXTENTS != 0 {
                    let mut extents_tree: ExtExtentsTree =
                        ExtExtentsTree::new(block_size, number_of_blocks);

                    match extents_tree.read_data_reference(
                        &self.data_reference,
                        data_stream,
                        &mut self.block_ranges,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read extents tree"
                            );
                            return Err(error);
                        }
                    }
                } else {
                    let mut block_numbers_tree: ExtBlockNumbersTree =
                        ExtBlockNumbersTree::new(block_size, number_of_blocks);

                    match block_numbers_tree.read_data_reference(
                        &self.data_reference,
                        data_stream,
                        &mut self.block_ranges,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read block numbers"
                            );
                            return Err(error);
                        }
                    }
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_ext2() -> Vec<u8> {
        return vec![
            0xed, 0x41, 0xe8, 0x03, 0x00, 0x04, 0x00, 0x00, 0xa3, 0x7b, 0xf9, 0x60, 0xa4, 0x7b,
            0xf9, 0x60, 0xa4, 0x7b, 0xf9, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x94, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    fn get_test_data_ext3() -> Vec<u8> {
        return vec![
            0xed, 0x41, 0x00, 0x00, 0x3d, 0x13, 0xc1, 0x3f, 0x44, 0x13, 0xc1, 0x3f, 0x44, 0x13,
            0xc1, 0x3f, 0x00, 0x00, 0x00, 0x00, 0xf4, 0x01, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa5, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    fn get_test_data_ext4() -> Vec<u8> {
        return vec![
            0xed, 0x41, 0xe8, 0x03, 0x00, 0x04, 0x00, 0x00, 0xdf, 0xc3, 0xd7, 0x49, 0xdf, 0xc3,
            0xd7, 0x49, 0xdf, 0xc3, 0xd7, 0x49, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x03, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data_ext2() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        test_struct.read_data(2, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    #[test]
    fn test_read_data_ext3() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext3();
        test_struct.read_data(3, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    // TODO: add test for ext3 inode > 128 bytes

    #[test]
    fn test_read_data_ext4() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext4();
        test_struct.read_data(4, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    // TODO: add test for ext4 inode > 128 bytes
}
