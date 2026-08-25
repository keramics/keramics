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
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "size", data_type = "u32"),
        field(name = "physical_address", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) object map value.
pub struct ApfsObjectMapValue {
    /// Flags.
    pub flags: u32,

    /// Size.
    pub size: u32,

    /// Physical address.
    pub physical_address: u64,
}

impl ApfsObjectMapValue {
    /// Creates a new value.
    pub fn new() -> Self {
        Self {
            flags: 0,
            size: 0,
            physical_address: 0,
        }
    }

    /// Reads the value from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.flags = bytes_to_u32_le!(data, 0);
        self.size = bytes_to_u32_le!(data, 4);
        self.physical_address = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x93, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsObjectMapValue::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x00000000);
        assert_eq!(test_struct.size, 4096);
        assert_eq!(test_struct.physical_address, 147);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsObjectMapValue::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
