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
use keramics_datetime::{DateTime, HfsTime, PosixTime32};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_i32_be, bytes_to_u16_be, bytes_to_u32_be};

use super::catalog_folder_record::HfsCatalogFolderRecord;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u16", format = "hex"),
        field(name = "flags", data_type = "u16", format = "hex"),
        field(name = "number_of_entries", data_type = "u32"),
        field(name = "identifier", data_type = "u32"),
        field(name = "creation_time", data_type = "HfsTime"),
        field(name = "modification_time", data_type = "HfsTime"),
        field(name = "change_time", data_type = "HfsTime"),
        field(name = "access_time", data_type = "HfsTime"),
        field(name = "backup_time", data_type = "HfsTime"),
        field(name = "owner_identifier", data_type = "u32"),
        field(name = "group_identifier", data_type = "u32"),
        field(name = "administration_flags", data_type = "u8", format = "hex"),
        field(name = "owner_flags", data_type = "u8", format = "hex"),
        field(name = "file_mode", data_type = "u16", format = "hex"),
        field(name = "special_permissions", data_type = "u32"),
        field(name = "folder_information", data_type = "[u8; 16]"),
        field(name = "extended_folder_information", data_type = "[u8; 16]"),
        field(name = "text_encoding_hint", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) catalog folder record.
pub struct HfsExtendedCatalogFolderRecord {}

impl HfsExtendedCatalogFolderRecord {
    /// Reads the catalog folder record from a buffer.
    pub fn read_data(
        folder_record: &mut HfsCatalogFolderRecord,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 10 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        folder_record.record_type = bytes_to_u16_be!(data, 0);

        if folder_record.record_type != 0x0001 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                folder_record.record_type
            )));
        }
        folder_record.flags = bytes_to_u16_be!(data, 2);
        folder_record.identifier = bytes_to_u32_be!(data, 8);

        let timestamp: u32 = bytes_to_u32_be!(data, 12);
        if timestamp > 0 {
            folder_record.creation_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 16);
        if timestamp > 0 {
            folder_record.modification_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 20);
        if timestamp > 0 {
            folder_record.change_time = Some(DateTime::HfsTime(HfsTime::new(timestamp)));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 24);
        if timestamp == 0 {
            folder_record.access_time = Some(DateTime::NotSet);
        } else {
            folder_record.access_time = Some(DateTime::HfsTime(HfsTime::new(timestamp)));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 28);
        if timestamp > 0 {
            folder_record.backup_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        folder_record.owner_identifier = Some(bytes_to_u32_be!(data, 32));
        folder_record.group_identifier = Some(bytes_to_u32_be!(data, 36));
        folder_record.file_mode = Some(bytes_to_u16_be!(data, 42));

        if folder_record.flags & 0x0080 != 0 {
            let timestamp: i32 = bytes_to_i32_be!(data, 68);
            if timestamp == 0 {
                folder_record.added_time = Some(DateTime::NotSet);
            } else {
                folder_record.added_time = Some(DateTime::PosixTime32(PosixTime32::new(timestamp)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x02, 0xe3, 0x5f,
            0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xbb, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x41, 0xed, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x7e,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogFolderRecord::new();
        HfsExtendedCatalogFolderRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0001);
        assert_eq!(test_struct.flags, 0x0000);
        assert_eq!(test_struct.identifier, 2);
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701243,
            }))
        );
        assert_eq!(test_struct.access_time, Some(DateTime::NotSet));
        assert_eq!(test_struct.owner_identifier, Some(501));
        assert_eq!(test_struct.group_identifier, Some(20));
        assert_eq!(test_struct.file_mode, Some(0o40755));
        assert_eq!(test_struct.added_time, None);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogFolderRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsExtendedCatalogFolderRecord::read_data(&mut test_struct, &test_data[0..9]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_record_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsCatalogFolderRecord::new();
        let result = HfsExtendedCatalogFolderRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
