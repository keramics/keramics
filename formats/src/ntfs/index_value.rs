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
use types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "file_reference", data_type = "u64", format = "hex")),
        member(field(name = "size", data_type = "u16")),
        member(field(name = "key_data_size", data_type = "u16")),
        member(field(name = "flags", data_type = "u32", format = "hex")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) index value.
pub struct NtfsIndexValue {
    /// File reference.
    pub file_reference: u64,

    /// Size.
    pub size: u16,

    /// Key data size.
    pub key_data_size: u16,

    /// Flags.
    pub flags: u32,
}

impl NtfsIndexValue {
    /// Creates a new index value.
    pub fn new() -> Self {
        Self {
            file_reference: 0,
            size: 0,
            key_data_size: 0,
            flags: 0,
        }
    }

    /// Reads the index value from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.file_reference = bytes_to_u64_le!(data, 0);
        self.size = bytes_to_u16_le!(data, 8);
        self.key_data_size = bytes_to_u16_le!(data, 10);
        self.flags = bytes_to_u32_le!(data, 12);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsIndexValue::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.file_reference, 0);
        assert_eq!(test_struct.size, 24);
        assert_eq!(test_struct.key_data_size, 0);
        assert_eq!(test_struct.flags, 0x00000003);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsIndexValue::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
