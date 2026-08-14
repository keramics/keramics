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
use keramics_types::bytes_to_u64_le;

#[derive(Debug, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "used_size", data_type = "u64"),
        field(name = "allocated_size", data_type = "u64"),
        field(name = "encryption_identifier", data_type = "u64"),
        field(name = "total_bytes_written", data_type = "u64"),
        field(name = "total_bytes_read", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) data stream descriptor.
pub struct ApfsDataStreamDescriptor {
    /// Size.
    pub size: u64,

    /// Encryption identifier.
    pub encryption_identifier: u64,
}

impl ApfsDataStreamDescriptor {
    /// Creates a new data stream descriptor.
    pub fn new() -> Self {
        Self {
            size: 0,
            encryption_identifier: 0,
        }
    }

    /// Reads the data stream descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 40 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.size = bytes_to_u64_le!(data, 0);
        self.encryption_identifier = bytes_to_u64_le!(data, 16);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x53, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsDataStreamDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.size, 83);
        assert_eq!(test_struct.encryption_identifier, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsDataStreamDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..39]);
        assert!(result.is_err());
    }
}
