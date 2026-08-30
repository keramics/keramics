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
use keramics_datetime::{DateTime, PosixTime32, PosixTime64Ns};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_i32_be, bytes_to_u16_be, bytes_to_u32_be, bytes_to_u64_be};

use super::constants::*;
use super::inode::XfsInode;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<2>"),
        field(name = "file_mode", data_type = "u16"),
        field(name = "format_version", data_type = "u8"),
        field(name = "fork_type", data_type = "u8"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "owner_identifier", data_type = "u32"),
        field(name = "group_identifier", data_type = "u32"),
        field(name = "number_of_links", data_type = "u32"),
        field(name = "project_identifier", data_type = "u16"),
        field(name = "unknown2", data_type = "[u8; 8]"),
        field(name = "flush_counter", data_type = "u16"),
        field(name = "access_time", data_type = "PosixTime32"),
        field(name = "access_time_nanoseconds", data_type = "u32"),
        field(name = "modification_time", data_type = "PosixTime32"),
        field(name = "modification_time_nanoseconds", data_type = "u32"),
        field(name = "change_time", data_type = "PosixTime32"),
        field(name = "change_time_nanoseconds", data_type = "u32"),
        field(name = "data_size", data_type = "u64"),
        field(name = "number_of_blocks", data_type = "u64"),
        field(name = "extent_size", data_type = "u32"),
        field(name = "number_of_data_extents", data_type = "u32"),
        field(name = "number_of_attributes_extents", data_type = "u16"),
        field(name = "attributes_fork_offset", data_type = "u8"),
        field(name = "attributes_fork_type", data_type = "u8"),
        field(name = "unknown3", data_type = "[u8; 4]"),
        field(name = "unknown4", data_type = "[u8; 2]"),
        field(name = "inode_flags", data_type = "u16", format = "hex"),
        field(name = "generation_number", data_type = "u32"),
        field(name = "unknown5", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) inode version 2.
pub struct XfsInodeV2 {}

impl XfsInodeV2 {
    /// Reads the inode from a buffer.
    pub fn read_data(inode: &mut XfsInode, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 100 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != XFS_INODE_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        if data[4] != 2 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported format version"
            ));
        }
        inode.file_mode = bytes_to_u16_be!(data, 2);
        inode.fork_type = data[5];
        inode.owner_identifier = bytes_to_u32_be!(data, 8);
        inode.group_identifier = bytes_to_u32_be!(data, 12);
        inode.number_of_links = bytes_to_u32_be!(data, 16);

        let timestamp: i32 = bytes_to_i32_be!(data, 32);
        let nanoseconds: u32 = bytes_to_u32_be!(data, 36);

        inode.access_time = if timestamp == 0 && nanoseconds == 0 {
            DateTime::NotSet
        } else if nanoseconds == 0 {
            DateTime::PosixTime32(PosixTime32::new(timestamp))
        } else {
            DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp as i64, nanoseconds))
        };
        let timestamp: i32 = bytes_to_i32_be!(data, 40);
        let nanoseconds: u32 = bytes_to_u32_be!(data, 44);

        inode.modification_time = if timestamp == 0 && nanoseconds == 0 {
            DateTime::NotSet
        } else if nanoseconds == 0 {
            DateTime::PosixTime32(PosixTime32::new(timestamp))
        } else {
            DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp as i64, nanoseconds))
        };
        let timestamp: i32 = bytes_to_i32_be!(data, 48);
        let nanoseconds: u32 = bytes_to_u32_be!(data, 52);

        inode.change_time = if timestamp == 0 && nanoseconds == 0 {
            DateTime::NotSet
        } else if nanoseconds == 0 {
            DateTime::PosixTime32(PosixTime32::new(timestamp))
        } else {
            DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp as i64, nanoseconds))
        };
        inode.data_size = bytes_to_u64_be!(data, 56);
        inode.number_of_extents = bytes_to_u32_be!(data, 76) as u64;
        inode.number_of_attributes_extents = bytes_to_u16_be!(data, 80);
        inode.attributes_fork_offset = data[82];
        inode.attributes_fork_type = data[83];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
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

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsInode::new();
        XfsInodeV2::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);
        assert_eq!(test_struct.fork_type, 2);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(test_struct.number_of_links, 3);
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

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV2::read_data(&mut test_struct, &test_data[0..99]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV2::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[4] = 0xff;

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV2::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
