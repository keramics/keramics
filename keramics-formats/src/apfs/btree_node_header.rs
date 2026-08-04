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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "flags", data_type = "u16", format = "hex"),
        field(name = "level", data_type = "u16"),
        field(name = "number_of_keys", data_type = "u32"),
        field(name = "entries_data_offset", data_type = "u16", format = "hex"),
        field(name = "entries_data_size", data_type = "u16"),
        field(name = "unused_data_offset", data_type = "u16", format = "hex"),
        field(name = "unused_data_size", data_type = "u16"),
        field(name = "key_free_list_offset", data_type = "u16", format = "hex"),
        field(name = "key_free_list_size", data_type = "u16"),
        field(name = "value_free_list_offset", data_type = "u16", format = "hex"),
        field(name = "value_free_list_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) B-Tree node header.
pub struct ApfsBtreeNodeHeader {
    /// Flags.
    pub flags: u16,

    /// Level.
    pub level: u16,

    /// Number of keys.
    pub number_of_keys: u32,

    /// Entries data offset.
    pub entries_data_offset: u16,

    /// Entries data size.
    pub entries_data_size: u16,
}

impl ApfsBtreeNodeHeader {
    /// Creates a new B-Tree node header.
    pub fn new() -> Self {
        Self {
            flags: 0,
            level: 0,
            number_of_keys: 0,
            entries_data_offset: 0,
            entries_data_size: 0,
        }
    }

    /// Reads the B-Tree node header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 24 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.flags = bytes_to_u16_le!(data, 0);
        self.level = bytes_to_u16_le!(data, 2);
        self.number_of_keys = bytes_to_u32_le!(data, 4);
        self.entries_data_offset = bytes_to_u16_le!(data, 8);
        self.entries_data_size = bytes_to_u16_le!(data, 10);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x07, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x01, 0x20, 0x00,
            0xa0, 0x0d, 0x10, 0x00, 0x10, 0x00, 0x20, 0x00, 0x10, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsBtreeNodeHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x0007);
        assert_eq!(test_struct.level, 0);
        assert_eq!(test_struct.number_of_keys, 1);
        assert_eq!(test_struct.entries_data_offset, 0x0000);
        assert_eq!(test_struct.entries_data_size, 448);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsBtreeNodeHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }
}
