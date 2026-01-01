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
        field(name = "data_size", data_type = "u32"),
        field(name = "access_time", data_type = "PosixTime32"),
        field(name = "change_time", data_type = "PosixTime32"),
        field(name = "modification_time", data_type = "PosixTime32"),
        field(name = "deletion_time", data_type = "PosixTime32"),
        field(name = "group_identifier_lower", data_type = "u16"),
        field(name = "number_of_links", data_type = "u16"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "data_reference", data_type = "[u8; 60]"),
        field(name = "nfs_generation_number", data_type = "u32"),
        field(name = "file_acl_block_number", data_type = "u32"),
        field(name = "directory_acl", data_type = "u32"),
        field(name = "fragment_block_address", data_type = "u32"),
        field(name = "fragment_block_index", data_type = "u8"),
        field(name = "fragment_size", data_type = "u8"),
        field(name = "padding1", data_type = "[u8; 2]"),
        field(name = "owner_identifier_upper", data_type = "u16"),
        field(name = "group_identifier_upper", data_type = "u16"),
        field(name = "unknown2", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// Extended File System (ext2) inode.
pub struct Ext2Inode {}

impl Ext2Inode {
    /// Reads the inode from a buffer.
    pub fn read_data(inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
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
        }
        let timestamp: i32 = bytes_to_i32_le!(data, 20);
        if timestamp > 0 {
            inode.deletion_time = DateTime::PosixTime32(PosixTime32::new(timestamp));
        }
        inode.number_of_links = bytes_to_u16_le!(data, 26);
        inode.number_of_blocks = bytes_to_u32_le!(data, 28) as u64;

        inode.data_reference.copy_from_slice(&data[40..100]);

        inode.file_acl_block_number = bytes_to_u32_le!(data, 104) as u64;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::PosixTime32;

    fn get_test_data() -> Vec<u8> {
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

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data();
        Ext2Inode::read_data(&mut test_struct, &test_data)?;

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
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExtInode::new();

        let test_data: Vec<u8> = get_test_data();
        let result = Ext2Inode::read_data(&mut test_struct, &test_data[0..127]);
        assert!(result.is_err());
    }
}
