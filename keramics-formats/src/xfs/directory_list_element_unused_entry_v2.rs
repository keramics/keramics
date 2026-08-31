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
        field(name = "free_tag", data_type = "u16", format = "hex"),
        field(name = "entry_size", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 2]"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) directory list element unused entry version 2.
pub struct XfsDirectoryListElementUnusedEntryV2 {
    /// Entry size.
    pub entry_size: u16,
}

impl XfsDirectoryListElementUnusedEntryV2 {
    /// Creates a new entry.
    pub fn new() -> Self {
        Self { entry_size: 0 }
    }

    /// Reads the entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 6 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != &[0xff, 0xff] {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.entry_size = bytes_to_u16_be!(data, 2);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xff, 0xff, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryListElementUnusedEntryV2::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entry_size, 16);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryListElementUnusedEntryV2::new();
        let result = test_struct.read_data(&test_data[0..5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0x00;

        let mut test_struct = XfsDirectoryListElementUnusedEntryV2::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
