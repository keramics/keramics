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

use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, ByteString};
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "inode_number", data_type = "u32"),
        field(name = "size", data_type = "u16"),
        field(name = "name_size", data_type = "u8"),
        field(name = "file_type", data_type = "u8"),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System directory entry.
pub struct ExtDirectoryEntry {
    /// Inode number.
    pub inode_number: u32,

    /// Size.
    pub size: u16,

    /// Name size.
    pub name_size: u8,

    /// File type.
    pub file_type: u8,
}

impl ExtDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new() -> Self {
        Self {
            inode_number: 0,
            size: 0,
            name_size: 0,
            file_type: 0,
        }
    }

    /// Reads the directory entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported ext directory entry data size"),
            ));
        }
        self.inode_number = bytes_to_u32_le!(data, 0);
        self.size = bytes_to_u16_le!(data, 4);
        self.name_size = data[6];
        self.file_type = data[7];

        Ok(())
    }

    /// Reads the name from a buffer.
    pub fn read_name(&self, data: &[u8]) -> io::Result<ByteString> {
        let data_end_offset: usize = self.name_size as usize;

        if data_end_offset > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported ext directory entry name size"),
            ));
        }
        let name: ByteString = ByteString::from_bytes(&data[0..data_end_offset]);

        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x0c, 0x00, 0x00, 0x00, 0x10, 0x00, 0x05, 0x01, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtDirectoryEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.inode_number, 12);
        assert_eq!(test_struct.size, 16);
        assert_eq!(test_struct.name_size, 5);
        assert_eq!(test_struct.file_type, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtDirectoryEntry::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_name() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtDirectoryEntry::new();
        test_struct.read_data(&test_data)?;

        let name: ByteString = test_struct.read_name(&test_data[8..])?;

        assert_eq!(name.to_string(), "file1");

        Ok(())
    }
}
