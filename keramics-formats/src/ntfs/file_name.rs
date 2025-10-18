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

use keramics_core::ErrorTrace;
use keramics_datetime::{DateTime, Filetime};
use keramics_layout_map::LayoutMap;
use keramics_types::{Ucs2String, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;
use super::mft_attribute::NtfsMftAttribute;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "parent_file_reference", data_type = "u64", format = "hex")),
        member(field(name = "creation_time", data_type = "Filetime")),
        member(field(name = "modification_time", data_type = "Filetime")),
        member(field(name = "entry_modification_time", data_type = "Filetime")),
        member(field(name = "access_time", data_type = "Filetime")),
        member(field(name = "allocated_data_size", data_type = "u64")),
        member(field(name = "data_size", data_type = "u64")),
        member(field(name = "file_attribute_flags", data_type = "u32", format = "hex")),
        member(field(name = "extended_data", data_type = "[u8; 4]")),
        member(field(name = "name_size", data_type = "u8")),
        member(field(name = "name_space", data_type = "u8")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) file name ($FILE_NAME).
pub struct NtfsFileName {
    /// Parent file reference.
    pub parent_file_reference: u64,

    /// Creation time.
    pub creation_time: DateTime,

    /// Modification time.
    pub modification_time: DateTime,

    /// Entry modification time.
    pub entry_modification_time: DateTime,

    /// Access time.
    pub access_time: DateTime,

    /// Data size.
    pub data_size: u64,

    /// File attribute flags.
    pub file_attribute_flags: u32,

    /// Name size.
    pub name_size: u8,

    /// Name space.
    pub name_space: u8,

    /// Name.
    pub name: Ucs2String,

    /// Reparse point tag.
    pub reparse_point_tag: Option<u32>,
}

impl NtfsFileName {
    /// Creates a new file name.
    pub fn new() -> Self {
        Self {
            parent_file_reference: 0,
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            entry_modification_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            data_size: 0,
            file_attribute_flags: 0,
            name_size: 0,
            name_space: 0,
            name: Ucs2String::new(),
            reparse_point_tag: None,
        }
    }

    /// Reads the file name from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();
        if data_size < 66 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported NTFS file name data size"
            ));
        }
        self.parent_file_reference = bytes_to_u64_le!(data, 0);

        let filetime: Filetime = Filetime::from_bytes(&data[8..]);

        self.creation_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[16..]);

        self.modification_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[24..]);

        self.entry_modification_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[32..]);

        self.access_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        self.data_size = bytes_to_u64_le!(data, 48);
        self.file_attribute_flags = bytes_to_u32_le!(data, 56);

        if self.file_attribute_flags & 0x00000400 != 0 {
            self.reparse_point_tag = Some(bytes_to_u32_le!(data, 60));
        }
        self.name_size = data[64];
        self.name_space = data[65];

        if data_size > 66 {
            let data_end_offset: usize = 66 + (self.name_size as usize) * 2;

            if data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported NTFS file name data size"
                ));
            }
            self.name.read_data_le(&data[66..data_end_offset]);
        }
        Ok(())
    }

    /// Reads the file name from a MFT attribute.
    pub fn from_attribute(mft_attribute: &NtfsMftAttribute) -> Result<Self, ErrorTrace> {
        if mft_attribute.attribute_type != NTFS_ATTRIBUTE_TYPE_FILE_NAME {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported attribute type: 0x{:08x}",
                mft_attribute.attribute_type
            )));
        }
        if !mft_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported non-resident $FILE_NAME attribute"
            ));
        }
        let mut file_name: NtfsFileName = NtfsFileName::new();

        match file_name.read_data(&mft_attribute.resident_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file name");
                return Err(error);
            }
        }
        Ok(file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e,
            0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e, 0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c,
            0xdc, 0x8e, 0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e, 0xd0, 0x01, 0x00, 0x40,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x03, 0x24, 0x00, 0x4d, 0x00,
            0x46, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = NtfsFileName::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.parent_file_reference, 0x0005000000000005);
        assert_eq!(
            test_struct.creation_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad,
            })
        );
        assert_eq!(
            test_struct.entry_modification_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad,
            })
        );
        assert_eq!(
            test_struct.access_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad,
            })
        );
        assert_eq!(test_struct.name_size, 4);
        assert_eq!(test_struct.name_space, 3);
        assert_eq!(test_struct.name.to_string(), "$MFT");
        assert!(test_struct.reparse_point_tag.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsFileName::new();
        let result = test_struct.read_data(&test_data[0..65]);
        assert!(result.is_err());
    }

    // TODO: add tests for from_attribute
}
