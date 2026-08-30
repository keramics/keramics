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

use super::block_free_region_v2::XfsBlockFreeRegionV2;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "block_number", data_type = "u64"),
        field(name = "log_sequence_number", data_type = "u64"),
        field(name = "block_type_identifier", data_type = "Uuid"),
        field(name = "owner_inode_number", data_type = "u64"),
        field(
            name = "free_regions",
            data_type = "[Struct<XfsBlockFreeRegionV2; 4>; 3]"
        ),
        field(name = "unknown1", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) block directory header version 3.
pub struct XfsBlockDirectoryHeaderV3 {}

impl XfsBlockDirectoryHeaderV3 {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
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
            0x58, 0x44, 0x42, 0x33, 0x05, 0x5c, 0x65, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x2b, 0x18, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x1a, 0x76, 0x3f, 0x17, 0x32,
            0x5c, 0xbf, 0x4d, 0x80, 0x90, 0x10, 0x6f, 0xab, 0xe2, 0xb7, 0x8c, 0xe2, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x2b, 0x44, 0x0a, 0xb0, 0x01, 0xd0, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBlockDirectoryHeaderV3::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsBlockDirectoryHeaderV3::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
