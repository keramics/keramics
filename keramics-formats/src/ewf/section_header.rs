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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "section_type", data_type = "ByteString<16>"),
        field(name = "next_offset", data_type = "u64"),
        field(name = "size", data_type = "u64"),
        field(name = "padding1", data_type = "[u8; 40]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// Expert Witness Compression Format (EWF) section header.
pub struct EwfSectionHeader {
    /// Type string.
    pub section_type: [u8; 16],

    /// Next offset.
    pub next_offset: u64,

    /// Size.
    pub size: u64,
}

impl EwfSectionHeader {
    /// Creates a new section header.
    pub fn new() -> Self {
        Self {
            section_type: [0; 16],
            next_offset: 0,
            size: 0,
        }
    }

    /// Reads the section header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 76 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF section header data size"),
            ));
        }
        let stored_checksum: u32 = bytes_to_u32_le!(data, 72);

        let mut adler32_context: Adler32Context = Adler32Context::new(1);
        adler32_context.update(&data[0..72]);
        let calculated_checksum: u32 = adler32_context.finalize();

        if stored_checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} EWF section header checksums",
                    stored_checksum, calculated_checksum
                ),
            ));
        }
        self.section_type.copy_from_slice(&data[0..16]);
        self.next_offset = bytes_to_u64_le!(data, 16);
        self.size = bytes_to_u64_le!(data, 24);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x64, 0x6f, 0x6e, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xa1, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x6a, 0x02, 0x03, 0x9f,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfSectionHeader::new();
        test_struct.read_data(&test_data)?;

        let expected_section_type: [u8; 16] = [
            0x64, 0x6f, 0x6e, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(test_struct.section_type, expected_section_type);
        assert_eq!(test_struct.next_offset, 0x00000000000121a1);
        assert_eq!(test_struct.size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfSectionHeader::new();
        let result = test_struct.read_data(&test_data[0..75]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = EwfSectionHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        let expected_section_type: [u8; 16] = [
            0x64, 0x6f, 0x6e, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(test_struct.section_type, expected_section_type);
        assert_eq!(test_struct.next_offset, 0x00000000000121a1);
        assert_eq!(test_struct.size, 0);

        Ok(())
    }
}
