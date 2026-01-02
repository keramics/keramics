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
use keramics_datetime::{DateTime, PosixTime32};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_i32_le, bytes_to_u16_le, bytes_to_u32_le};

use crate::ext::constants::*;
use crate::ext::inode::ExtInode;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "file_mode", data_type = "u16"),
        field(name = "owner_identifier_lower", data_type = "u16"),
        field(name = "data_size_lower", data_type = "u32"),
        field(name = "access_time", data_type = "PosixTime32"),
        field(name = "change_time", data_type = "PosixTime32"),
        field(name = "modification_time", data_type = "PosixTime32"),
        field(name = "deletion_time", data_type = "PosixTime32"),
        field(name = "group_identifier_lower", data_type = "u16"),
        field(name = "number_of_links", data_type = "u16"),
        field(name = "number_of_blocks_lower", data_type = "u32"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "version_lower", data_type = "u32"),
        field(name = "data_reference", data_type = "[u8; 60]"),
        field(name = "nfs_generation_number", data_type = "u32"),
        field(name = "file_acl_block_number_lower", data_type = "u32"),
        field(name = "data_size_upper", data_type = "u32"),
        field(name = "fragment_block_address", data_type = "u32"),
        field(name = "number_of_blocks_upper", data_type = "u16"),
        field(name = "file_acl_block_number_upper", data_type = "u16"),
        field(name = "owner_identifier_upper", data_type = "u16"),
        field(name = "group_identifier_upper", data_type = "u16"),
        field(name = "checksum_lower", data_type = "u16", format = "hex"),
        field(name = "unknown2", data_type = "[u8; 2]"),
    ),
    methods("debug_read_data")
)]
/// Extended File System (ext4) inode.
pub struct Ext4Inode {}

impl Ext4Inode {
    /// Reads the inode from a buffer.
    pub fn read_data(inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
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

        if inode.flags & EXT_INODE_FLAG_IS_EXTENDED_ATTRIBUTE_INODE == 0 {
            inode.access_timestamp = bytes_to_i32_le!(data, 8);
            if inode.access_timestamp > 0 {
                inode.access_time = Some(DateTime::PosixTime32(PosixTime32::new(
                    inode.access_timestamp,
                )));
            }
            inode.change_timestamp = bytes_to_i32_le!(data, 12);
            if inode.change_timestamp > 0 {
                inode.change_time = Some(DateTime::PosixTime32(PosixTime32::new(
                    inode.change_timestamp,
                )));
            }
            inode.modification_timestamp = bytes_to_i32_le!(data, 16);
            if inode.modification_timestamp > 0 {
                inode.modification_time = Some(DateTime::PosixTime32(PosixTime32::new(
                    inode.modification_timestamp,
                )));
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

        let lower_32bit: u32 = bytes_to_u32_le!(data, 104);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 118);
        inode.file_acl_block_number = ((upper_16bit as u64) << 32) | (lower_32bit as u64);

        let lower_16bit: u16 = bytes_to_u16_le!(data, 124);
        inode.checksum = lower_16bit as u32;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keramics_datetime::PosixTime32;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xb4, 0x81, 0xe8, 0x03, 0xdc, 0x48, 0x00, 0x00, 0xf0, 0x73, 0x3d, 0x5f, 0xf0, 0x73,
            0x3d, 0x5f, 0xf0, 0x73, 0x3d, 0x5f, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x01, 0x00,
            0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0xf3,
            0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x13, 0x00, 0x00, 0x00, 0xc3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xcd, 0xea, 0x3a, 0xf6, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb5, 0xc8,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data();
        Ext4Inode::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o100664);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.data_size, 18652);
        assert_eq!(test_struct.access_timestamp, 1597862896);
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862896)))
        );
        assert_eq!(test_struct.change_timestamp, 1597862896);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862896)))
        );
        assert_eq!(test_struct.modification_timestamp, 1597862896);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1597862896)))
        );
        assert_eq!(test_struct.deletion_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.number_of_blocks, 40);
        assert_eq!(test_struct.flags, 0x00080000);
        assert_eq!(test_struct.data_reference, &test_data[40..100]);
        assert_eq!(test_struct.file_acl_block_number, 48);
        assert_eq!(test_struct.checksum, 0x0000c8b5);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data();
        let result = Ext4Inode::read_data(&mut test_struct, &test_data[0..127]);
        assert!(result.is_err());
    }
}
