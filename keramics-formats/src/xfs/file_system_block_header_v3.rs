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
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "block_number", data_type = "u64"),
        field(name = "log_sequence_number", data_type = "u64"),
        field(name = "block_type_identifier", data_type = "Uuid"),
        field(name = "owner_inode_number", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) file system block header version 1.
pub struct XfsFileSystemBlockHeaderV3 {}

impl XfsFileSystemBlockHeaderV3 {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 56 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[8..10] != &[0x3b, 0xee] {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0xee, 0x00, 0x00, 0x83, 0x64,
            0x08, 0x8a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x38, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x02, 0xeb, 0xd6, 0x54, 0x96, 0xec, 0xd8, 0x49, 0x90, 0x95, 0x48,
            0x47, 0x85, 0x39, 0x5a, 0x1b, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x4f,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsFileSystemBlockHeaderV3::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsFileSystemBlockHeaderV3::new();
        let result = test_struct.read_data(&test_data[0..55]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[8] = 0xff;

        let mut test_struct = XfsFileSystemBlockHeaderV3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
