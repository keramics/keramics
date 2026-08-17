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
use keramics_datetime::{ApfsTime, DateTime};
use keramics_layout_map::LayoutMap;
use keramics_types::{
    ByteString, bytes_to_i64_le, bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le,
};

use super::data_stream_descriptor::ApfsDataStreamDescriptor;
use super::extended_fields::ApfsExtendedFields;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "parent_identifier", data_type = "u64"),
        field(name = "data_stream_identifier", data_type = "u64"),
        field(name = "modification_time", data_type = "ApfsTime"),
        field(name = "creation_time", data_type = "ApfsTime"),
        field(name = "inode_change_time", data_type = "ApfsTime"),
        field(name = "access_time", data_type = "ApfsTime"),
        field(name = "inode_flags", data_type = "u64", format = "hex"),
        field(name = "number_of_links", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "bsd_flags", data_type = "u32"),
        field(name = "owner_identifier", data_type = "u32"),
        field(name = "group_identifier", data_type = "u32"),
        field(name = "file_mode", data_type = "u16"),
        field(name = "unknown3", data_type = "[u8; 2]"),
        field(name = "unknown4", data_type = "[u8; 8]"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) inode.
pub struct ApfsInode {
    /// Parent identifier.
    pub parent_identifier: u64,

    /// Data stream identifier.
    pub data_stream_identifier: u64,

    /// Modification date and time.
    pub modification_time: DateTime,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Change date and time.
    pub change_time: DateTime,

    /// Access date and time.
    pub access_time: DateTime,

    /// Number of links.
    pub number_of_links: u32,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// File mode.
    pub file_mode: u16,

    /// Name.
    pub name: Option<ByteString>,

    /// Data stream descriptor.
    pub data_stream_descriptor: Option<ApfsDataStreamDescriptor>,
}

impl ApfsInode {
    /// Creates an inode.
    pub fn new() -> Self {
        Self {
            parent_identifier: 0,
            data_stream_identifier: 0,
            modification_time: DateTime::NotSet,
            creation_time: DateTime::NotSet,
            change_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            number_of_links: 0,
            owner_identifier: 0,
            group_identifier: 0,
            file_mode: 0,
            name: None,
            data_stream_descriptor: None,
        }
    }

    /// Reads the value from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 98 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.parent_identifier = bytes_to_u64_le!(data, 0);
        self.data_stream_identifier = bytes_to_u64_le!(data, 8);

        let timestamp: i64 = bytes_to_i64_le!(data, 16);

        self.modification_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::ApfsTime(ApfsTime::new(timestamp))
        };
        let timestamp: i64 = bytes_to_i64_le!(data, 24);

        self.creation_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::ApfsTime(ApfsTime::new(timestamp))
        };
        let timestamp: i64 = bytes_to_i64_le!(data, 32);

        self.change_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::ApfsTime(ApfsTime::new(timestamp))
        };
        let timestamp: i64 = bytes_to_i64_le!(data, 40);

        self.access_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::ApfsTime(ApfsTime::new(timestamp))
        };
        self.number_of_links = bytes_to_u32_le!(data, 56);
        self.owner_identifier = bytes_to_u32_le!(data, 72);
        self.group_identifier = bytes_to_u32_le!(data, 76);
        self.file_mode = bytes_to_u16_le!(data, 80);

        if data_size >= 96 {
            let mut extended_fields: ApfsExtendedFields = ApfsExtendedFields::new();

            match extended_fields.read_data(&data[92..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read extended fields");
                    return Err(error);
                }
            }
            self.name = match extended_fields.get(&4) {
                Some(field_data) => Some(ByteString::from(field_data)),
                None => None,
            };
            self.data_stream_descriptor = match extended_fields.get(&8) {
                Some(field_data) => {
                    keramics_core::debug_trace_structure!(
                        ApfsDataStreamDescriptor::debug_read_data(&field_data)
                    );
                    let mut data_stream_descriptor: ApfsDataStreamDescriptor =
                        ApfsDataStreamDescriptor::new();

                    match data_stream_descriptor.read_data(field_data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read data stream descriptor"
                            );
                            return Err(error);
                        }
                    }
                    Some(data_stream_descriptor)
                }
                None => None,
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xd7, 0x4f, 0x17, 0x4e, 0x6a, 0xdd, 0x59, 0x15, 0x1b, 0xf8, 0x41, 0xaf,
            0x6a, 0xdd, 0x59, 0x15, 0x1b, 0xf8, 0x41, 0xaf, 0x6a, 0xdd, 0x59, 0x15, 0xd7, 0x4f,
            0x17, 0x4e, 0x6a, 0xdd, 0x59, 0x15, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfd, 0x41, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x04, 0x02,
            0x05, 0x00, 0x72, 0x6f, 0x6f, 0x74, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsInode::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.data_stream_identifier, 2);
        assert_eq!(
            test_struct.modification_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1538504196370157527
            })
        );
        assert_eq!(
            test_struct.creation_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1538504198000343067
            })
        );
        assert_eq!(
            test_struct.change_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1538504198000343067
            })
        );
        assert_eq!(
            test_struct.access_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1538504196370157527
            })
        );
        assert_eq!(test_struct.number_of_links, 2);
        assert_eq!(test_struct.owner_identifier, 0);
        assert_eq!(test_struct.group_identifier, 0);
        assert_eq!(test_struct.file_mode, 0o40775);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsInode::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..97]);
        assert!(result.is_err());
    }
}
