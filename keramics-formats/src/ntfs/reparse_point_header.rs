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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "tag", data_type = "u32", format = "hex")),
        member(field(name = "data_size", data_type = "u16")),
        member(field(name = "unknown1", data_type = "[u8; 2]")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) reparse point ($REPARSE_POINT) header.
pub struct NtfsReparsePointHeader {
    /// Tag.
    pub tag: u32,

    /// Data size.
    pub data_size: u16,
}

impl NtfsReparsePointHeader {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            tag: 0,
            data_size: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size = data.len();
        if data_size < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported NTFS reparse point header data size"),
            ));
        }
        self.tag = bytes_to_u32_le!(data, 0);
        self.data_size = bytes_to_u16_le!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x17, 0x00, 0x00, 0x80, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsReparsePointHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.tag, 0x80000017);
        assert_eq!(test_struct.data_size, 16);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsReparsePointHeader::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
