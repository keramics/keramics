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
use keramics_types::bytes_to_u64_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "inode_number", data_type = "u64"),
        field(name = "name_size", data_type = "u8"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) directory list element entry version 2.
pub struct XfsDirectoryListElementEntryV2 {
    /// Inode number.
    pub inode_number: u64,

    /// Name size.
    pub name_size: u8,
}

impl XfsDirectoryListElementEntryV2 {
    /// Creates a new entry.
    pub fn new() -> Self {
        Self {
            inode_number: 0,
            name_size: 0,
        }
    }

    /// Reads the entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 9 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.inode_number = bytes_to_u64_be!(data, 0);
        self.name_size = data[8];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x44, 0x01, 0x2e, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x40,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryListElementEntryV2::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.inode_number, 11076);
        assert_eq!(test_struct.name_size, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryListElementEntryV2::new();
        let result = test_struct.read_data(&test_data[0..8]);
        assert!(result.is_err());
    }
}
