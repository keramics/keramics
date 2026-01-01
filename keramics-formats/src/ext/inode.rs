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

use std::cmp::max;
use std::collections::BTreeMap;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_types::ByteString;

use super::attributes_entry::ExtAttributesEntry;
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
    pub access_time: Option<DateTime>,

    /// Change timestamp.
    pub(super) change_timestamp: i32,

    /// Change date and time.
    pub change_time: Option<DateTime>,

    /// Modification timestamp.
    pub(super) modification_timestamp: i32,

    /// Modification date and time.
    pub modification_time: Option<DateTime>,

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

    /// File ACL block number.
    pub file_acl_block_number: u64,

    /// Checksum.
    pub checksum: u32,

    /// Creation date and time.
    pub creation_time: Option<DateTime>,

    /// Attributes.
    pub attributes: BTreeMap<ByteString, ExtAttributesEntry>,
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
            access_time: None,
            change_timestamp: 0,
            change_time: None,
            modification_timestamp: 0,
            modification_time: None,
            deletion_time: DateTime::NotSet,
            number_of_links: 0,
            number_of_blocks: 0,
            flags: 0,
            data_reference: [0; 60],
            file_acl_block_number: 0,
            checksum: 0,
            creation_time: None,
            attributes: BTreeMap::new(),
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

    /// Determines if the inode is has inline data.
    pub fn has_inline_data(&self) -> bool {
        self.flags & EXT_INODE_FLAG_INLINE_DATA != 0
    }

    /// Reads the block ranges.
    pub fn read_block_ranges(
        &self,
        format_version: u8,
        data_stream: &DataStreamReference,
        block_size: u32,
        block_ranges: &mut Vec<ExtBlockRange>,
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
                        block_ranges,
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
                        block_ranges,
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
        }
        Ok(())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::{PosixTime32, PosixTime64Ns};

    fn get_test_data_ext2() -> Vec<u8> {
        return vec![
            0xa4, 0x81, 0xe8, 0x03, 0x5e, 0x2c, 0x00, 0x00, 0x0a, 0xea, 0x78, 0x67, 0x09, 0xea,
            0x78, 0x67, 0x09, 0xea, 0x78, 0x67, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x01, 0x00,
            0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x0c,
            0x00, 0x00, 0x02, 0x0c, 0x00, 0x00, 0x03, 0x0c, 0x00, 0x00, 0x04, 0x0c, 0x00, 0x00,
            0x05, 0x0c, 0x00, 0x00, 0x06, 0x0c, 0x00, 0x00, 0x07, 0x0c, 0x00, 0x00, 0x08, 0x0c,
            0x00, 0x00, 0x09, 0x0c, 0x00, 0x00, 0x0a, 0x0c, 0x00, 0x00, 0x0b, 0x0c, 0x00, 0x00,
            0x0c, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x66, 0x70, 0x90, 0x8e, 0xa2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    fn get_test_data_ext3() -> Vec<u8> {
        return vec![
            0xb4, 0x81, 0xe8, 0x03, 0xdc, 0x48, 0x00, 0x00, 0xe8, 0x73, 0x3d, 0x5f, 0xe8, 0x73,
            0x3d, 0x5f, 0xe8, 0x73, 0x3d, 0x5f, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x01, 0x00,
            0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0c,
            0x00, 0x00, 0x02, 0x0c, 0x00, 0x00, 0x03, 0x0c, 0x00, 0x00, 0x04, 0x0c, 0x00, 0x00,
            0x05, 0x0c, 0x00, 0x00, 0x06, 0x0c, 0x00, 0x00, 0x07, 0x0c, 0x00, 0x00, 0x08, 0x0c,
            0x00, 0x00, 0x09, 0x0c, 0x00, 0x00, 0x0a, 0x0c, 0x00, 0x00, 0x0b, 0x0c, 0x00, 0x00,
            0x0c, 0x0c, 0x00, 0x00, 0xa4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x52, 0x04, 0x82, 0x3b, 0xa2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    fn get_test_data_ext4() -> Vec<u8> {
        return vec![
            0xa4, 0x81, 0xe8, 0x03, 0x5e, 0x2c, 0x00, 0x00, 0x0a, 0xea, 0x78, 0x67, 0x0a, 0xea,
            0x78, 0x67, 0x0a, 0xea, 0x78, 0x67, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x01, 0x00,
            0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0a, 0xf3,
            0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0c, 0x00, 0x00, 0x00, 0x53, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x8a, 0xe5, 0x90, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9c, 0xbd,
            0x00, 0x00, 0x20, 0x00, 0xd5, 0x53, 0x0c, 0xd9, 0x63, 0x38, 0x0c, 0xd9, 0x63, 0x38,
            0x0c, 0xd9, 0x63, 0x38, 0x0a, 0xea, 0x78, 0x67, 0x0c, 0xd9, 0x63, 0x38, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xea, 0x07, 0x06, 0x34, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x65,
            0x6c, 0x69, 0x6e, 0x75, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0x6e, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x6e,
            0x65, 0x64, 0x5f, 0x75, 0x3a, 0x6f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x5f, 0x72, 0x3a,
            0x75, 0x6e, 0x6c, 0x61, 0x62, 0x65, 0x6c, 0x65, 0x64, 0x5f, 0x74, 0x3a, 0x73, 0x30,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    // TODO: add tests for has_inline_data
    // TODO: add tests for read_block_ranges

    #[test]
    fn test_read_data_ext2() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        test_struct.read_data(2, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.data_size, 11358);
        assert_eq!(test_struct.access_timestamp, 1735977482);
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977482)))
        );
        assert_eq!(test_struct.change_timestamp, 1735977481);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977481)))
        );
        assert_eq!(test_struct.modification_timestamp, 1735977481);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977481)))
        );
        assert_eq!(test_struct.deletion_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.number_of_blocks, 26);
        assert_eq!(test_struct.flags, 0);
        assert_eq!(test_struct.data_reference, &test_data[40..100]);
        assert_eq!(test_struct.file_acl_block_number, 162);

        Ok(())
    }

    #[test]
    fn test_read_data_ext3() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext3();
        test_struct.read_data(3, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o100664);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.data_size, 18652);
        assert_eq!(test_struct.access_timestamp, 1597862888);
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862888)))
        );
        assert_eq!(test_struct.change_timestamp, 1597862888);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862888)))
        );
        assert_eq!(test_struct.modification_timestamp, 1597862888);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862888)))
        );
        assert_eq!(test_struct.deletion_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.number_of_blocks, 42);
        assert_eq!(test_struct.flags, 0);
        assert_eq!(test_struct.data_reference, &test_data[40..100]);
        assert_eq!(test_struct.file_acl_block_number, 162);

        Ok(())
    }

    #[test]
    fn test_read_data_ext4() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext4();
        test_struct.read_data(4, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.data_size, 11358);
        assert_eq!(test_struct.access_timestamp, 1735977482);
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                1735977482, 236516931
            )))
        );
        assert_eq!(test_struct.change_timestamp, 1735977482);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                1735977482, 236516931
            )))
        );
        assert_eq!(test_struct.modification_timestamp, 1735977482);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                1735977482, 236516931
            )))
        );
        assert_eq!(test_struct.deletion_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.number_of_blocks, 24);
        assert_eq!(test_struct.flags, 0x00080000);
        assert_eq!(test_struct.data_reference, &test_data[40..100]);
        assert_eq!(test_struct.file_acl_block_number, 0);
        assert_eq!(
            test_struct.creation_time,
            Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                1735977482, 236516931
            )))
        );
        assert_eq!(test_struct.checksum, 0x53d5bd9c);
        assert_eq!(test_struct.attributes.len(), 1);

        Ok(())
    }
}
