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

use crate::bytes_to_u16_le;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_identifier", data_type = "Uuid"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "number_of_entries", data_type = "u16"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) parent locator header.
pub struct VhdxParentLocatorHeader {
    /// Number of entries.
    pub number_of_entries: u16,
}

impl VhdxParentLocatorHeader {
    /// Creates a new parent locator header.
    pub fn new() -> Self {
        Self {
            number_of_entries: 0,
        }
    }

    /// Reads the parent locator header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..16] != VHDX_PARENT_LOCATOR_TYPE_INDICATOR {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported type indicator"),
            ));
        }
        self.number_of_entries = bytes_to_u16_le!(data, 18);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xb7, 0xef, 0x4a, 0xb0, 0x9e, 0xd1, 0x81, 0x4a, 0xb7, 0x89, 0x25, 0xb8, 0xe9, 0x44,
            0x59, 0x13, 0x00, 0x00, 0x05, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxParentLocatorHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_entries, 5);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxParentLocatorHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..19]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_type_indicator() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VhdxParentLocatorHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
