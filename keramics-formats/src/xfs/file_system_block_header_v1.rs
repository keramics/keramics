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
        field(name = "next_block_number", data_type = "u32"),
        field(name = "previous_block_number", data_type = "u32"),
        field(name = "signature", data_type = "[u8; 2]", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 2]"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) file system block header version 1.
#[allow(dead_code)]
pub struct XfsFileSystemBlockHeaderV1 {}

impl XfsFileSystemBlockHeaderV1 {
    /// Creates a new header.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the header from a buffer.
    #[allow(dead_code)]
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 12 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfb, 0xee, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsFileSystemBlockHeaderV1::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsFileSystemBlockHeaderV1::new();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }
}
