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

use crate::bytes_to_u32_be;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "[u8; 4]"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "sectors_per_band", data_type = "u32"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "number_of_sectors", data_type = "u32"),
        field(name = "unknown3", data_type = "[u8; 12]"),
        field(name = "unknown4", data_type = "[u8; 4]"),
        field(name = "unknown5", data_type = "[u8; 28]"),
    ),
    method(name = "debug_read_data")
)]
/// Mac OS sparse image (.sparseimage) file header.
pub struct SparseImageFileHeader {
    /// The number of sectors per band.
    pub sectors_per_band: u32,

    /// The total number of sectors in the image.
    pub number_of_sectors: u32,
}

impl SparseImageFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            sectors_per_band: 0,
            number_of_sectors: 0,
        }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != SPARSEIMAGE_FILE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        self.sectors_per_band = bytes_to_u32_be!(data, 8);
        self.number_of_sectors = bytes_to_u32_be!(data, 16);

        if self.sectors_per_band == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid sectors per band: {} value out of bounds",
                    self.sectors_per_band
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
            0x73, 0x70, 0x72, 0x73, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = SparseImageFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.sectors_per_band, 2048);
        assert_eq!(test_struct.number_of_sectors, 8192);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = SparseImageFileHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..63]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = SparseImageFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_sectors_per_band() {
        let mut test_data = get_test_data();
        test_data[10] = 0x00;

        let mut test_struct = SparseImageFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
