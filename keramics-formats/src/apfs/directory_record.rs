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
use keramics_types::bytes_to_u64_le;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "object_identifier", data_type = "u64"),
        field(name = "added_time", data_type = "ApfsTime"),
        field(name = "directory_entry_flags", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) directory record.
pub struct ApfsDirectoryRecord {
    /// Object identifier.
    pub object_identifier: u64,
}

impl ApfsDirectoryRecord {
    /// Creates a new directory record.
    pub fn new() -> Self {
        Self {
            object_identifier: 0,
        }
    }

    /// Reads the directory record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 18 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.object_identifier = bytes_to_u64_le!(data, 0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x87, 0xe6, 0x41, 0xaf, 0x6a, 0xdd,
            0x59, 0x15, 0x04, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsDirectoryRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.object_identifier, 18);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsDirectoryRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..17]);
        assert!(result.is_err());
    }
}
