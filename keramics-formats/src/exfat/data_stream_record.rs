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
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_code", data_type = "u8", format = "hex"),
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "unknown1", data_type = "u8"),
        field(name = "name_size", data_type = "u8"),
        field(name = "name_hash", data_type = "u16", format = "hex"),
        field(name = "unknown2", data_type = "u16"),
        field(name = "valid_data_size", data_type = "u64"),
        field(name = "unknown3", data_type = "u32"),
        field(name = "data_start_cluster", data_type = "u32"),
        field(name = "data_size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Extensible File Allocation Table (exFAT) data stream (directory) record.
pub struct ExFatDataStreamRecord {
    /// Flags.
    pub flags: u8,

    /// Valid data size.
    pub valid_data_size: u64,

    /// Data start cluster.
    pub data_start_cluster: u32,

    /// Data size.
    pub data_size: u64,
}

impl ExFatDataStreamRecord {
    /// Creates a new data stream record.
    pub fn new() -> Self {
        Self {
            flags: 0,
            valid_data_size: 0,
            data_start_cluster: 0,
            data_size: 0,
        }
    }

    /// Reads the data stream record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let type_code: u8 = data[0];

        if type_code != 0xc0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported type code: 0x{:02x}",
                type_code
            )));
        }
        self.flags = data[1];
        self.valid_data_size = bytes_to_u64_le!(data, 8);
        self.data_start_cluster = bytes_to_u32_le!(data, 20);
        self.data_size = bytes_to_u64_le!(data, 24);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xc0, 0x01, 0x00, 0x09, 0x6b, 0x8e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatDataStreamRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x01);
        assert_eq!(test_struct.valid_data_size, 0);
        assert_eq!(test_struct.data_start_cluster, 0);
        assert_eq!(test_struct.data_size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatDataStreamRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_type_code() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExFatDataStreamRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
