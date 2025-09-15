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

use keramics_types::bytes_to_u32_le;
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "index_values_offset", data_type = "u32")),
        member(field(name = "size", data_type = "u32")),
        member(field(name = "allocated_size", data_type = "u32")),
        member(field(name = "flags", data_type = "u32", format = "hex")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) index node header.
pub struct NtfsIndexNodeHeader {
    /// Index values offset.
    pub index_values_offset: u32,

    /// Size.
    pub size: u32,

    /// Flags.
    pub flags: u32,
}

impl NtfsIndexNodeHeader {
    /// Creates a new index node header.
    pub fn new() -> Self {
        Self {
            index_values_offset: 0,
            size: 0,
            flags: 0,
        }
    }

    /// Reads the index node header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported NTFS index node header data size"),
            ));
        }
        self.index_values_offset = bytes_to_u32_le!(data, 0);
        self.size = bytes_to_u32_le!(data, 4);
        self.flags = bytes_to_u32_le!(data, 12);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x10, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsIndexNodeHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.index_values_offset, 16);
        assert_eq!(test_struct.size, 40);
        assert_eq!(test_struct.flags, 0x00000001);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsIndexNodeHeader::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
