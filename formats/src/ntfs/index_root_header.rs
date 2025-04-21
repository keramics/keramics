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
use types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "attribute_type", data_type = "u32", format = "hex")),
        member(field(name = "collation_type", data_type = "u32")),
        member(field(name = "index_entry_size", data_type = "u32")),
        member(field(name = "index_entry_number_of_cluster_blocks", data_type = "u32")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) index root header.
pub struct NtfsIndexRootHeader {
    /// Attribute type.
    pub attribute_type: u32,

    /// Collation type.
    pub collation_type: u32,

    /// Index entry size.
    pub index_entry_size: u32,
}

impl NtfsIndexRootHeader {
    /// Creates a new index root header.
    pub fn new() -> Self {
        Self {
            attribute_type: 0,
            collation_type: 0,
            index_entry_size: 0,
        }
    }

    /// Reads the index root header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.attribute_type = bytes_to_u32_le!(data, 0);
        self.collation_type = bytes_to_u32_le!(data, 4);
        self.index_entry_size = bytes_to_u32_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x30, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsIndexRootHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.attribute_type, 0x00000030);
        assert_eq!(test_struct.collation_type, 1);
        assert_eq!(test_struct.index_entry_size, 4096);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsIndexRootHeader::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
