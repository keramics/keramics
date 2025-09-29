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

use keramics_types::bytes_to_u16_le;
use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 8]", format = "hex"),
        field(name = "fields_start", data_type = "u8", format = "hex"),
        field(name = "segment_number", data_type = "u16"),
        field(name = "fields_end", data_type = "[u8; 2]", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// Expert Witness Compression Format (EWF) file header.
pub struct EwfFileHeader {
    /// Segment number.
    pub segment_number: u16,
}

impl EwfFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self { segment_number: 0 }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 13 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF file header data size"),
            ));
        }
        if data[0..8] != EWF_FILE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported EWF file header signature"),
            ));
        }
        if data[8] != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported EWF file header start of fields"),
            ));
        }
        if data[11..13] != [0; 2] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported EWF file header end of fields"),
            ));
        }
        self.segment_number = bytes_to_u16_le!(data, 9);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x45, 0x56, 0x46, 0x09, 0x0d, 0x0a, 0xff, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.segment_number, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfFileHeader::new();
        let result = test_struct.read_data(&test_data[0..12]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = EwfFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_start_of_fields() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[8] = 0xff;

        let mut test_struct = EwfFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_end_of_fields() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[11] = 0xff;

        let mut test_struct = EwfFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = EwfFileHeader::new();
        test_struct.read_at_position(&data_stream, io::SeekFrom::Start(0))?;

        assert_eq!(test_struct.segment_number, 1);

        Ok(())
    }
}
