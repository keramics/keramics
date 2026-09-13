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
        field(name = "tag", data_type = "u8", format = "hex"),
        field(name = "data_size_with_flag", data_type = "u8", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) key bag packed value.
pub struct ApfsKeyBagPackedValue {
    /// Tag.
    pub tag: u8,

    /// Extended size.
    pub extended_size: u8,

    /// Data size.
    pub data_size: u16,
}

impl ApfsKeyBagPackedValue {
    /// Creates a new key bag packed value.
    pub fn new() -> Self {
        Self {
            tag: 0,
            extended_size: 0,
            data_size: 0,
        }
    }

    /// Reads the key bag packed value from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 2 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.tag = data[0];

        self.extended_size = if data[1] & 0x80 == 0 {
            0
        } else {
            data[1] & 0x7f
        };
        if (self.extended_size as usize) > data_size - 2 {
            return Err(keramics_core::error_trace_new!(
                "Invalid extended size value out of bounds"
            ));
        }
        self.data_size = match self.extended_size {
            0 => data[1] as u16,
            1 => data[2] as u16,
            2 => bytes_to_u16_le!(data, 2),
            _ => return Err(keramics_core::error_trace_new!("Unsupported extended size")),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![0x80, 0x01, 0x00]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsKeyBagPackedValue::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.tag, 0x80);
        assert_eq!(test_struct.extended_size, 0);
        assert_eq!(test_struct.data_size, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsKeyBagPackedValue::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..1]);
        assert!(result.is_err());
    }
}
