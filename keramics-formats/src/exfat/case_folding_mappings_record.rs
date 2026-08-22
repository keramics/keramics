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
        byte_order = "little",
        field(name = "type_code", data_type = "u8", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 3]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "unknown2", data_type = "[u8; 12]"),
        field(name = "data_start_cluster", data_type = "u32"),
        field(name = "data_size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Extensible File Allocation Table (exFAT) case folding mappings (directory) record.
pub struct ExFatCaseFoldingMappingsRecord {}

impl ExFatCaseFoldingMappingsRecord {
    /// Creates a new case folding mappings record.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the case folding mappings record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let type_code: u8 = data[0];

        if type_code != 0x82 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported type code: 0x{:02x}",
                type_code
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x82, 0x00, 0x00, 0x00, 0x0d, 0xd3, 0x19, 0xe6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0xcc, 0x16, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatCaseFoldingMappingsRecord::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatCaseFoldingMappingsRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_type_code() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExFatCaseFoldingMappingsRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
