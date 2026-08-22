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
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "name", data_type = "Ucs2String<30>"),
    ),
    methods("debug_read_data")
)]
/// Extensible File Allocation Table (exFAT) file name (directory) record.
pub struct ExFatFileNameRecord {
    /// Name.
    pub name: Ucs2String,
}

impl ExFatFileNameRecord {
    /// Creates a new file name record.
    pub fn new() -> Self {
        Self {
            name: Ucs2String::new(),
        }
    }

    /// Reads the file name record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let type_code: u8 = data[0];

        if type_code != 0xc1 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported type code: 0x{:02x}",
                type_code
            )));
        }
        self.name.read_data_le(&data[2..32]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xc1, 0x00, 0x65, 0x00, 0x6d, 0x00, 0x70, 0x00, 0x74, 0x00, 0x79, 0x00, 0x66, 0x00,
            0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatFileNameRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name, Ucs2String::from("emptyfile"));

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatFileNameRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_type_code() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExFatFileNameRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
