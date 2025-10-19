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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_i32_le, bytes_to_u16_le, bytes_to_u32_le};

use super::attribute::ExtAttribute;
use super::attributes_block::ExtAttributesBlock;
use super::block_numbers_tree::ExtBlockNumbersTree;
use super::block_range::ExtBlockRange;
use super::constants::*;
use super::extents_tree::ExtExtentsTree;

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
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported inode data size"
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

        inode.access_timestamp = bytes_to_i32_le!(data, 8);
        if inode.access_timestamp > 0 {
            inode.access_time = DateTime::PosixTime32(PosixTime32::new(inode.access_timestamp));
        }
        inode.change_timestamp = bytes_to_i32_le!(data, 12);
        if inode.change_timestamp > 0 {
            inode.change_time = DateTime::PosixTime32(PosixTime32::new(inode.change_timestamp));
        }
        inode.modification_timestamp = bytes_to_i32_le!(data, 16);
        if inode.modification_timestamp > 0 {
            inode.modification_time =
                DateTime::PosixTime32(PosixTime32::new(inode.modification_timestamp));
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
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported inode data size"
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

        inode.access_timestamp = bytes_to_i32_le!(data, 8);
        if inode.access_timestamp > 0 {
            inode.access_time = DateTime::PosixTime32(PosixTime32::new(inode.access_timestamp));
        }
        inode.change_timestamp = bytes_to_i32_le!(data, 12);
        if inode.change_timestamp > 0 {
            inode.change_time = DateTime::PosixTime32(PosixTime32::new(inode.change_timestamp));
        }
        inode.modification_timestamp = bytes_to_i32_le!(data, 16);
        if inode.modification_timestamp > 0 {
            inode.modification_time =
                DateTime::PosixTime32(PosixTime32::new(inode.modification_timestamp));
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
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported inode data size"
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
            inode.access_timestamp = bytes_to_i32_le!(data, 8);
            if inode.access_timestamp > 0 {
                inode.access_time = DateTime::PosixTime32(PosixTime32::new(inode.access_timestamp));
            }
            inode.change_timestamp = bytes_to_i32_le!(data, 12);
            if inode.change_timestamp > 0 {
                inode.change_time = DateTime::PosixTime32(PosixTime32::new(inode.change_timestamp));
            }
            inode.modification_timestamp = bytes_to_i32_le!(data, 16);
            if inode.modification_timestamp > 0 {
                inode.modification_time =
                    DateTime::PosixTime32(PosixTime32::new(inode.modification_timestamp));
            }
            let timestamp: i32 = bytes_to_i32_le!(data, 20);
            if timestamp > 0 {
                inode.deletion_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
            }
        }
        inode.number_of_links = bytes_to_u16_le!(data, 26);

        let lower_32bit: u32 = bytes_to_u32_le!(data, 28);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 116);
        inode.number_of_blocks = ((upper_16bit as u64) << 32) | (lower_32bit as u64);

        inode.data_reference.copy_from_slice(&data[40..100]);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 124);
        inode.checksum = lower_16bit as u32;

        Ok(())
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "extra_size", data_type = "u16")),
        member(field(name = "checksum_upper", data_type = "u16", format = "hex")),
        member(field(name = "change_time_extra_precision", data_type = "u32")),
        member(field(name = "modification_time_extra_precision", data_type = "u32")),
        member(field(name = "access_time_extra_precision", data_type = "u32")),
        member(field(name = "creation_time", data_type = "PosixTime32")),
        member(field(name = "creation_time_extra_precision", data_type = "u32")),
        member(field(name = "version_upper", data_type = "u32")),
        member(field(name = "unknown3", data_type = "[u8; 4]")),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext4) inode extension.
struct Ext4InodeExtension {}

impl Ext4InodeExtension {
    /// Creates a new inode extension.
    pub fn new() -> Self {
        Self {}
    }

    /// Determines an extra precision timestamp.
    fn get_extra_precision_timestamp(timestamp: i32, mut extra_precision: u32) -> (i64, u32) {
        let mut extra_precision_timestamp: i64 = timestamp as i64;
        if extra_precision > 0 {
            let multiplier: u32 = extra_precision & 0x00000003;

            if multiplier != 0 {
                extra_precision_timestamp += 0x100000000 * (multiplier as i64);
            }
            extra_precision >>= 2;
        }
        (extra_precision_timestamp, extra_precision)
    }

    /// Reads the inode exension from a buffer.
    pub fn read_data(&self, inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        let extra_size: u16 = bytes_to_u16_le!(data, 0);

        if extra_size >= 4 {
            let upper_16bit: u16 = bytes_to_u16_le!(data, 2);
            inode.checksum |= (upper_16bit as u32) << 16;
        }
        if inode.flags & 0x00200000 == 0 {
            if extra_size >= 8 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 4);
                let (timestamp, fraction): (i64, u32) =
                    Self::get_extra_precision_timestamp(inode.change_timestamp, extra_precision);

                if timestamp > 0 {
                    inode.change_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
            if extra_size >= 12 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 8);
                let (timestamp, fraction): (i64, u32) = Self::get_extra_precision_timestamp(
                    inode.modification_timestamp,
                    extra_precision,
                );

                if timestamp > 0 {
                    inode.modification_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
            if extra_size >= 16 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 12);
                let (timestamp, fraction): (i64, u32) =
                    Self::get_extra_precision_timestamp(inode.access_timestamp, extra_precision);

                if timestamp > 0 {
                    inode.access_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
        }
        if extra_size >= 24 {
            let timestamp: i32 = bytes_to_i32_le!(data, 16);
            let extra_precision: u32 = bytes_to_u32_le!(data, 20);
            let (extra_precision_timestamp, fraction): (i64, u32) =
                Self::get_extra_precision_timestamp(timestamp, extra_precision);

            if timestamp > 0 {
                inode.creation_time = Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                    extra_precision_timestamp,
                    fraction,
                )));
            }
        }
        let mut data_offset: usize = extra_size as usize;
        let data_end_offset: usize = data_offset + 4;
        let data_size: usize = data.len();

        if data_end_offset < data_size {
            if data[data_offset..data_end_offset] == EXT_ATTRIBUTES_HEADER_SIGNATURE {
                data_offset = data_end_offset;

                let attributes_block: ExtAttributesBlock = ExtAttributesBlock::new();

                match attributes_block.read_entries(
                    data,
                    data_offset,
                    data_size,
                    &mut inode.attributes,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read extended attributes"
                        );
                        return Err(error);
                    }
                }
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

    /// Access timestamp.
    access_timestamp: i32,

    /// Access date and time.
    pub access_time: DateTime,

    /// Change timestamp.
    change_timestamp: i32,

    /// Change date and time.
    pub change_time: DateTime,

    /// Modification timestamp.
    modification_timestamp: i32,

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
                let inode: Ext4Inode = Ext4Inode::new();
                inode.read_data(self, data)?;
            }
            3 => {
                let inode: Ext3Inode = Ext3Inode::new();
                inode.read_data(self, data)?;
            }
            _ => {
                let inode: Ext2Inode = Ext2Inode::new();
                inode.read_data(self, data)?;
            }
        }
        if data.len() > 128 {
            let inode_extension: Ext4InodeExtension = Ext4InodeExtension::new();
            inode_extension.read_data(self, &data[128..])?;
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
    fn test_get_extra_precision_timestamp() -> Result<(), ErrorTrace> {
        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(1733561817, 0);
        assert_eq!(timestamp, 1733561817);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(1733561817, 49382716);
        assert_eq!(timestamp, 1733561817);
        assert_eq!(fraction, 12345679);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 1);
        assert_eq!(timestamp, 4294967295);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 1);
        assert_eq!(timestamp, 2147483648);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 1);
        assert_eq!(timestamp, 4294967296);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 1);
        assert_eq!(timestamp, 6442450943);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 2);
        assert_eq!(timestamp, 8589934591);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 2);
        assert_eq!(timestamp, 6442450944);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 2);
        assert_eq!(timestamp, 8589934592);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 2);
        assert_eq!(timestamp, 10737418239);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 3);
        assert_eq!(timestamp, 12884901887);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 3);
        assert_eq!(timestamp, 10737418240);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 3);
        assert_eq!(timestamp, 12884901888);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 3);
        assert_eq!(timestamp, 15032385535);
        assert_eq!(fraction, 0);

        Ok(())
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
    fn test_read_data_ext2_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data_ext2();
        let result = test_struct.read_data(2, &test_data[0..127]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_ext3() -> Result<(), ErrorTrace> {
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
    fn test_read_data_ext4() -> Result<(), ErrorTrace> {
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
