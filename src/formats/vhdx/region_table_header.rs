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

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "number_of_entries", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) region table header.
pub struct VhdxRegionTableHeader {
    /// Checksum.
    pub checksum: u32,

    /// Number of entries.
    pub number_of_entries: u32,
}

impl VhdxRegionTableHeader {
    /// Creates a new region table header.
    pub fn new() -> Self {
        Self {
            checksum: 0,
            number_of_entries: 0,
        }
    }

    /// Reads the region table header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != VHDX_REGION_TABLE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        self.checksum = crate::bytes_to_u32_le!(data, 4);
        self.number_of_entries = crate::bytes_to_u32_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x72, 0x65, 0x67, 0x69, 0xae, 0x8c, 0x6b, 0xc6, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data = get_test_data();

        let mut test_struct = VhdxRegionTableHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.checksum, 0xc66b8cae);
        assert_eq!(test_struct.number_of_entries, 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VhdxRegionTableHeader::new();

        let test_data = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VhdxRegionTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
