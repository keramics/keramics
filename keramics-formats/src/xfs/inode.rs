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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_types::bytes_to_u32_le;

use super::constants::*;
use super::extent_list::XfsExtentList;
use super::extent_tree::XfsExtentTree;
use super::inode_v1::XfsInodeV1;
use super::inode_v2::XfsInodeV2;
use super::inode_v3::XfsInodeV3;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) inode.
pub struct XfsInode {
    /// File mode.
    pub file_mode: u16,

    /// Fork type.
    pub fork_type: u8,

    /// Number of links.
    pub number_of_links: u32,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// Data size.
    pub data_size: u64,

    /// Number of extents.
    pub number_of_extents: u64,

    /// Access date and time.
    pub access_time: DateTime,

    /// Change date and time.
    pub change_time: DateTime,

    /// Modification date and time.
    pub modification_time: DateTime,

    /// Creation date and time.
    pub creation_time: Option<DateTime>,

    /// Number of attributes extents.
    pub number_of_attributes_extents: u16,

    /// Attributes fork offset.
    pub attributes_fork_offset: u8,

    /// Attributes fork type.
    pub attributes_fork_type: u8,

    /// Data fork.
    pub data_fork: Vec<u8>,

    /// Attributes fork.
    pub attributes_fork: Vec<u8>,

    /// Extents.
    pub extents: Vec<XfsPackedExtent>,

    /// Device identifier.
    pub device_identifier: Option<u32>,
}

impl XfsInode {
    /// Creates a new inode.
    pub fn new() -> Self {
        Self {
            file_mode: 0,
            fork_type: 0,
            number_of_links: 0,
            owner_identifier: 0,
            group_identifier: 0,
            data_size: 0,
            number_of_extents: 0,
            access_time: DateTime::NotSet,
            change_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            creation_time: None,
            number_of_attributes_extents: 0,
            attributes_fork_offset: 0,
            attributes_fork_type: 0,
            data_fork: Vec::new(),
            attributes_fork: Vec::new(),
            extents: Vec::new(),
            device_identifier: None,
        }
    }

    /// Reads the inode for debugging.
    #[cfg(feature = "debug-trace")]
    pub fn debug_read_data(data: &[u8]) -> String {
        let format_version: u8 = if data.len() < 5 { 0 } else { data[4] };

        match format_version {
            1 => XfsInodeV1::debug_read_data(data),
            2 => XfsInodeV2::debug_read_data(data),
            3 => XfsInodeV3::debug_read_data(data),
            _ => format!("Unsupported format version: {}", format_version),
        }
    }

    /// Reads the inode from a buffer.
    pub fn read_data(
        &mut self,
        format_version: u16,
        has_bigtime: bool,
        has_64bit_number_of_extents: bool,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 5 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != XFS_INODE_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let inode_format_version: u8 = data[4];

        match inode_format_version {
            1 => {
                XfsInodeV1::read_data(self, data)?;
            }
            2 => {
                if format_version < 4 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported inode version 2 for superblock version: {}",
                        format_version
                    )));
                }
                XfsInodeV2::read_data(self, data)?;
            }
            3 => {
                if format_version < 5 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported inode version 3 for superblock version: {}",
                        format_version
                    )));
                }
                XfsInodeV3::read_data(self, has_bigtime, has_64bit_number_of_extents, data)?;
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format version: {}",
                    data[4]
                )));
            }
        }
        let data_fork_offset: usize = if inode_format_version < 3 { 100 } else { 176 };
        let mut data_fork_size: usize = data_size - data_fork_offset;

        if self.attributes_fork_offset > 0 {
            if (self.attributes_fork_offset as usize) > data_fork_size / 8 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid attributes fork offset value out of bounds"
                ));
            }
            data_fork_size = (self.attributes_fork_offset as usize) * 8;
            let attributes_fork_offset: usize = data_fork_offset + data_fork_size;

            self.attributes_fork = data[attributes_fork_offset..data_size].to_vec();
        }
        let data_fork_end_offset: usize = data_fork_offset + data_fork_size;
        self.data_fork = data[data_fork_offset..data_fork_end_offset].to_vec();

        Ok(())
    }

    /// Reads the inode from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        format_version: u16,
        has_bigtime: bool,
        has_64bit_number_of_extents: bool,
        data_stream: &DataStreamReference,
        inode_size: u16,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; inode_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data_and_structure!(
            "XfsInode",
            offset,
            &data,
            inode_size,
            Self::debug_read_data(&data)
        );
        match self.read_data(
            format_version,
            has_bigtime,
            has_64bit_number_of_extents,
            &data,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read inode at offset: {} (0x{:08x})",
                        offset, offset
                    ),
                );
                return Err(error);
            }
        }
        if self.file_mode & 0xf000 == XFS_FILE_MODE_TYPE_CHARACTER_DEVICE
            || self.file_mode & 0xf000 == XFS_FILE_MODE_TYPE_BLOCK_DEVICE
        {
            if self.fork_type == XFS_FORK_TYPE_DEVICE {
                let data_fork_size: usize = self.data_fork.len();

                if data_fork_size < 4 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid device identifier data size: {} value out of bounds",
                        data_fork_size
                    )));
                }
                self.device_identifier = Some(bytes_to_u32_le!(&self.data_fork, 0));
            }
        }
        Ok(())
    }

    /// Reads the extents.
    pub fn read_extents(
        &mut self,
        format_version: u16,
        allocation_group_size: u32,
        number_of_relative_block_number_bits: u32,
        data_stream: &DataStreamReference,
        block_size: u32,
    ) -> Result<(), ErrorTrace> {
        match self.fork_type {
            XFS_FORK_TYPE_EXTENTS => {
                let extent_list: XfsExtentList = XfsExtentList::new();

                match extent_list.read_data(
                    self.number_of_extents,
                    &self.data_fork,
                    &mut self.extents,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read extent list");
                        return Err(error);
                    }
                }
            }
            XFS_FORK_TYPE_BTREE => {
                let extent_tree: XfsExtentTree = XfsExtentTree::new(
                    format_version,
                    allocation_group_size,
                    number_of_relative_block_number_bits,
                    block_size,
                );
                match extent_tree.read_extents(data_stream, &self.data_fork, &mut self.extents) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read extent tree");
                        return Err(error);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::PosixTime64Ns;

    fn get_test_data_v1() -> Vec<u8> {
        vec![
            0x49, 0x4e, 0x41, 0xed, 0x01, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x6a, 0x11, 0xcc, 0xee, 0x0e, 0x3e, 0x12, 0x88, 0x6a, 0x11,
            0xcc, 0xee, 0x0e, 0x3e, 0x12, 0x88, 0x6a, 0x11, 0xcc, 0xee, 0x0e, 0x3e, 0x12, 0x88,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x4b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn get_test_data_v2() -> Vec<u8> {
        vec![
            0x49, 0x4e, 0x41, 0xed, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5f, 0xb2,
            0xd0, 0x14, 0x26, 0xde, 0x5a, 0xc6, 0x5f, 0xb2, 0xd0, 0x14, 0x26, 0xde, 0x5a, 0xc6,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x6b, 0xa0, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn get_test_data_v3() -> Vec<u8> {
        vec![
            0x49, 0x4e, 0x41, 0xed, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x6a, 0x90, 0x3c, 0xcf, 0x26, 0xe5, 0x32, 0xf8, 0x6a, 0x90,
            0x3c, 0xd0, 0x0b, 0x23, 0x40, 0x34, 0x6a, 0x90, 0x3c, 0xd0, 0x0b, 0x23, 0x40, 0x34,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x25, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
            0xff, 0xff, 0xcd, 0x89, 0x2c, 0x69, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x6a, 0x90, 0x3c, 0xcf, 0x26, 0xe5, 0x32, 0xf8, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x1a, 0x06, 0xf8, 0x61, 0x55, 0x10, 0x42, 0x99,
            0x86, 0xaa, 0x62, 0x9a, 0x2d, 0x10, 0x94, 0xe4, 0x09, 0x00, 0x00, 0x00, 0x3f, 0x00,
            0x09, 0x00, 0x60, 0x65, 0x6d, 0x70, 0x74, 0x79, 0x66, 0x69, 0x6c, 0x65, 0x01, 0x00,
            0x00, 0x3f, 0x03, 0x08, 0x00, 0x78, 0x74, 0x65, 0x73, 0x74, 0x64, 0x69, 0x72, 0x31,
            0x02, 0x00, 0x00, 0x3f, 0x04, 0x0e, 0x00, 0x90, 0x66, 0x69, 0x6c, 0x65, 0x5f, 0x68,
            0x61, 0x72, 0x64, 0x6c, 0x69, 0x6e, 0x6b, 0x31, 0x01, 0x00, 0x00, 0x3f, 0x05, 0x12,
            0x00, 0xb0, 0x66, 0x69, 0x6c, 0x65, 0x5f, 0x73, 0x79, 0x6d, 0x62, 0x6f, 0x6c, 0x69,
            0x63, 0x6c, 0x69, 0x6e, 0x6b, 0x31, 0x07, 0x00, 0x00, 0x3f, 0x07, 0x17, 0x00, 0xd0,
            0x64, 0x69, 0x72, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x79, 0x5f, 0x73, 0x79, 0x6d, 0x62,
            0x6f, 0x6c, 0x69, 0x63, 0x6c, 0x69, 0x6e, 0x6b, 0x31, 0x07, 0x00, 0x00, 0x3f, 0x08,
            0x0e, 0x00, 0xf8, 0x6e, 0x66, 0x63, 0x5f, 0x74, 0xc3, 0xa9, 0x73, 0x74, 0x66, 0x69,
            0x6c, 0xc3, 0xa8, 0x01, 0x00, 0x00, 0x3f, 0x09, 0x10, 0x01, 0x18, 0x6e, 0x66, 0x64,
            0x5f, 0x74, 0x65, 0xcc, 0x81, 0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0xcc, 0x80, 0x01,
            0x00, 0x00, 0x3f, 0x0a, 0x06, 0x01, 0x38, 0x6e, 0x66, 0x64, 0x5f, 0xc2, 0xbe, 0x01,
            0x00, 0x00, 0x3f, 0x0b, 0x0a, 0x01, 0x50, 0x6e, 0x66, 0x6b, 0x64, 0x5f, 0x33, 0xe2,
            0x81, 0x84, 0x34, 0x01, 0x00, 0x00, 0x3f, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x01, 0x00,
            0x0c, 0x06, 0x02, 0x78, 0x66, 0x73, 0x3a, 0x61, 0x75, 0x74, 0x6f, 0x66, 0x73, 0x63,
            0x6b, 0x72, 0x65, 0x70, 0x61, 0x69, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data_v1() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_v1();

        let mut test_struct = XfsInode::new();
        test_struct.read_data(5, false, true, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);
        assert_eq!(test_struct.fork_type, 1);
        assert_eq!(test_struct.number_of_links, 2);
        assert_eq!(test_struct.owner_identifier, 0);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(
            test_struct.access_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1779551470,
                fraction: 238949000
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1779551470,
                fraction: 238949000
            })
        );
        assert_eq!(
            test_struct.change_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1779551470,
                fraction: 238949000
            })
        );
        assert_eq!(test_struct.data_size, 6);
        assert_eq!(test_struct.number_of_extents, 0);
        assert_eq!(test_struct.number_of_attributes_extents, 0);
        assert_eq!(test_struct.attributes_fork_offset, 0);
        assert_eq!(test_struct.attributes_fork_type, 2);
        assert_eq!(test_struct.device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_read_data_v2() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_v2();

        let mut test_struct = XfsInode::new();
        test_struct.read_data(5, false, true, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);
        assert_eq!(test_struct.fork_type, 2);
        assert_eq!(test_struct.number_of_links, 3);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(test_struct.access_time, DateTime::NotSet);
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1605554196,
                fraction: 652106438
            })
        );
        assert_eq!(
            test_struct.change_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1605554196,
                fraction: 652106438
            })
        );
        assert_eq!(test_struct.data_size, 4096);
        assert_eq!(test_struct.number_of_extents, 1);
        assert_eq!(test_struct.number_of_attributes_extents, 0);
        assert_eq!(test_struct.attributes_fork_offset, 0);
        assert_eq!(test_struct.attributes_fork_type, 2);
        assert_eq!(test_struct.device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_read_data_v3() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_v3();

        let mut test_struct = XfsInode::new();
        test_struct.read_data(5, false, true, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);
        assert_eq!(test_struct.fork_type, 1);
        assert_eq!(test_struct.number_of_links, 3);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(
            test_struct.access_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837647,
                fraction: 652555000
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 186859572
            })
        );
        assert_eq!(
            test_struct.change_time,
            DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 186859572
            })
        );
        assert_eq!(
            test_struct.creation_time,
            Some(DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837647,
                fraction: 652555000
            }))
        );
        assert_eq!(test_struct.data_size, 196);
        assert_eq!(test_struct.number_of_extents, 0);
        assert_eq!(test_struct.number_of_attributes_extents, 0);
        assert_eq!(test_struct.attributes_fork_offset, 37);
        assert_eq!(test_struct.attributes_fork_type, 1);
        assert_eq!(test_struct.device_identifier, None);

        Ok(())
    }
}
