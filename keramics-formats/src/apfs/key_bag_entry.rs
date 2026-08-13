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
use keramics_types::{Uuid, bytes_to_u16_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "identifier", data_type = "Uuid"),
        field(name = "entry_type", data_type = "u16"),
        field(name = "data_size", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 4]"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) key bag entry.
pub struct ApfsKeyBagEntry {
    /// Identifier.
    pub identifier: Uuid,

    /// Number of entries.
    pub entry_type: u16,

    /// Data size.
    pub data_size: u16,

    /// Data.
    pub data: Vec<u8>,
}

impl ApfsKeyBagEntry {
    /// Creates a new key bag entry.
    pub fn new() -> Self {
        Self {
            identifier: Uuid::new(),
            entry_type: 0,
            data_size: 0,
            data: Vec::new(),
        }
    }

    /// Reads the key bag entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 24 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.identifier = Uuid::from_be_bytes(&data[0..16]);
        self.entry_type = bytes_to_u16_le!(data, 16);
        self.data_size = bytes_to_u16_le!(data, 18);

        if (self.data_size as usize) > data_size - 24 {
            return Err(keramics_core::error_trace_new!(
                "Invalid entry data size value out of bounds"
            ));
        }
        let data_end_offset: usize = 24 + (self.data_size as usize);

        self.data = data[24..data_end_offset].to_vec();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x41, 0xd2, 0x38, 0x68, 0x9d, 0x22, 0x49, 0x40, 0xbb, 0xf9, 0x9e, 0x1a, 0xfe, 0xb6,
            0xc9, 0x96, 0x03, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x56, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsKeyBagEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.identifier.to_string(),
            "41d23868-9d22-4940-bbf9-9e1afeb6c996"
        );
        assert_eq!(test_struct.entry_type, 3);
        assert_eq!(test_struct.data_size, 16);
        assert_eq!(test_struct.data, &test_data[24..40]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsKeyBagEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_entry_data_size() {
        let mut test_struct = ApfsKeyBagEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..39]);
        assert!(result.is_err());
    }
}
