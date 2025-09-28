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

use keramics_types::{Uuid, bytes_to_u32_le};
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "item_identifier", data_type = "Uuid"),
        field(name = "item_offset", data_type = "u32", format = "hex"),
        field(name = "item_size", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 8]"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) metadata table entry.
pub struct VhdxMetadataTableEntry {
    /// Item identifier.
    pub item_identifier: Uuid,

    /// Item offset.
    pub item_offset: u32,

    /// Item size.
    pub item_size: u32,
}

impl VhdxMetadataTableEntry {
    /// Creates a new metadata table entry.
    pub fn new() -> Self {
        Self {
            item_identifier: Uuid::new(),
            item_offset: 0,
            item_size: 0,
        }
    }

    /// Reads the metadata table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported VHDX metadata table entry data size"),
            ));
        }
        self.item_identifier = Uuid::from_le_bytes(&data[0..16]);
        self.item_offset = bytes_to_u32_le!(data, 16);
        self.item_size = bytes_to_u32_le!(data, 20);

        if self.item_offset < 65536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid item offset: 0x{:04x} value out of bounds",
                    self.item_offset
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x37, 0x67, 0xa1, 0xca, 0x36, 0xfa, 0x43, 0x4d, 0xb3, 0xb6, 0x33, 0xf0, 0xaa, 0x44,
            0xe7, 0x6b, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxMetadataTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.item_identifier.to_string(),
            "caa16737-fa36-4d43-b3b6-33f0aa44e76b"
        );
        assert_eq!(test_struct.item_offset, 65536);
        assert_eq!(test_struct.item_size, 8);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxMetadataTableEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }
}
