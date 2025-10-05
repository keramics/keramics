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

use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "attribute_type", data_type = "u32", format = "hex")),
        member(field(name = "attribute_size", data_type = "u16")),
        member(field(name = "name_size", data_type = "u8")),
        member(field(name = "name_offset", data_type = "u8")),
        member(field(name = "data_first_vcn", data_type = "u64")),
        member(field(name = "file_reference", data_type = "u64", format = "hex")),
        member(field(name = "identifier", data_type = "u16")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) attribute list entry.
pub struct NtfsAttributeListEntry {
    /// Attribute type.
    pub attribute_type: u32,

    /// Attribute size.
    pub attribute_size: u16,

    /// Name size.
    pub name_size: u8,

    /// Name offset.
    pub name_offset: u8,

    /// File reference.
    pub file_reference: u64,
}

impl NtfsAttributeListEntry {
    /// Creates a new attribute list entry.
    pub fn new() -> Self {
        Self {
            attribute_type: 0,
            attribute_size: 0,
            name_size: 0,
            name_offset: 0,
            file_reference: 0,
        }
    }

    /// Reads the attribute list entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 26 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported NTFS attribute list entry data size"),
            ));
        }
        self.attribute_type = bytes_to_u32_le!(data, 0);
        self.attribute_size = bytes_to_u16_le!(data, 4);
        self.name_size = data[6];
        self.name_offset = data[7];
        self.file_reference = bytes_to_u64_le!(data, 16);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x80, 0x00, 0x00, 0x00, 0x28, 0x00, 0x04, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xc8, 0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x24, 0x00,
            0x53, 0x00, 0x44, 0x00, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsAttributeListEntry::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.attribute_type, 0x00000080);
        assert_eq!(test_struct.attribute_size, 40);
        assert_eq!(test_struct.name_size, 4);
        assert_eq!(test_struct.name_offset, 26);
        assert_eq!(test_struct.file_reference, 0x10000000008c8);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsAttributeListEntry::new();
        let result = test_struct.read_data(&test_data[0..25]);
        assert!(result.is_err());
    }
}
