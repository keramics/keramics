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

use super::fork_descriptor::HfsForkDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "fork_descriptor", data_type = "Struct<HfsForkDescriptor; 80>"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) attribute fork data record.
pub struct HfsAttributeForkDataRecord {
    /// Record type.
    pub record_type: u32,

    /// Fork descriptor.
    pub fork_descriptor: HfsForkDescriptor,
}

impl HfsAttributeForkDataRecord {
    /// Creates a new attribute record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            fork_descriptor: HfsForkDescriptor::new(),
        }
    }

    /// Reads the attribute record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 88 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.record_type = bytes_to_u32_be!(data, 0);

        if self.record_type != 0x00000020 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:08x}",
                self.record_type
            )));
        }
        match self.fork_descriptor.read_data(&data[8..88]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read fork descriptor");
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
            0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x40, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0xf2,
            0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsAttributeForkDataRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.record_type, 0x00000020);
        assert_eq!(
            test_struct.fork_descriptor,
            HfsForkDescriptor {
                size: 81920,
                number_of_blocks: 20,
                extents: vec![HfsExtentDescriptor {
                    block_number: 242,
                    number_of_blocks: 20
                },],
            }
        );

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsAttributeForkDataRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..87]);
        assert!(result.is_err());
    }
}
