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

use super::catalog_file_record::HfsCatalogFileRecord;
use super::fork_descriptor::HfsForkDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u16", format = "hex"),
        field(name = "flags", data_type = "u16", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 4]"),
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
        field(name = "file_information", data_type = "[u8; 16]"),
        field(name = "extended_file_information", data_type = "[u8; 16]"),
        field(name = "text_encoding_hint", data_type = "u32"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(
            name = "data_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
        field(
            name = "resource_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) catalog file record.
pub struct HfsExtendedCatalogFileRecord {}

impl HfsExtendedCatalogFileRecord {
    /// Reads the catalog file record from a buffer.
    pub fn read_data(
        file_record: &mut HfsCatalogFileRecord,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 248 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        file_record.record_type = bytes_to_u16_be!(data, 0);

        if file_record.record_type != 0x0002 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                file_record.record_type
            )));
        }
        file_record.flags = bytes_to_u16_be!(data, 2);
        file_record.identifier = bytes_to_u32_be!(data, 8);

        let timestamp: u32 = bytes_to_u32_be!(data, 12);
        if timestamp > 0 {
            file_record.creation_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 16);
        if timestamp > 0 {
            file_record.modification_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 20);
        if timestamp == 0 {
            file_record.change_time = Some(DateTime::NotSet);
        } else {
            file_record.change_time = Some(DateTime::HfsTime(HfsTime::new(timestamp)));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 24);
        if timestamp == 0 {
            file_record.access_time = Some(DateTime::NotSet);
        } else {
            file_record.access_time = Some(DateTime::HfsTime(HfsTime::new(timestamp)));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 28);
        if timestamp > 0 {
            file_record.backup_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        file_record.owner_identifier = Some(bytes_to_u32_be!(data, 32));
        file_record.group_identifier = Some(bytes_to_u32_be!(data, 36));
        file_record.file_mode = Some(bytes_to_u16_be!(data, 42));

        let special_permissions: u32 = bytes_to_u32_be!(data, 44);
        if file_record.flags & 0x0020 != 0 && &data[48..56] == b"hlnkhfs+" {
            file_record.link_reference = Some(special_permissions);
        } else {
            file_record.special_permissions = Some(special_permissions);
        }
        if file_record.flags & 0x0080 != 0 {
            let timestamp: i32 = bytes_to_i32_be!(data, 68);
            if timestamp == 0 {
                file_record.added_time = Some(DateTime::NotSet);
            } else {
                file_record.added_time = Some(DateTime::PosixTime32(PosixTime32::new(timestamp)));
            }
        }
        match file_record.data_fork_descriptor.read_data(&data[88..168]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data fork descriptor");
                return Err(error);
            }
        }
        match file_record
            .resource_fork_descriptor
            .read_data(&data[168..248])
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read resource fork descriptor"
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::hfs::extent_descriptor::HfsExtentDescriptor;
    use crate::hfs::fork_descriptor::HfsForkDescriptor;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x02, 0x00, 0xa2, 0x00, 0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x15, 0xe3, 0x5f,
            0xb8, 0xa4, 0xe3, 0x5f, 0xb8, 0xa4, 0xe3, 0x5f, 0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xb5,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x81, 0xa4, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x3a,
            0x08, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0xcf, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogFileRecord::new();
        HfsExtendedCatalogFileRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0002);
        assert_eq!(test_struct.flags, 0x00a2);
        assert_eq!(test_struct.identifier, 21);
        assert_eq!(
            test_struct.data_fork_descriptor,
            HfsForkDescriptor {
                size: 9,
                number_of_blocks: 1,
                extents: vec![HfsExtentDescriptor {
                    block_number: 463,
                    number_of_blocks: 1
                }],
            }
        );
        assert_eq!(
            test_struct.resource_fork_descriptor,
            HfsForkDescriptor {
                size: 0,
                number_of_blocks: 0,
                extents: vec![],
            }
        );
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            }))
        );
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701237,
            }))
        );
        assert_eq!(test_struct.owner_identifier, Some(501));
        assert_eq!(test_struct.group_identifier, Some(20));
        assert_eq!(test_struct.file_mode, Some(0o100644));
        assert_eq!(test_struct.special_permissions, Some(2));
        assert_eq!(test_struct.link_reference, None);

        assert_eq!(
            test_struct.added_time,
            Some(DateTime::PosixTime32(PosixTime32 {
                timestamp: 1731856442,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogFileRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsExtendedCatalogFileRecord::read_data(&mut test_struct, &test_data[0..247]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_record_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsCatalogFileRecord::new();
        let result = HfsExtendedCatalogFileRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
