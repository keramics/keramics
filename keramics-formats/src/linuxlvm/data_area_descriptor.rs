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

/// Linux Logical Volume Manager (LVM) data area descriptor.
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "offset", data_type = "u64", format = "hex"),
        field(name = "size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
pub struct LinuxLvmDataAreaDescriptor {
    /// Logical offset.
    pub logical_offset: u64,

    /// Physical offset.
    pub physical_offset: u64,

    /// Size.
    pub size: u64,
}

impl LinuxLvmDataAreaDescriptor {
    /// Creates a new data area descriptor.
    pub fn new() -> Self {
        Self {
            logical_offset: 0,
            physical_offset: 0,
            size: 0,
        }
    }

    /// Reads the data area descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.physical_offset = bytes_to_u64_le!(data, 0);
        self.size = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LinuxLvmDataAreaDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.physical_offset, 0x00100000);
        assert_eq!(test_struct.size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = LinuxLvmDataAreaDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
