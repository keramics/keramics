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
use std::io;

use core::DataStreamReference;
use datetime::{DateTime, PosixTime32, PosixTime64Ns};
use layout_map::LayoutMap;
use types::{bytes_to_i32_le, bytes_to_u16_le, bytes_to_u32_le};

use super::attribute::ExtAttribute;
use super::attributes_block::ExtAttributesBlock;
use super::block_numbers_tree::ExtBlockNumbersTree;
use super::block_range::ExtBlockRange;
use super::constants::*;
use super::extents_tree::ExtExtentsTree;

/// Determines the date and time value of a timestamp and extra precision.
fn get_datetime_value(timestamp: i32, mut extra_precision: u32) -> DateTime {
    let mut extra_precision_timestamp: i64 = 0;
    if extra_precision > 0 {
        let multiplier: u32 = extra_precision & 0x00000003;

        if multiplier != 0 {
            extra_precision_timestamp = 0x100000000 * (multiplier as i64);
        }
        extra_precision_timestamp += timestamp as i64;
        extra_precision >>= 2;
    }
    if extra_precision_timestamp != 0 {
        DateTime::PosixTime64Ns(PosixTime64Ns::new(
            extra_precision_timestamp,
            extra_precision,
        ))
    } else if timestamp != 0 {
        DateTime::PosixTime32(PosixTime32::new(timestamp))
    } else {
        DateTime::NotSet
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "file_mode", data_type = "u16")),
        member(field(name = "owner_identifier_lower", data_type = "u16")),
        member(field(name = "data_size", data_type = "u32")),
        member(field(name = "access_time", data_type = "PosixTime32")),
        member(field(name = "change_time", data_type = "PosixTime32")),
        member(field(name = "modification_time", data_type = "PosixTime32")),
        member(field(name = "deletion_time", data_type = "PosixTime32")),
        member(field(name = "group_identifier_lower", data_type = "u16")),
        member(field(name = "number_of_links", data_type = "u16")),
        member(field(name = "number_of_blocks", data_type = "u32")),
        member(field(name = "flags", data_type = "u32", format = "hex")),
        member(field(name = "unknown1", data_type = "[u8; 4]")),
        member(field(name = "data_reference", data_type = "[u8; 60]")),
        member(field(name = "nfs_generation_number", data_type = "u32")),
        member(field(name = "file_acl_block_number", data_type = "u32")),
        member(field(name = "directory_acl", data_type = "u32")),
        member(field(name = "fragment_block_address", data_type = "u32")),
        member(field(name = "fragment_block_index", data_type = "u8")),
        member(field(name = "fragment_size", data_type = "u8")),
        member(field(name = "padding1", data_type = "[u8; 2]")),
        member(field(name = "owner_identifier_upper", data_type = "u16")),
        member(field(name = "group_identifier_upper", data_type = "u16")),
        member(field(name = "unknown2", data_type = "[u8; 4]")),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext2) inode.
struct Ext2Inode {}

impl Ext2Inode {
    /// Creates a new inode.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the inode from a buffer.
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> io::Result<()> {
        if data.len() < 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        inode.file_mode = bytes_to_u16_le!(data, 0);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 2);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 120);
        inode.owner_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 24);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 122);
        inode.group_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        inode.data_size = bytes_to_u32_le!(data, 4) as u64;

        let timestamp: i32 = bytes_to_i32_le!(data, 8);
        if timestamp > 0 {
            inode.access_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 12);
        if timestamp > 0 {
            inode.change_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 16);
        if timestamp > 0 {
            inode.modification_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 20);
        if timestamp > 0 {
            inode.deletion_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        inode.number_of_links = bytes_to_u16_le!(data, 26);
        inode.number_of_blocks = bytes_to_u32_le!(data, 28) as u64;
        inode.flags = bytes_to_u32_le!(data, 32);

        inode.data_reference.copy_from_slice(&data[40..100]);

        Ok(())
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "file_mode", data_type = "u16")),
        member(field(name = "owner_identifier_lower", data_type = "u16")),
        member(field(name = "data_size", data_type = "u32")),
        member(field(name = "access_time", data_type = "PosixTime32")),
        member(field(name = "change_time", data_type = "PosixTime32")),
        member(field(name = "modification_time", data_type = "PosixTime32")),
        member(field(name = "deletion_time", data_type = "PosixTime32")),
        member(field(name = "group_identifier_lower", data_type = "u16")),
        member(field(name = "number_of_links", data_type = "u16")),
        member(field(name = "number_of_blocks", data_type = "u32")),
        member(field(name = "flags", data_type = "u32", format = "hex")),
        member(field(name = "unknown1", data_type = "[u8; 4]")),
        member(field(name = "data_reference", data_type = "[u8; 60]")),
        member(field(name = "nfs_generation_number", data_type = "u32")),
        member(field(name = "file_acl_block_number", data_type = "u32")),
        member(field(name = "directory_acl", data_type = "u32")),
        member(field(name = "fragment_block_address", data_type = "u32")),
        member(field(name = "fragment_block_index", data_type = "u8")),
        member(field(name = "fragment_size", data_type = "u8")),
        member(field(name = "padding1", data_type = "[u8; 2]")),
        member(field(name = "owner_identifier_upper", data_type = "u16")),
        member(field(name = "group_identifier_upper", data_type = "u16")),
        member(field(name = "unknown2", data_type = "[u8; 4]")),
        member(group(
            size_condition = "> 128",
            field(name = "extra_size", data_type = "u16"),
            field(name = "padding2", data_type = "[u8; 2]"),
        )),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext3) inode.
struct Ext3Inode {}

impl Ext3Inode {
    /// Creates a new inode.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the inode from a buffer.
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> io::Result<()> {
        if data.len() < 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        inode.file_mode = bytes_to_u16_le!(data, 0);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 2);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 120);
        inode.owner_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 24);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 122);
        inode.group_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        inode.data_size = bytes_to_u32_le!(data, 4) as u64;

        let timestamp: i32 = bytes_to_i32_le!(data, 8);
        if timestamp > 0 {
            inode.access_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 12);
        if timestamp > 0 {
            inode.change_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 16);
        if timestamp > 0 {
            inode.modification_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 20);
        if timestamp > 0 {
            inode.deletion_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        inode.number_of_links = bytes_to_u16_le!(data, 26);
        inode.number_of_blocks = bytes_to_u32_le!(data, 28) as u64;
        inode.flags = bytes_to_u32_le!(data, 32);

        inode.data_reference.copy_from_slice(&data[40..100]);

        Ok(())
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "file_mode", data_type = "u16")),
        member(field(name = "owner_identifier_lower", data_type = "u16")),
        member(field(name = "data_size_lower", data_type = "u32")),
        member(field(name = "access_time", data_type = "PosixTime32")),
        member(field(name = "change_time", data_type = "PosixTime32")),
        member(field(name = "modification_time", data_type = "PosixTime32")),
        member(field(name = "deletion_time", data_type = "PosixTime32")),
        member(field(name = "group_identifier_lower", data_type = "u16")),
        member(field(name = "number_of_links", data_type = "u16")),
        member(field(name = "number_of_blocks_lower", data_type = "u32")),
        member(field(name = "flags", data_type = "u32", format = "hex")),
        member(field(name = "version_lower", data_type = "u32")),
        member(field(name = "data_reference", data_type = "[u8; 60]")),
        member(field(name = "nfs_generation_number", data_type = "u32")),
        member(field(name = "file_acl_block_number_lower", data_type = "u32")),
        member(field(name = "data_size_upper", data_type = "u32")),
        member(field(name = "fragment_block_address", data_type = "u32")),
        member(field(name = "number_of_blocks_upper", data_type = "u16")),
        member(field(name = "file_acl_block_number_upper", data_type = "u16")),
        member(field(name = "owner_identifier_upper", data_type = "u16")),
        member(field(name = "group_identifier_upper", data_type = "u16")),
        member(field(name = "checksum_lower", data_type = "u16", format = "hex")),
        member(field(name = "unknown2", data_type = "[u8; 2]")),
        member(group(
            size_condition = "> 128",
            field(name = "extra_size", data_type = "u16"),
            field(name = "checksum_upper", data_type = "u16", format = "hex"),
            field(name = "change_time_extra_precision", data_type = "u32"),
            field(name = "modification_time_extra_precision", data_type = "u32"),
            field(name = "access_time_extra_precision", data_type = "u32"),
            field(name = "creation_time", data_type = "PosixTime32"),
            field(name = "creation_time_extra_precision", data_type = "u32"),
            field(name = "version_upper", data_type = "u32"),
            field(name = "unknown3", data_type = "[u8; 4]"),
        )),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext4) inode.
struct Ext4Inode {}

impl Ext4Inode {
    /// Creates a new inode.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the inode from a buffer.
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> io::Result<()> {
        let data_size: usize = data.len();

        let extra_size: u16 = if data_size < 132 {
            0
        } else {
            bytes_to_u16_le!(data, 128)
        };
        if data_size < 128 + (extra_size as usize) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        inode.file_mode = bytes_to_u16_le!(data, 0);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 2);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 120);
        inode.owner_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 24);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 122);
        inode.group_identifier = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        let lower_32bit: u32 = bytes_to_u32_le!(data, 4);
        let upper_32bit: u32 = bytes_to_u32_le!(data, 108);
        inode.data_size = ((upper_32bit as u64) << 32) | (lower_32bit as u64);

        inode.flags = bytes_to_u32_le!(data, 32);

        if inode.flags & 0x00200000 == 0 {
            let timestamp: i32 = bytes_to_i32_le!(data, 8);
            let extra_precision: u32 = if data_size < 144 {
                0
            } else {
                bytes_to_u32_le!(data, 140)
            };
            inode.access_time = get_datetime_value(timestamp, extra_precision);

            let timestamp: i32 = bytes_to_i32_le!(data, 12);
            let extra_precision: u32 = if data_size < 136 {
                0
            } else {
                bytes_to_u32_le!(data, 132)
            };
            inode.change_time = get_datetime_value(timestamp, extra_precision);

            let timestamp: i32 = bytes_to_i32_le!(data, 16);
            let extra_precision: u32 = if data_size < 140 {
                0
            } else {
                bytes_to_u32_le!(data, 136)
            };
            inode.modification_time = get_datetime_value(timestamp, extra_precision);

            let timestamp: i32 = bytes_to_i32_le!(data, 20);
            inode.deletion_time = if timestamp != 0 {
                DateTime::PosixTime32(PosixTime32::new(timestamp))
            } else {
                DateTime::NotSet
            };
        }
        inode.number_of_links = bytes_to_u16_le!(data, 26);

        let lower_32bit: u32 = bytes_to_u32_le!(data, 28);
        inode.number_of_blocks = lower_32bit as u64;
        if data_size >= 120 {
            let upper_16bit: u16 = bytes_to_u16_le!(data, 116);
            inode.number_of_blocks |= (upper_16bit as u64) << 32;
        }
        inode.data_reference.copy_from_slice(&data[40..100]);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 124);
        inode.checksum = lower_16bit as u32;
        if data_size >= 132 {
            let upper_16bit: u16 = bytes_to_u16_le!(data, 130);
            inode.checksum |= (upper_16bit as u32) << 16;
        }
        if data_size >= 152 {
            let timestamp: i32 = bytes_to_i32_le!(data, 144);
            let extra_precision: u32 = bytes_to_u32_le!(data, 148);
            inode.creation_time = Some(get_datetime_value(timestamp, extra_precision));
        }
        let mut data_offset: usize = 128 + (extra_size as usize);
        let data_end_offset: usize = data_offset + 4;
        if data_end_offset < data_size {
            if data[data_offset..data_end_offset] == EXT_ATTRIBUTES_HEADER_SIGNATURE {
                data_offset = data_end_offset;

                let attributes_block: ExtAttributesBlock = ExtAttributesBlock::new();

                attributes_block.read_entries(
                    data,
                    data_offset,
                    data_size,
                    &mut inode.attributes,
                )?;
            }
        }
        Ok(())
    }
}

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

    /// Access date and time.
    pub access_time: DateTime,

    /// Change date and time.
    pub change_time: DateTime,

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
            access_time: DateTime::NotSet,
            change_time: DateTime::NotSet,
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
        match format_version {
            4 => Ext4Inode::debug_read_data(data),
            3 => Ext3Inode::debug_read_data(data),
            _ => Ext2Inode::debug_read_data(data),
        }
    }

    /// Reads the inode from a buffer.
    pub fn read_data(&mut self, format_version: u8, data: &[u8]) -> io::Result<()> {
        match format_version {
            4 => {
                let inode: Ext4Inode = Ext4Inode::new();
                inode.read_data(self, data)
            }
            3 => {
                let inode: Ext3Inode = Ext3Inode::new();
                inode.read_data(self, data)
            }
            _ => {
                let inode: Ext2Inode = Ext2Inode::new();
                inode.read_data(self, data)
            }
        }
    }

    /// Reads the data reference.
    pub fn read_data_reference(
        &mut self,
        format_version: u8,
        data_stream: &DataStreamReference,
        block_size: u32,
    ) -> io::Result<()> {
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
                    extents_tree.read_data_reference(
                        &self.data_reference,
                        data_stream,
                        &mut self.block_ranges,
                    )?;
                } else {
                    let mut block_numbers_tree: ExtBlockNumbersTree =
                        ExtBlockNumbersTree::new(block_size, number_of_blocks);
                    block_numbers_tree.read_data_reference(
                        &self.data_reference,
                        data_stream,
                        &mut self.block_ranges,
                    )?;
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
    fn test_get_datetime_value() -> io::Result<()> {
        let datetime: DateTime = get_datetime_value(-1, 0);
        assert_eq!(
            datetime,
            DateTime::PosixTime32(PosixTime32 { timestamp: -1 })
        );

        let datetime: DateTime = get_datetime_value(-2147483648, 0);
        assert_eq!(
            datetime,
            DateTime::PosixTime32(PosixTime32 {
                timestamp: -2147483648
            })
        );

        let datetime: DateTime = get_datetime_value(0, 0);
        assert_eq!(datetime, DateTime::NotSet);

        let datetime: DateTime = get_datetime_value(1, 0);
        assert_eq!(
            datetime,
            DateTime::PosixTime32(PosixTime32 { timestamp: 1 })
        );

        let datetime: DateTime = get_datetime_value(2147483647, 0);
        assert_eq!(
            datetime,
            DateTime::PosixTime32(PosixTime32 {
                timestamp: 2147483647
            })
        );

        let datetime: DateTime = get_datetime_value(-1, 1);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 4294967295,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(-2147483648, 1);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 2147483648,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(0, 1);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 4294967296,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(2147483647, 1);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 6442450943,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(-1, 2);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 8589934591,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(-2147483648, 2);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 6442450944,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(0, 2);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 8589934592,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(2147483647, 2);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 10737418239,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(-1, 3);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 12884901887,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(-2147483648, 3);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 10737418240,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(0, 3);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 12884901888,
                fraction: 0
            })
        );

        let datetime: DateTime = get_datetime_value(2147483647, 3);
        assert_eq!(
            datetime,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 15032385535,
                fraction: 0
            })
        );

        Ok(())
    }

    #[test]
    fn test_read_data_ext2() -> io::Result<()> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        test_struct.read_data(2, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    #[test]
    fn test_read_data_ext2_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        let result = test_struct.read_data(2, &test_data[0..127]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_ext3() -> io::Result<()> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext3();
        test_struct.read_data(3, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    #[test]
    fn test_read_data_ext3_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext3();
        let result = test_struct.read_data(3, &test_data[0..127]);
        assert!(result.is_err());
    }

    // TODO: add test for ext3 inode > 128 bytes

    #[test]
    fn test_read_data_ext4() -> io::Result<()> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext4();
        test_struct.read_data(4, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);

        Ok(())
    }

    #[test]
    fn test_read_data_ext4_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext4();
        let result = test_struct.read_data(4, &test_data[0..127]);
        assert!(result.is_err());
    }

    // TODO: add test for ext4 inode > 128 bytes
}
