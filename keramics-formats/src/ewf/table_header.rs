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

use keramics_checksums::Adler32Context;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "number_of_entries", data_type = "u32"),
        field(name = "padding1", data_type = "[u8; 4]"),
        field(name = "base_offset", data_type = "u64", format = "hex"),
        field(name = "padding2", data_type = "[u8; 4]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data")
)]
/// Expert Witness Compression Format (EWF) table header.
pub struct EwfTableHeader {
    /// Number of entries.
    pub number_of_entries: u32,

    /// Base offset.
    pub base_offset: u64,
}

impl EwfTableHeader {
    /// Creates a new table header.
    pub fn new() -> Self {
        Self {
            number_of_entries: 0,
            base_offset: 0,
        }
    }

    /// Reads the table header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 24 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF table header data size"),
            ));
        }
        let stored_checksum: u32 = bytes_to_u32_le!(data, 20);

        let mut adler32_context: Adler32Context = Adler32Context::new(1);
        adler32_context.update(&data[0..20]);
        let calculated_checksum: u32 = adler32_context.finalize();

        if stored_checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} EWF table entries checksums",
                    stored_checksum, calculated_checksum
                ),
            ));
        }
        self.number_of_entries = bytes_to_u32_le!(data, 0);
        self.base_offset = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4d, 0x07, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd5, 0x00, 0xfd, 0x0d,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfTableHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_entries, 128);
        assert_eq!(test_struct.base_offset, 1869);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfTableHeader::new();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_checksum_mismatch() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[20] = 0xff;

        let mut test_struct = EwfTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
