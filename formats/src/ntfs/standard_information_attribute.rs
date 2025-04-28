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

use std::io;

use datetime::{DateTime, Filetime};
use layout_map::LayoutMap;
use types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "creation_time", data_type = "Filetime")),
        member(field(name = "modification_time", data_type = "Filetime")),
        member(field(name = "entry_modification_time", data_type = "Filetime")),
        member(field(name = "access_time", data_type = "Filetime")),
        member(field(name = "file_attribute_flags", data_type = "u32", format = "hex")),
        member(field(name = "maximum_number_of_versions", data_type = "u32")),
        member(field(name = "version_number", data_type = "u32")),
        member(field(name = "class_identifier", data_type = "u32")),
        member(field(name = "owner_identifier", data_type = "u32")),
        member(field(name = "security_descriptor_identifier", data_type = "u32")),
        member(field(name = "quota_charged", data_type = "[u8; 8]")),
        member(field(name = "update_sequence_number", data_type = "u64")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) standard information attribute ($STANDARD_INFORMATION).
pub struct NtfsStandardInformationAttribute {
    /// Creation time.
    pub creation_time: DateTime,

    /// Modification time.
    pub modification_time: DateTime,

    /// Entry modification time.
    pub entry_modification_time: DateTime,

    /// Access time.
    pub access_time: DateTime,

    /// File attribute flags.
    pub file_attribute_flags: u32,

    /// Maximum number of versions.
    pub maximum_number_of_versions: u32,

    /// Version number.
    pub version_number: u32,
}

impl NtfsStandardInformationAttribute {
    /// Creates a new attribute.
    pub fn new() -> Self {
        Self {
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            entry_modification_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            file_attribute_flags: 0,
            maximum_number_of_versions: 0,
            version_number: 0,
        }
    }

    /// Reads the attribute from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 48 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        let filetime: Filetime = Filetime::from_bytes(&data);

        self.creation_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[8..]);

        self.modification_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[16..]);

        self.entry_modification_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        let filetime: Filetime = Filetime::from_bytes(&data[24..]);

        self.access_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        self.file_attribute_flags = bytes_to_u32_le!(data, 32);
        self.maximum_number_of_versions = bytes_to_u32_le!(data, 36);
        self.version_number = bytes_to_u32_le!(data, 40);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e, 0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e,
            0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c, 0xdc, 0x8e, 0xd0, 0x01, 0xad, 0xca, 0xbc, 0x0c,
            0xdc, 0x8e, 0xd0, 0x01, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsStandardInformationAttribute::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.creation_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad
            })
        );
        assert_eq!(
            test_struct.entry_modification_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad
            })
        );
        assert_eq!(
            test_struct.access_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x01d08edc0cbccaad
            })
        );
        assert_eq!(test_struct.file_attribute_flags, 0x00000006);
        assert_eq!(test_struct.maximum_number_of_versions, 0);
        assert_eq!(test_struct.version_number, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsStandardInformationAttribute::new();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }
}
