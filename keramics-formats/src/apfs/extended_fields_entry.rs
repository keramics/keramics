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
use keramics_types::bytes_to_u16_le;

#[derive(Debug, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "field_type", data_type = "u8"),
        field(name = "flags", data_type = "u8"),
        field(name = "data_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) extended fields entry.
pub struct ApfsExtendedFieldsEntry {
    /// Field type.
    pub field_type: u8,

    /// Flags.
    pub flags: u8,

    /// Data size.
    pub data_size: u16,
}

impl ApfsExtendedFieldsEntry {
    /// Creates a new extended fields entry.
    pub fn new() -> Self {
        Self {
            field_type: 0,
            flags: 0,
            data_size: 0,
        }
    }

    /// Reads the extended fields entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.field_type = data[0];
        self.flags = data[1];
        self.data_size = bytes_to_u16_le!(data, 2);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x04, 0x02, 0x05, 0x00, 0x72, 0x6f, 0x6f, 0x74];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsExtendedFieldsEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.field_type, 4);
        assert_eq!(test_struct.flags, 2);
        assert_eq!(test_struct.data_size, 5);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsExtendedFieldsEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
