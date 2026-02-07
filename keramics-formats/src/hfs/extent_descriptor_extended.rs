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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "block_number", data_type = "u32"),
        field(name = "number_of_blocks", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) extent descriptor.
pub struct HfsExtendedExtentDescriptor {}

impl HfsExtendedExtentDescriptor {
    /// Reads the extent descriptor from a buffer.
    pub fn read_data(
        extent_descriptor: &mut HfsExtentDescriptor,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        extent_descriptor.block_number = bytes_to_u32_be!(data, 0);
        extent_descriptor.number_of_blocks = bytes_to_u32_be!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x00, 0x00, 0x00, 0xf2, 0x00, 0x00, 0x00, 0x14];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsExtentDescriptor::new();
        HfsExtendedExtentDescriptor::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.block_number, 242);
        assert_eq!(test_struct.number_of_blocks, 20);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsExtentDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsExtendedExtentDescriptor::read_data(&mut test_struct, &test_data[0..7]);
        assert!(result.is_err());
    }
}
