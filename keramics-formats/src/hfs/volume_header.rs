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
use keramics_types::bytes_to_u32_be;

use super::constants::*;
use super::enums::HfsFormat;
use super::fork_descriptor::HfsForkDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<2>"),
        field(name = "format_version", data_type = "u16"),
        field(name = "attribute_flags", data_type = "u32"),
        field(name = "last_mounted_version", data_type = "ByteString<4>"),
        field(name = "journal_information_block_number", data_type = "u32"),
        field(name = "creation_time", data_type = "HfsTime"),
        field(name = "modification_time", data_type = "HfsTime"),
        field(name = "backup_time", data_type = "HfsTime"),
        field(name = "checked_time", data_type = "HfsTime"),
        field(name = "number_of_files", data_type = "u32"),
        field(name = "number_of_directories", data_type = "u32"),
        field(name = "block_size", data_type = "u32"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "number_of_unused_blocks", data_type = "u32"),
        field(name = "next_available_block_number", data_type = "u32"),
        field(name = "resource_fork_clump_size", data_type = "u32"),
        field(name = "data_fork_clump_size", data_type = "u32"),
        field(name = "next_available_catalog_node_identifier", data_type = "u32"),
        field(name = "volume_write_count", data_type = "u32"),
        field(name = "encodings_bitmap", data_type = "[u8; 8]"),
        field(name = "finder_information", data_type = "[u8; 32]"),
        field(
            name = "allocation_file_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
        field(
            name = "extents_overflow_file_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
        field(
            name = "catalog_file_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
        field(
            name = "attributes_file_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
        field(
            name = "startup_file_fork_descriptor",
            data_type = "Struct<HfsForkDescriptor; 80>"
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) volume header.
pub struct HfsVolumeHeader {
    /// Format.
    pub format: HfsFormat,

    /// Block size.
    pub block_size: u32,

    /// Extents overflow file fork descriptor.
    pub extents_overflow_file_fork_descriptor: HfsForkDescriptor,

    /// Catalog file fork descriptor.
    pub catalog_file_fork_descriptor: HfsForkDescriptor,

    /// Attributes file fork descriptor.
    pub attributes_file_fork_descriptor: HfsForkDescriptor,
}

impl HfsVolumeHeader {
    /// Creates a new volume header.
    pub fn new() -> Self {
        Self {
            format: HfsFormat::HfsPlus,
            block_size: 0,
            extents_overflow_file_fork_descriptor: HfsForkDescriptor::new(),
            catalog_file_fork_descriptor: HfsForkDescriptor::new(),
            attributes_file_fork_descriptor: HfsForkDescriptor::new(),
        }
    }

    /// Reads the volume header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.format = match &data[0..2] {
            HFSPLUS_VOLUME_HEADER_SIGNATURE => HfsFormat::HfsPlus,
            HFSX_VOLUME_HEADER_SIGNATURE => HfsFormat::HfsX,
            _ => {
                return Err(keramics_core::error_trace_new!("Unsupported signature"));
            }
        };
        self.block_size = bytes_to_u32_be!(data, 40);

        match self
            .extents_overflow_file_fork_descriptor
            .read_data(&data[192..272])
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read extents overflow file fork descriptor"
                );
                return Err(error);
            }
        }
        match self.catalog_file_fork_descriptor.read_data(&data[272..352]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read catalog file fork descriptor"
                );
                return Err(error);
            }
        }
        match self
            .attributes_file_fork_descriptor
            .read_data(&data[352..432])
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read attributes file fork descriptor"
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

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x48, 0x2b, 0x00, 0x04, 0x80, 0x00, 0x01, 0x00, 0x31, 0x30, 0x2e, 0x30, 0x00, 0x00,
            0x00, 0x00, 0xe3, 0x5f, 0xc6, 0xca, 0xe3, 0x5f, 0xb8, 0xbb, 0x00, 0x00, 0x00, 0x00,
            0xe3, 0x5f, 0xb8, 0xba, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x01, 0xd6, 0x00, 0x00, 0x01, 0x90, 0x00, 0x00, 0x01, 0xd4,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe5, 0x57, 0xc2, 0xa1, 0x7f, 0xaa, 0x73, 0xe2,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x40, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x40, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0xf2, 0x00, 0x00,
            0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
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

        let mut test_struct = HfsVolumeHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format, HfsFormat::HfsPlus);
        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(
            test_struct.extents_overflow_file_fork_descriptor,
            HfsForkDescriptor {
                size: 81920,
                number_of_blocks: 20,
                extents: vec![HfsExtentDescriptor {
                    block_number: 2,
                    number_of_blocks: 20
                },],
            }
        );
        assert_eq!(
            test_struct.catalog_file_fork_descriptor,
            HfsForkDescriptor {
                size: 81920,
                number_of_blocks: 20,
                extents: vec![HfsExtentDescriptor {
                    block_number: 242,
                    number_of_blocks: 20
                },],
            }
        );
        assert_eq!(
            test_struct.attributes_file_fork_descriptor,
            HfsForkDescriptor {
                size: 81920,
                number_of_blocks: 20,
                extents: vec![HfsExtentDescriptor {
                    block_number: 22,
                    number_of_blocks: 20
                },],
            }
        );
        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsVolumeHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsVolumeHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
