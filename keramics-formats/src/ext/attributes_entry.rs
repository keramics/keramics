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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{ByteString, bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "name_size", data_type = "u8"),
        field(name = "name_index", data_type = "u8"),
        field(name = "value_data_offset", data_type = "u16"),
        field(name = "value_data_inode_number", data_type = "u32"),
        field(name = "value_data_size", data_type = "u32"),
        field(name = "attribute_hash", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext) attributes entry.
pub struct ExtAttributesEntry {
    /// Name size.
    pub name_size: u8,

    /// Name index.
    pub name_index: u8,

    /// Value data offset.
    pub value_data_offset: u16,

    /// Value data inode number.
    pub value_data_inode_number: u32,

    /// Value data size.
    pub value_data_size: u32,
}

impl ExtAttributesEntry {
    /// Creates a new attributes entry.
    pub fn new() -> Self {
        Self {
            name_size: 0,
            name_index: 0,
            value_data_offset: 0,
            value_data_inode_number: 0,
            value_data_size: 0,
        }
    }

    /// Reads the attributes entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported ext attributes entry data size"
            ));
        }
        self.name_size = data[0];
        self.name_index = data[1];
        self.value_data_offset = bytes_to_u16_le!(data, 2);
        self.value_data_inode_number = bytes_to_u32_le!(data, 4);
        self.value_data_size = bytes_to_u32_le!(data, 8);

        Ok(())
    }

    /// Reads the attributes entry name from a buffer.
    pub fn read_name(&self, data: &[u8]) -> Result<ByteString, ErrorTrace> {
        let data_end_offset: usize = self.name_size as usize;

        if data.len() < data_end_offset {
            return Err(keramics_core::error_trace_new!(
                "Unsupported ext attributes entry name size"
            ));
        }
        let name_prefix: &str = match self.name_index {
            0 => "",
            1 => "user.",
            2 => "system.posix_acl_access",
            3 => "system.posix_acl_default",
            4 => "trusted.",
            6 => "security.",
            7 => "system.",
            8 => "system.richacl",
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported attribute name index: {}",
                    self.name_index
                )));
            }
        };
        let mut name: ByteString = ByteString::from(name_prefix);
        name.read_data(&data[0..data_end_offset]);

        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x07, 0x01, 0xe8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0xa4, 0x6f,
            0xe0, 0xd7, 0x6d, 0x79, 0x78, 0x61, 0x74, 0x74, 0x72, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtAttributesEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name_size, 7);
        assert_eq!(test_struct.name_index, 1);
        assert_eq!(test_struct.value_data_offset, 1000);
        assert_eq!(test_struct.value_data_inode_number, 0);
        assert_eq!(test_struct.value_data_size, 21);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtAttributesEntry::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_name() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtAttributesEntry::new();
        test_struct.read_data(&test_data)?;

        let name: ByteString = test_struct.read_name(&test_data[16..])?;

        assert_eq!(name.to_string(), "user.myxattr");

        Ok(())
    }
}
