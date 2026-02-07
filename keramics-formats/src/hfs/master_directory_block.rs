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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_be, bytes_to_u32_be};

use super::constants::*;
use super::extent_descriptor::HfsExtentDescriptor;
use super::extent_descriptor_standard::HfsStandardExtentDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<2>"),
        field(name = "creation_time", data_type = "HfsTime"),
        field(name = "modification_time", data_type = "HfsTime"),
        field(name = "attribute_flags", data_type = "u16"),
        field(name = "number_of_files_in_root", data_type = "u16"),
        field(name = "bitmap_block_number", data_type = "u16"),
        field(name = "next_allocation_search_block_number", data_type = "u16"),
        field(name = "number_of_blocks", data_type = "u16"),
        field(name = "block_size", data_type = "u32"),
        field(name = "clump_size", data_type = "u32"),
        field(name = "data_area_block_number", data_type = "u16"),
        field(name = "next_available_catalog_node_identifier", data_type = "u32"),
        field(name = "number_of_unused_blocks", data_type = "u16"),
        field(name = "volume_label_size", data_type = "u8"),
        field(name = "volume_label", data_type = "[u8; 27]"),
        field(name = "backup_time", data_type = "HfsTime"),
        field(name = "backup_sequence_number", data_type = "u16"),
        field(name = "volume_write_count", data_type = "u32"),
        field(name = "extents_overflow_file_clump_size", data_type = "u32"),
        field(name = "catalog_file_clump_size", data_type = "u32"),
        field(name = "number_of_directories_in_root", data_type = "u16"),
        field(name = "number_of_files", data_type = "u32"),
        field(name = "number_of_directories", data_type = "u32"),
        field(name = "finder_information", data_type = "[u8; 32]"),
        field(name = "embedded_volume_signature", data_type = "u16"),
        field(name = "embedded_volume_extent_descriptor", data_type = "[u8; 4]"),
        field(name = "extents_overflow_file_size", data_type = "u32"),
        field(
            name = "extents_overflow_file_extents_record",
            data_type = "[Struct<HfsStandardExtentDescriptor; 4>; 3]"
        ),
        field(name = "catalog_file_size", data_type = "u32"),
        field(
            name = "catalog_file_extents_record",
            data_type = "[Struct<HfsStandardExtentDescriptor; 4>; 3]"
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS standard) master directory block.
pub struct HfsMasterDirectoryBlock {
    /// Block size.
    pub block_size: u32,

    /// Data area block number.
    pub data_area_block_number: u16,

    /// Extents overflow file size.
    pub extents_overflow_file_size: u32,

    /// Extents overflow file extents.
    pub extents_overflow_file_extents: Vec<HfsExtentDescriptor>,

    /// Catalog file size.
    pub catalog_file_size: u32,

    /// Catalog file extents.
    pub catalog_file_extents: Vec<HfsExtentDescriptor>,
}

impl HfsMasterDirectoryBlock {
    /// Creates a new master directory block.
    pub fn new() -> Self {
        Self {
            block_size: 0,
            data_area_block_number: 0,
            extents_overflow_file_size: 0,
            extents_overflow_file_extents: Vec::new(),
            catalog_file_size: 0,
            catalog_file_extents: Vec::new(),
        }
    }

    /// Reads the master directory block from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 162 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != HFS_MASTER_DIRECTORY_BLOCK_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.block_size = bytes_to_u32_be!(data, 20);
        self.data_area_block_number = bytes_to_u16_be!(data, 28);
        self.extents_overflow_file_size = bytes_to_u32_be!(data, 130);

        for data_offset in (134..146).step_by(4) {
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
                            "Unable to read extents overflow file extent descriptor at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            self.extents_overflow_file_extents.push(extent_descriptor);
        }
        self.catalog_file_size = bytes_to_u32_be!(data, 146);

        for data_offset in (150..162).step_by(4) {
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
                            "Unable to read catalog file extent descriptor at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            self.catalog_file_extents.push(extent_descriptor);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x42, 0x44, 0xe5, 0x79, 0x60, 0xda, 0xe5, 0x79, 0x60, 0xda, 0x01, 0x00, 0x00, 0x01,
            0x00, 0x03, 0x00, 0x7e, 0x1f, 0xf9, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x00, 0x05, 0x00, 0x00, 0x00, 0x14, 0x1f, 0x63, 0x08, 0x68, 0x66, 0x73, 0x5f, 0x74,
            0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x3f, 0x00, 0x3f,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsMasterDirectoryBlock::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_size, 512);
        assert_eq!(test_struct.data_area_block_number, 5);
        assert_eq!(test_struct.extents_overflow_file_size, 32256);
        assert_eq!(
            test_struct.extents_overflow_file_extents,
            vec![HfsExtentDescriptor {
                block_number: 0,
                number_of_blocks: 63
            }]
        );
        assert_eq!(test_struct.catalog_file_size, 32256);
        assert_eq!(
            test_struct.catalog_file_extents,
            vec![HfsExtentDescriptor {
                block_number: 63,
                number_of_blocks: 63
            }]
        );

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsMasterDirectoryBlock::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..161]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsMasterDirectoryBlock::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
