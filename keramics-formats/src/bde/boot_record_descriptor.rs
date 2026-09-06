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

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "boot_record_offset", data_type = "u64", format = "hex"),
        field(name = "boot_record_size", data_type = "u64"),
        group(
            size_condition = ">= 76",
            field(name = "unknown1", data_type = "u16"),
            field(name = "additional_data_size", data_type = "u16"),
            field(name = "unknown2", data_type = "u32"),
            field(name = "unknown3", data_type = "u64", format = "hex"),
            field(name = "convert_log_offset", data_type = "u64", format = "hex"),
            field(name = "convert_log_size", data_type = "u32"),
            field(name = "unknown4", data_type = "u32"),
            field(name = "unknown5", data_type = "u32"),
            field(name = "unknown6", data_type = "u64"),
            field(name = "unknown7", data_type = "u32", format = "hex"),
            field(name = "unknown8", data_type = "u64"),
            field(name = "unknown9", data_type = "u32", format = "hex"),
        ),
        group(
            size_condition = ">= 92",
            field(name = "unknown_offset", data_type = "u64", format = "hex"),
            field(name = "unknown_size", data_type = "u32"),
            field(name = "unknown10", data_type = "u32"),
        )
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) metadata header.
pub struct BdeBootRecordDescriptor {
    /// Boot record offset.
    pub boot_record_offset: u64,

    /// Boot record size.
    pub boot_record_size: u64,
}

impl BdeBootRecordDescriptor {
    /// Creates a new metadata header.
    pub fn new() -> Self {
        Self {
            boot_record_offset: 0,
            boot_record_size: 0,
        }
    }

    /// Reads the metadata header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.boot_record_offset = bytes_to_u64_le!(data, 0);
        self.boot_record_size = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x05, 0x00, 0x4c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x65, 0x4a, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x10, 0x01, 0x01, 0x02, 0x00, 0x20, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecordDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.boot_record_offset, 0x02200000);
        assert_eq!(test_struct.boot_record_size, 8192);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecordDescriptor::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
