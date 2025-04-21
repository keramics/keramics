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

use layout_map::LayoutMap;
use types::{bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "attribute_type", data_type = "u32", format = "hex")),
        member(field(name = "attribute_size", data_type = "u32")),
        member(field(name = "non_resident_flag", data_type = "u8", format = "hex")),
        member(field(name = "name_size", data_type = "u8")),
        member(field(name = "name_offset", data_type = "u16")),
        member(field(name = "data_flags", data_type = "u16", format = "hex")),
        member(field(name = "identifier", data_type = "u16")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) Master File Table (MFT) attribute header.
pub struct NtfsMftAttributeHeader {
    /// Attribute type.
    pub attribute_type: u32,

    /// Attribute size.
    pub attribute_size: u32,

    /// Non-resident flag.
    pub non_resident_flag: u8,

    /// Name size.
    pub name_size: u8,

    /// Name offset.
    pub name_offset: u16,

    /// Data flags.
    pub data_flags: u16,
}

impl NtfsMftAttributeHeader {
    /// Creates a new MFT attribute header.
    pub fn new() -> Self {
        Self {
            attribute_type: 0,
            attribute_size: 0,
            non_resident_flag: 0,
            name_size: 0,
            name_offset: 0,
            data_flags: 0,
        }
    }

    /// Reads the MFT attribute header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.attribute_type = bytes_to_u32_le!(data, 0);
        self.attribute_size = bytes_to_u32_le!(data, 4);
        self.non_resident_flag = data[8];
        self.name_size = data[9];
        self.name_offset = bytes_to_u16_le!(data, 10);
        self.data_flags = bytes_to_u16_le!(data, 12);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x10, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsMftAttributeHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.attribute_type, 0x00000010);
        assert_eq!(test_struct.attribute_size, 96);
        assert_eq!(test_struct.non_resident_flag, 0);
        assert_eq!(test_struct.name_size, 0);
        assert_eq!(test_struct.name_offset, 24);
        assert_eq!(test_struct.data_flags, 0x0000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftAttributeHeader::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
