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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "entry_size", data_type = "u16"),
        field(name = "entry_type", data_type = "u16", format = "hex"),
        field(name = "value_type", data_type = "u16", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 2]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker Drive Encryption (BDE) metadata entry header.
pub struct BdeMetadataEntryHeader {
    /// Entry size.
    pub entry_size: u16,

    /// Entry type.
    pub entry_type: u16,

    /// Value type.
    pub value_type: u16,
}

impl BdeMetadataEntryHeader {
    /// Creates a new metadata entry header.
    pub fn new() -> Self {
        Self {
            entry_size: 0,
            entry_type: 0,
            value_type: 0,
        }
    }

    /// Reads the metadata entry header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.entry_size = bytes_to_u16_le!(data, 0);
        self.entry_type = bytes_to_u16_le!(data, 2);
        self.value_type = bytes_to_u16_le!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x6c, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataEntryHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entry_size, 108);
        assert_eq!(test_struct.entry_type, 0);
        assert_eq!(test_struct.value_type, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataEntryHeader::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
