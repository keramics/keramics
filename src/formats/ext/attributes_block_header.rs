/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::bytes_to_u32_le;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 4]"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "attributes_hash", data_type = "u32"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "unknown2", data_type = "[u8; 12]"),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext) attributes block header.
pub struct ExtAttributesBlockHeader {}

impl ExtAttributesBlockHeader {
    /// Creates a new attributes block header.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the attributes block header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != EXT_ATTRIBUTES_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        let number_of_blocks: u32 = bytes_to_u32_le!(data, 8);

        if number_of_blocks != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid number of blocks: {} value out of bounds",
                    number_of_blocks
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
            0x00, 0x00, 0x02, 0xea, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xa4, 0x6f,
            0xe0, 0xd7, 0x9b, 0xfa, 0x78, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtAttributesBlockHeader::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtAttributesBlockHeader::new();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExtAttributesBlockHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
