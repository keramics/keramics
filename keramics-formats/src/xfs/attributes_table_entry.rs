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
        field(name = "name_size", data_type = "u8"),
        field(name = "value_size", data_type = "u8"),
        field(name = "flags", data_type = "u8"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) attributes table entry.
pub struct XfsAttributesTableEntry {
    /// Name size.
    pub name_size: u8,

    /// Value size.
    pub value_size: u8,

    /// Flags.
    pub flags: u8,
}

impl XfsAttributesTableEntry {
    /// Creates a new entry.
    pub fn new() -> Self {
        Self {
            name_size: 0,
            value_size: 0,
            flags: 0,
        }
    }

    /// Reads the entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 3 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.name_size = data[0];
        self.value_size = data[1];
        self.flags = data[2];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x06, 0x0c, 0x08, 0x78, 0x61, 0x74, 0x74, 0x72, 0x31]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsAttributesTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name_size, 6);
        assert_eq!(test_struct.value_size, 12);
        assert_eq!(test_struct.flags, 0x08);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsAttributesTableEntry::new();
        let result = test_struct.read_data(&test_data[0..2]);
        assert!(result.is_err());
    }
}
