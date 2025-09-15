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
        member(field(name = "verion", data_type = "u32")),
        member(field(name = "provider", data_type = "u32")),
        member(field(name = "unknown1", data_type = "[u8; 4]")),
        member(field(name = "compression_method", data_type = "u32")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) Windows Overlay Filter (WOF) reparse data.
pub struct NtfsWofReparseData {
    /// Compression method.
    pub compression_method: u32,
}

impl NtfsWofReparseData {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            compression_method: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size = data.len();
        if data_size != 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported WOF reparse data size"),
            ));
        }
        self.compression_method = bytes_to_u32_le!(data, 12);

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
        let mut test_struct = NtfsWofReparseData::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data[8..])?;

        assert_eq!(test_struct.compression_method, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsWofReparseData::new();
        let result = test_struct.read_data(&test_data[8..23]);
        assert!(result.is_err());
    }
}
