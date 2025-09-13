/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;

use keramics_types::{bytes_to_u32_le, bytes_to_u64_le, Uuid};
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_identifier", data_type = "Uuid"),
        field(name = "data_offset", data_type = "u64", format = "hex"),
        field(name = "data_size", data_type = "u32"),
        field(name = "is_required_flag", data_type = "u32"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) region table entry.
pub struct VhdxRegionTableEntry {
    /// Type identifier.
    pub type_identifier: Uuid,

    /// Data offset.
    pub data_offset: u64,

    /// Data size.
    pub data_size: u32,

    /// Value to indicate the region type needs to be supported.
    pub is_required: bool,
}

impl VhdxRegionTableEntry {
    /// Creates a new region table entry.
    pub fn new() -> Self {
        Self {
            type_identifier: Uuid::new(),
            data_offset: 0,
            data_size: 0,
            is_required: false,
        }
    }

    /// Reads the region table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        let is_required_flag: u32 = bytes_to_u32_le!(data, 28);

        self.type_identifier = Uuid::from_le_bytes(&data[0..16]);
        self.data_offset = bytes_to_u64_le!(data, 16);
        self.data_size = bytes_to_u32_le!(data, 24);
        self.is_required = if is_required_flag & 0x00000001 == 0 {
            false
        } else {
            true
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x66, 0x77, 0xc2, 0x2d, 0x23, 0xf6, 0x00, 0x42, 0x9d, 0x64, 0x11, 0x5e, 0x9b, 0xfd,
            0x4a, 0x08, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00,
            0x01, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxRegionTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.type_identifier.to_string(),
            "2dc27766-f623-4200-9d64-115e9bfd4a08"
        );
        assert_eq!(test_struct.data_offset, 3145728);
        assert_eq!(test_struct.data_size, 1048576);
        assert_eq!(test_struct.is_required, true);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxRegionTableEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }
}
