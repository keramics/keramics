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
use keramics_types::{bytes_to_u32_be, bytes_to_u64_be};

use super::extent_descriptor::HfsExtentDescriptor;
use super::extent_descriptor_extended::HfsExtendedExtentDescriptor;

#[derive(Clone, Debug, LayoutMap, PartialEq)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "size", data_type = "u64"),
        field(name = "clump_size", data_type = "u32"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(
            name = "extents",
            data_type = "[Struct<HfsExtendedExtentDescriptor; 8>; 8]"
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) fork descriptor.
pub struct HfsForkDescriptor {
    /// Size.
    pub size: u64,

    /// Number of blocks.
    pub number_of_blocks: u32,

    /// Extents.
    pub extents: Vec<HfsExtentDescriptor>,
}

impl HfsForkDescriptor {
    /// Creates a new fork descriptor.
    pub fn new() -> Self {
        Self {
            size: 0,
            number_of_blocks: 0,
            extents: Vec::new(),
        }
    }

    /// Reads the fork descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 80 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.size = bytes_to_u64_be!(data, 0);
        self.number_of_blocks = bytes_to_u32_be!(data, 12);

        for data_offset in (16..80).step_by(8) {
            let data_end_offset = data_offset + 8;

            if &data[data_offset..data_end_offset] == [0; 8] {
                break;
            }
            let mut extent_descriptor: HfsExtentDescriptor = HfsExtentDescriptor::new();

            match HfsExtendedExtentDescriptor::read_data(
                &mut extent_descriptor,
                &data[data_offset..data_end_offset],
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read extent descriptor at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            self.extents.push(extent_descriptor);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x14, 0x00, 0x00, 0x00, 0xf2, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsForkDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.size, 81920);
        assert_eq!(test_struct.number_of_blocks, 20);
        assert_eq!(
            test_struct.extents,
            vec![HfsExtentDescriptor {
                block_number: 242,
                number_of_blocks: 20
            },],
        );
        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsForkDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..63]);
        assert!(result.is_err());
    }
}
