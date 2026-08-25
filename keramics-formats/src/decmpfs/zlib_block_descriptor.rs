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
use keramics_types::bytes_to_u32_le;

#[derive(Debug, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "offset", data_type = "u32"),
        field(name = "size", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// Apple File System Compression (decmpfs) zlib (compressed) block descriptor.
pub struct DecmpfsZlibBlockDescriptor {
    /// Offset.
    pub offset: u32,

    /// Size.
    pub size: u32,
}

impl DecmpfsZlibBlockDescriptor {
    /// Creates a new block descriptor.
    pub fn new() -> Self {
        Self { offset: 0, size: 0 }
    }

    /// Reads the fork descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.offset = bytes_to_u32_le!(data, 0);
        self.size = bytes_to_u32_le!(data, 4);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = DecmpfsZlibBlockDescriptor::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = DecmpfsZlibBlockDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
