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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "object_type", data_type = "u32", format = "hex"),
        field(name = "object_subtype", data_type = "u32", format = "hex"),
        field(name = "size", data_type = "u32"),
        field(name = "unknown1", data_type = "u32"),
        field(name = "file_system_object_identifier", data_type = "u64"),
        field(name = "object_identifier", data_type = "u64"),
        field(name = "physical_address", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) checkpoint map entry.
pub struct ApfsCheckpointMapEntry {
    /// Object type.
    pub object_type: u32,

    /// Object subtype.
    pub object_subtype: u32,
}

impl ApfsCheckpointMapEntry {
    /// Creates a new checkpoint map entry.
    pub fn new() -> Self {
        Self {
            object_type: 0,
            object_subtype: 0,
        }
    }

    /// Reads the checkpoint map entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 40 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.object_type = bytes_to_u32_le!(data, 0);
        self.object_subtype = bytes_to_u32_le!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x05, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x17, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsCheckpointMapEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.object_type, 0x80000005);
        assert_eq!(test_struct.object_subtype, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsCheckpointMapEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..39]);
        assert!(result.is_err());
    }
}
