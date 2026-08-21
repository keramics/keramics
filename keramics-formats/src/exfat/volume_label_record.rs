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
use keramics_types::Ucs2String;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_code", data_type = "u8", format = "hex"),
        field(name = "name_size", data_type = "u8"),
        field(name = "name", data_type = "Ucs2String<22>"),
        field(name = "unknown1", data_type = "[u8; 8]"),
    ),
    methods("debug_read_data")
)]
/// Extensible File Allocation Table (exFAT) volume label (directory) record.
pub struct ExFatVolumeLabelRecord {
    /// Name.
    pub name: Ucs2String,
}

impl ExFatVolumeLabelRecord {
    /// Creates a new volume label record.
    pub fn new() -> Self {
        Self {
            name: Ucs2String::new(),
        }
    }

    /// Reads the volume label record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let type_code: u8 = data[0];

        if type_code != 0x83 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported type code: 0x{:02x}",
                type_code
            )));
        }
        let data_end_offset: usize = 2 + ((data[1] as usize) * 2);

        if data_end_offset > 24 {
            return Err(keramics_core::error_trace_new!(
                "Invalid name size value out of bounds"
            ));
        }
        self.name.read_data_le(&data[2..data_end_offset]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x83, 0x0a, 0x65, 0x00, 0x78, 0x00, 0x66, 0x00, 0x61, 0x00, 0x74, 0x00, 0x5f, 0x00,
            0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatVolumeLabelRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name, Ucs2String::from("exfat_test"));

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatVolumeLabelRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_type_code() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExFatVolumeLabelRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_name_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[1] = 0xff;

        let mut test_struct = ExFatVolumeLabelRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
