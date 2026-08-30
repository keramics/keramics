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

use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) extent list.
pub struct XfsExtentList {}

impl XfsExtentList {
    /// Creates a new extent list.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the extent list from a buffer.
    pub fn read_data(
        &self,
        number_of_extents: u64,
        data: &[u8],
        extents: &mut Vec<XfsPackedExtent>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();
        let data_end_offset: usize = (number_of_extents as usize) * 16;

        if data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of extents value out of bounds"
            ));
        }
        for (extent_index, chunk) in data[0..data_end_offset].chunks_exact(16).enumerate() {
            keramics_core::debug_trace_structure!(XfsPackedExtent::debug_read_data(chunk));

            let mut packed_extent: XfsPackedExtent = XfsPackedExtent::new();

            match packed_extent.read_data(chunk) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read packed extent: {}", extent_index),
                    );
                    return Err(error);
                }
            }
            extents.push(packed_extent);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfb, 0xc0,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let test_struct = XfsExtentList::new();
        let mut extents: Vec<XfsPackedExtent> = Vec::new();
        test_struct.read_data(1, &test_data, &mut extents)?;

        assert_eq!(extents.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_struct = XfsExtentList::new();

        let test_data: Vec<u8> = get_test_data();
        let mut extents: Vec<XfsPackedExtent> = Vec::new();
        let result = test_struct.read_data(1, &test_data[0..15], &mut extents);
        assert!(result.is_err());
    }
}
