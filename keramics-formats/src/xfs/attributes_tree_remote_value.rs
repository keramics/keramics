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
use keramics_types::bytes_to_u32_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "value_block_number", data_type = "u32"),
        field(name = "value_data_size", data_type = "u32"),
        field(name = "name_size", data_type = "u8"),
    ),
    methods("debug_read_data")
)]
/// X File System (XFS) attributes tree local value.
pub struct XfsAttributesTreeRemoteValue {
    /// Value block number.
    pub value_block_number: u32,

    /// Value data size.
    pub value_data_size: u32,

    /// name size.
    pub name_size: u8,
}

impl XfsAttributesTreeRemoteValue {
    /// Creates a new entry.
    pub fn new() -> Self {
        Self {
            value_block_number: 0,
            value_data_size: 0,
            name_size: 0,
        }
    }

    /// Reads the entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 9 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.value_block_number = bytes_to_u32_be!(data, 0);
        self.value_data_size = bytes_to_u32_be!(data, 4);
        self.name_size = data[8];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x10, 0x06, 0x00]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsAttributesTreeRemoteValue::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.value_block_number, 32);
        assert_eq!(test_struct.value_data_size, 16);
        assert_eq!(test_struct.name_size, 6);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsAttributesTreeRemoteValue::new();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
