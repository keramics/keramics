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
use keramics_types::bytes_to_u16_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "name_hash", data_type = "u32", format = "hex"),
        field(name = "values_offset", data_type = "u16"),
        field(name = "name_size", data_type = "u8"),
        field(name = "unknonw1", data_type = "u8"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) directory tree leaf entry.
pub struct XfsDirectoryTreeLeafEntry {
    /// Values offset.
    pub values_offset: u16,

    /// Name size.
    pub name_size: u8,
}

impl XfsDirectoryTreeLeafEntry {
    /// Creates a new entry.
    pub fn new() -> Self {
        Self {
            values_offset: 0,
            name_size: 0,
        }
    }

    /// Reads the entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.values_offset = bytes_to_u16_be!(data, 4);
        self.name_size = data[6];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x12, 0x34, 0x56, 0x78, 0x00, 0x10, 0x06, 0x00]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryTreeLeafEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.values_offset, 16);
        assert_eq!(test_struct.name_size, 6);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryTreeLeafEntry::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
