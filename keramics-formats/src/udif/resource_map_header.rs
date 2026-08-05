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
        field(name = "unknown1", data_type = "[u8; 16]"),
        field(name = "unknown2", data_type = "u32"),
        field(name = "unknown3", data_type = "u16"),
        field(name = "unknown4", data_type = "u16", format = "hex"),
        field(name = "entries_list_offset", data_type = "u16"),
        field(name = "names_list_offset", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Universal Disk Image Format (UDIF) resource map header.
pub struct UdifResourceMapHeader {
    /// Entries list offset.
    pub entries_list_offset: u16,

    /// Names list offset.
    pub names_list_offset: u16,
}

impl UdifResourceMapHeader {
    /// Creates a new resource map header.
    pub fn new() -> Self {
        Self {
            entries_list_offset: 0,
            names_list_offset: 0,
        }
    }

    /// Reads the resource map header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 28 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.entries_list_offset = bytes_to_u16_be!(data, 24);
        self.names_list_offset = bytes_to_u16_be!(data, 26);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x2c, 0x00, 0x00, 0x09, 0x2c, 0x00, 0x00,
            0x00, 0xd7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x6a,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceMapHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entries_list_offset, 28);
        assert_eq!(test_struct.names_list_offset, 106);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceMapHeader::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
