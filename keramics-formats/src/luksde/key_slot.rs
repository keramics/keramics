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
        field(name = "state", data_type = "u32", format = "hex"),
        field(name = "number_of_iterations", data_type = "u32"),
        field(name = "salt", data_type = "[u8; 32]", format = "hex"),
        field(name = "key_material_start_sector", data_type = "u32"),
        field(name = "number_of_stripes", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// Linux Unified Key Setup (LUKS) Disk Encryption key slot.
pub struct LuksKeySlot {
    /// State.
    pub state: u32,

    /// Number of iterations.
    pub number_of_iterations: u32,

    /// Salt.
    pub salt: Vec<u8>,

    /// Key material start sector.
    pub key_material_start_sector: u32,

    /// Number of stripes.
    pub number_of_stripes: u32,
}

impl LuksKeySlot {
    /// Creates a new key slot.
    pub fn new() -> Self {
        Self {
            state: 0,
            number_of_iterations: 0,
            salt: Vec::new(),
            key_material_start_sector: 0,
            number_of_stripes: 0,
        }
    }

    /// Reads the key slot from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 48 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.state = bytes_to_u32_be!(data, 0);
        self.number_of_iterations = bytes_to_u32_be!(data, 4);
        self.salt = data[8..40].to_vec();
        self.key_material_start_sector = bytes_to_u32_be!(data, 40);
        self.number_of_stripes = bytes_to_u32_be!(data, 44);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0xac, 0x71, 0xf3, 0x00, 0xb2, 0x85, 0x88, 0xc9, 0x51, 0xd8, 0xac, 0x2d, 0x9e,
            0x71, 0x34, 0x7a, 0x7d, 0x42, 0x49, 0xba, 0x23, 0x85, 0x7d, 0x8b, 0x41, 0x7e, 0xe7,
            0x2c, 0xe6, 0xab, 0x2a, 0xb2, 0xa3, 0x9d, 0x90, 0x4d, 0x48, 0x68, 0xa0, 0x00, 0x00,
            0x00, 0x08, 0x00, 0x00, 0x0f, 0xa0,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LuksKeySlot::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.state, 0x00ac71f3);
        assert_eq!(test_struct.number_of_iterations, 11699592);
        assert_eq!(test_struct.salt, &test_data[8..40]);
        assert_eq!(test_struct.key_material_start_sector, 8);
        assert_eq!(test_struct.number_of_stripes, 4000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LuksKeySlot::new();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }
}
