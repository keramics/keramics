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
        field(name = "record_type", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "data_size", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) attribute inline data record.
pub struct HfsAttributeInlineDataRecord {
    /// Record type.
    pub record_type: u32,

    /// Data size.
    pub data_size: u32,

    /// Data.
    pub data: Vec<u8>,
}

impl HfsAttributeInlineDataRecord {
    /// Creates a new attribute record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            data_size: 0,
            data: Vec::new(),
        }
    }

    /// Reads the attribute record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.record_type = bytes_to_u32_be!(data, 0);

        if self.record_type != 0x00000010 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:08x}",
                self.record_type
            )));
        }
        self.data_size = bytes_to_u32_be!(data, 12);

        let data_end_offset: usize = 16 + (self.data_size as usize);

        if data_end_offset > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid attribute data size: {} value out of bounds",
                self.data_size
            )));
        }
        self.data = data[16..data_end_offset].to_vec();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0xff, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsAttributeInlineDataRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.record_type, 0x00000010);
        assert_eq!(test_struct.data_size, 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsAttributeInlineDataRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
