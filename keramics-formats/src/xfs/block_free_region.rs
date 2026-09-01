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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "offset", data_type = "u16"),
        field(name = "size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) block free region version 2.
#[allow(dead_code)]
pub struct XfsBlockFreeRegion {}

impl XfsBlockFreeRegion {
    /// Creates a new block free region.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the block free region from a buffer.
    #[allow(dead_code)]
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x20, 0x00, 0x00, 0x10]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBlockFreeRegion::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBlockFreeRegion::new();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
