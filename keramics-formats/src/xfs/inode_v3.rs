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
use keramics_datetime::{DateTime, PosixTime32, PosixTime64Ns, XfsBigtime};
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
        field(name = "project_identifier_lower", data_type = "u16"),
        field(name = "project_identifier_upper", data_type = "u16"),
        // number_of_data_extents_64bit if XFS_SB_FEAT_INCOMPAT_NREXT64 is set otherwise unknown
        field(name = "number_of_data_extents_64bit", data_type = "[u8; 8]"),
        field(name = "access_time", data_type = "[u8; 8]"),
        field(name = "modification_time", data_type = "[u8; 8]"),
        field(name = "change_time", data_type = "[u8; 8]"),
        field(name = "data_size", data_type = "u64"),
        field(name = "number_of_blocks", data_type = "u64"),
        field(name = "extent_size", data_type = "u32"),
        // number_of_attributes_extents_32bit if XFS_SB_FEAT_INCOMPAT_NREXT64 is set otherwise
        // number_of_data_extents
        field(name = "number_of_data_extents", data_type = "u32"),
        // unknown if XFS_SB_FEAT_INCOMPAT_NREXT64 is set otherwise number_of_attributes_extents
        field(name = "number_of_attributes_extents", data_type = "u16"),
        field(name = "attributes_fork_offset", data_type = "u8"),
        field(name = "attributes_fork_type", data_type = "u8"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "unknown3", data_type = "[u8; 2]"),
        field(name = "inode_flags", data_type = "u16", format = "hex"),
        field(name = "generation_number", data_type = "u32"),
        field(name = "unknown4", data_type = "[u8; 4]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "change_count", data_type = "u64"),
        field(name = "log_sequence_number", data_type = "u64"),
        field(name = "extended_inode_flags", data_type = "u64"),
        field(name = "cow_extent_size", data_type = "u32"),
        field(name = "unknown5", data_type = "[u8; 12]"),
        field(name = "creation_time", data_type = "[u8; 8]"),
        field(name = "inode_number", data_type = "u64"),
        field(name = "inode_type_identifier", data_type = "Uuid"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) inode version 3.
pub struct XfsInodeV3 {}

impl XfsInodeV3 {
    /// Reads the inode from a buffer.
    pub fn read_data(
        inode: &mut XfsInode,
        has_bigtime: bool,
        has_64bit_number_of_extents: bool,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 176 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != XFS_INODE_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        if data[4] != 3 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported format version"
            ));
        }
        inode.file_mode = bytes_to_u16_be!(data, 2);
        inode.fork_type = data[5];
        inode.owner_identifier = bytes_to_u32_be!(data, 8);
        inode.group_identifier = bytes_to_u32_be!(data, 12);
        inode.number_of_links = bytes_to_u32_be!(data, 16);

        if has_64bit_number_of_extents {
            inode.number_of_extents = bytes_to_u64_be!(data, 24);
        }
        if has_bigtime {
            let timestamp: u64 = bytes_to_u64_be!(data, 32);

            inode.access_time = if timestamp == 0 {
                DateTime::NotSet
            } else {
                DateTime::XfsBigtime(XfsBigtime::new(timestamp))
            };
            let timestamp: u64 = bytes_to_u64_be!(data, 40);

            inode.modification_time = if timestamp == 0 {
                DateTime::NotSet
            } else {
                DateTime::XfsBigtime(XfsBigtime::new(timestamp))
            };
            let timestamp: u64 = bytes_to_u64_be!(data, 48);

            inode.change_time = if timestamp == 0 {
                DateTime::NotSet
            } else {
                DateTime::XfsBigtime(XfsBigtime::new(timestamp))
            };
        } else {
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
        }
        inode.data_size = bytes_to_u64_be!(data, 56);

        if !has_64bit_number_of_extents {
            inode.number_of_extents = bytes_to_u32_be!(data, 76) as u64;
        }
        inode.number_of_attributes_extents = bytes_to_u16_be!(data, 80);
        inode.attributes_fork_offset = data[82];
        inode.attributes_fork_type = data[83];

        if has_bigtime {
            let timestamp: u64 = bytes_to_u64_be!(data, 144);

            inode.creation_time = if timestamp == 0 {
                Some(DateTime::NotSet)
            } else {
                Some(DateTime::XfsBigtime(XfsBigtime::new(timestamp)))
            };
        } else {
            let timestamp: i32 = bytes_to_i32_be!(data, 144);
            let nanoseconds: u32 = bytes_to_u32_be!(data, 148);

            inode.creation_time = if timestamp == 0 && nanoseconds == 0 {
                Some(DateTime::NotSet)
            } else if nanoseconds == 0 {
                Some(DateTime::PosixTime32(PosixTime32::new(timestamp)))
            } else {
                Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                    timestamp as i64,
                    nanoseconds,
                )))
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
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
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsInode::new();
        XfsInodeV3::read_data(&mut test_struct, false, true, &test_data)?;

        assert_eq!(test_struct.file_mode, 0o40755);
        assert_eq!(test_struct.fork_type, 1);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(test_struct.number_of_links, 3);
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

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV3::read_data(&mut test_struct, false, true, &test_data[0..175]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV3::read_data(&mut test_struct, false, true, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[4] = 0xff;

        let mut test_struct = XfsInode::new();
        let result = XfsInodeV3::read_data(&mut test_struct, false, true, &test_data);
        assert!(result.is_err());
    }
}
