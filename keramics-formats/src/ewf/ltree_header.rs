/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "integrity_hash", data_type = "[u8; 16]"),
        field(name = "data_size", data_type = "u64"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 20]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Expert Witness Compression Format (EWF) ltree header.
pub struct EwfLtreeHeader {
    /// Data size.
    pub data_size: u64,

    /// Checksum.
    pub checksum: u32,
}

impl EwfLtreeHeader {
    /// Creates a new ltree header.
    pub fn new() -> Self {
        Self {
            data_size: 0,
            checksum: 0,
        }
    }

    /// Reads the ltree header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 48 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.data_size = bytes_to_u64_le!(data, 16);
        self.checksum = bytes_to_u32_le!(data, 24);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        vec![
            0xbc, 0xd4, 0x88, 0x41, 0xd4, 0x6a, 0x3b, 0x25, 0x51, 0xd1, 0x15, 0x68, 0x0c, 0xaa,
            0x41, 0x22, 0xd2, 0x5f, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe2, 0x07, 0x97, 0x3e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfLtreeHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.data_size, 90066);
        assert_eq!(test_struct.checksum, 0x3e9707e2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfLtreeHeader::new();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = EwfLtreeHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.data_size, 90066);
        assert_eq!(test_struct.checksum, 0x3e9707e2);

        Ok(())
    }
}
