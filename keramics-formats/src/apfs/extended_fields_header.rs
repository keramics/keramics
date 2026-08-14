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
        field(name = "number_of_fields", data_type = "u16"),
        field(name = "data_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) extended fields header.
pub struct ApfsExtendedFieldsHeader {
    /// Number of fields.
    pub number_of_fields: u16,

    /// Data size.
    pub data_size: u16,
}

impl ApfsExtendedFieldsHeader {
    /// Creates a new extended fields header.
    pub fn new() -> Self {
        Self {
            number_of_fields: 0,
            data_size: 0,
        }
    }

    /// Reads the extended fields header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.number_of_fields = bytes_to_u16_le!(data, 0);
        self.data_size = bytes_to_u16_le!(data, 2);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x01, 0x00, 0x08, 0x00];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsExtendedFieldsHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_fields, 1);
        assert_eq!(test_struct.data_size, 8);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsExtendedFieldsHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
