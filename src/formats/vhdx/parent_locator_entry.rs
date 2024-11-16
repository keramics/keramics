/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "key_data_offset", data_type = "u32", format = "hex"),
        field(name = "value_data_offset", data_type = "u32", format = "hex"),
        field(name = "key_data_size", data_type = "u16"),
        field(name = "value_data_size", data_type = "u16"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) parent locator entry.
pub struct VhdxParentLocatorEntry {
    /// Key data offset.
    pub key_data_offset: u32,

    /// Value data offset.
    pub value_data_offset: u32,

    /// Key data size.
    pub key_data_size: u16,

    /// Value data size.
    pub value_data_size: u16,
}

impl VhdxParentLocatorEntry {
    /// Creates a new parent locator entry.
    pub fn new() -> Self {
        Self {
            key_data_offset: 0,
            value_data_offset: 0,
            key_data_size: 0,
            value_data_size: 0,
        }
    }

    /// Reads the parent locator entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.key_data_offset = crate::bytes_to_u32_le!(data, 0);
        self.value_data_offset = crate::bytes_to_u32_le!(data, 4);
        self.key_data_size = crate::bytes_to_u16_le!(data, 8);
        self.value_data_size = crate::bytes_to_u16_le!(data, 10);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x50, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x4c, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data = get_test_data();

        let mut test_struct = VhdxParentLocatorEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.key_data_offset, 80);
        assert_eq!(test_struct.value_data_offset, 108);
        assert_eq!(test_struct.key_data_size, 28);
        assert_eq!(test_struct.value_data_size, 76);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxParentLocatorEntry::new();

        let test_data = get_test_data();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }
}
