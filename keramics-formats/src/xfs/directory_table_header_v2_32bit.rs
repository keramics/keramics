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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "number_of_entries_32bit", data_type = "u8"),
        field(name = "number_of_entries_64bit", data_type = "u8"),
        field(name = "parent_inode_number", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) directory table header version 2 32-bit.
pub struct XfsDirectoryTableHeader32bitV2 {
    /// Parent inode number.
    pub parent_inode_number: u32,
}

impl XfsDirectoryTableHeader32bitV2 {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            parent_inode_number: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 6 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.parent_inode_number = bytes_to_u32_be!(data, 2);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x01]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryTableHeader32bitV2::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.parent_inode_number, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryTableHeader32bitV2::new();
        let result = test_struct.read_data(&test_data[0..5]);
        assert!(result.is_err());
    }
}
