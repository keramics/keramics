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
use keramics_datetime::{DateTime, HfsTime};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_be, bytes_to_u32_be};

use super::catalog_file_record::HfsCatalogFileRecord;
use super::extent_descriptor::HfsExtentDescriptor;
use super::extent_descriptor_standard::HfsStandardExtentDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u16", format = "hex"),
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "file_type", data_type = "u8"),
        field(name = "file_information", data_type = "[u8; 16]"),
        field(name = "identifier", data_type = "u32"),
        field(name = "data_fork_block_number", data_type = "u16"),
        field(name = "data_fork_size", data_type = "u32"),
        field(name = "data_fork_allocated_size", data_type = "u32"),
        field(name = "resource_fork_block_number", data_type = "u16"),
        field(name = "resource_fork_size", data_type = "u32"),
        field(name = "resource_fork_allocated_size", data_type = "u32"),
        field(name = "creation_time", data_type = "HfsTime"),
        field(name = "modification_time", data_type = "HfsTime"),
        field(name = "backup_time", data_type = "HfsTime"),
        field(name = "extended_file_information", data_type = "[u8; 16]"),
        field(name = "clump_size", data_type = "u16"),
        field(
            name = "data_fork_extents_record",
            data_type = "[Struct<HfsStandardExtentDescriptor; 4>; 3]"
        ),
        field(
            name = "resource_fork_extents_record",
            data_type = "[Struct<HfsStandardExtentDescriptor; 4>; 3]"
        ),
        field(name = "unknown1", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS standard) catalog file record.
pub struct HfsStandardCatalogFileRecord {}

impl HfsStandardCatalogFileRecord {
    /// Reads the catalog file record from a buffer.
    pub fn read_data(
        file_record: &mut HfsCatalogFileRecord,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 102 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        file_record.record_type = bytes_to_u16_be!(data, 0);

        if file_record.record_type != 0x0200 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                file_record.record_type
            )));
        }
        file_record.flags = data[2] as u16;
        file_record.identifier = bytes_to_u32_be!(data, 20);

        file_record.data_fork_descriptor.size = bytes_to_u32_be!(data, 26) as u64;

        file_record.resource_fork_descriptor.size = bytes_to_u32_be!(data, 36) as u64;

        let timestamp: u32 = bytes_to_u32_be!(data, 44);
        if timestamp > 0 {
            file_record.creation_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 48);
        if timestamp > 0 {
            file_record.modification_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        let timestamp: u32 = bytes_to_u32_be!(data, 52);
        if timestamp > 0 {
            file_record.backup_time = DateTime::HfsTime(HfsTime::new(timestamp));
        }
        for data_offset in (74..86).step_by(4) {
            let data_end_offset = data_offset + 4;

            if &data[data_offset..data_end_offset] == [0; 4] {
                break;
            }
            let mut extent_descriptor: HfsExtentDescriptor = HfsExtentDescriptor::new();

            match HfsStandardExtentDescriptor::read_data(
                &mut extent_descriptor,
                &data[data_offset..data_end_offset],
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read data fork extent descriptor at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            file_record
                .data_fork_descriptor
                .extents
                .push(extent_descriptor);
        }
        for data_offset in (86..98).step_by(4) {
            let data_end_offset = data_offset + 4;

            if &data[data_offset..data_end_offset] == [0; 4] {
                break;
            }
            let mut extent_descriptor: HfsExtentDescriptor = HfsExtentDescriptor::new();

            match HfsStandardExtentDescriptor::read_data(
                &mut extent_descriptor,
                &data[data_offset..data_end_offset],
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read resource fork extent descriptor at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            file_record
                .resource_fork_descriptor
                .extents
                .push(extent_descriptor);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::hfs::fork_descriptor::HfsForkDescriptor;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x02, 0x00, 0x82, 0x00, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x09, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe5, 0x79, 0x60, 0xda, 0xe5, 0x79, 0x60, 0xda, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x09, 0x74,
            0x65, 0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x09, 0x54, 0x65, 0x73,
            0x74, 0x46, 0x69, 0x6c, 0x65, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xb2, 0x01, 0x7c,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogFileRecord::new();
        HfsStandardCatalogFileRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0200);
        assert_eq!(test_struct.flags, 0x0082);
        assert_eq!(test_struct.identifier, 18);
        assert_eq!(
            test_struct.data_fork_descriptor,
            HfsForkDescriptor {
                size: 9,
                number_of_blocks: 0,
                extents: vec![HfsExtentDescriptor {
                    block_number: 126,
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
                timestamp: 3849937114,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3849937114,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet,);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogFileRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsStandardCatalogFileRecord::read_data(&mut test_struct, &test_data[0..101]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_record_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsCatalogFileRecord::new();
        let result = HfsStandardCatalogFileRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
