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

use super::extent_descriptor::HfsExtentDescriptor;
use super::extent_descriptor_extended::HfsExtendedExtentDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(
            name = "extents",
            data_type = "[Struct<HfsExtendedExtentDescriptor; 8>; 8]"
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) attribute fork data record.
pub struct HfsAttributeExtentsRecord {
    /// Record type.
    pub record_type: u32,

    /// Extents.
    pub extents: Vec<HfsExtentDescriptor>,
}

impl HfsAttributeExtentsRecord {
    /// Creates a new attribute record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            extents: Vec::new(),
        }
    }

    /// Reads the attribute record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 72 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.record_type = bytes_to_u32_be!(data, 0);

        if self.record_type != 0x00000030 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:08x}",
                self.record_type
            )));
        }
        for data_offset in (8..72).step_by(8) {
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
            0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf2, 0x00, 0x00,
            0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsAttributeExtentsRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.record_type, 0x00000030);
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
        let mut test_struct = HfsAttributeExtentsRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..71]);
        assert!(result.is_err());
    }
}
