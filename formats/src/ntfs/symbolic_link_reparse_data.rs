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
use types::bytes_to_u16_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "substitute_name_offset", data_type = "u16")),
        member(field(name = "substitute_name_size", data_type = "u16")),
        member(field(name = "display_name_offset", data_type = "u16")),
        member(field(name = "display_name_size", data_type = "u16")),
        member(field(name = "symbolic_link_flags", data_type = "u32")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) symbolic link reparse data.
pub struct NtfsSymoblicLinkReparseData {
    /// Substitute name offset.
    pub substitute_name_offset: u16,
}

impl NtfsSymoblicLinkReparseData {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            substitute_name_offset: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size = data.len();
        if data_size < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        // TODO: read substitute name.
        self.substitute_name_offset = bytes_to_u16_le!(data, 0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x0c, 0x00, 0x00, 0xa0, 0x68, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x32, 0x00, 0x00, 0x00,
            0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x00, 0x3a, 0x00, 0x5c, 0x00, 0x74, 0x00,
            0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x69, 0x00, 0x72, 0x00, 0x31, 0x00,
            0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x66, 0x00, 0x69, 0x00,
            0x6c, 0x00, 0x65, 0x00, 0x31, 0x00, 0x5c, 0x00, 0x3f, 0x00, 0x3f, 0x00, 0x5c, 0x00,
            0x78, 0x00, 0x3a, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00,
            0x64, 0x00, 0x69, 0x00, 0x72, 0x00, 0x31, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00,
            0x73, 0x00, 0x74, 0x00, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x31, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsSymoblicLinkReparseData::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data[8..])?;

        assert_eq!(test_struct.substitute_name_offset, 42);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsSymoblicLinkReparseData::new();
        let result = test_struct.read_data(&test_data[8..19]);
        assert!(result.is_err());
    }
}
