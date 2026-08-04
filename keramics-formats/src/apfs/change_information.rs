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
        field(name = "application", data_type = "ByteString<32>"),
        field(name = "change_time", data_type = "ApfsTime"),
        field(name = "change_transaction_identifier", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) change information.
pub struct ApfsChangeInformation {}

impl ApfsChangeInformation {
    /// Creates a new change information.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the change information from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 48 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x6e, 0x65, 0x77, 0x66, 0x73, 0x5f, 0x61, 0x70, 0x66, 0x73, 0x20, 0x28, 0x32, 0x38,
            0x31, 0x31, 0x2e, 0x31, 0x32, 0x31, 0x2e, 0x31, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x65, 0x28, 0xaf, 0xc8, 0x01, 0xfe, 0xc7, 0x18, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsChangeInformation::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsChangeInformation::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }
}
