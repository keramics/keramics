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

use super::constants::*;
use super::enums::DecmpfsCompressionMethod;

#[derive(Clone, Debug, LayoutMap, PartialEq)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 4]"),
        field(name = "compression_method", data_type = "u32"),
        field(name = "uncompressed_data_size", data_type = "u64"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Apple File System Compression (decmpfs) header.
pub struct DecmpfsHeader {
    /// Compression method.
    pub compression_method: u32,

    /// Uncompressed data size.
    pub uncompressed_data_size: u64,
}

impl DecmpfsHeader {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            compression_method: 0,
            uncompressed_data_size: 0,
        }
    }

    /// Retrieves the compression method.
    pub fn get_compression_method(&self) -> Option<DecmpfsCompressionMethod> {
        match self.compression_method {
            3 | 4 => Some(DecmpfsCompressionMethod::Zlib),
            5 => Some(DecmpfsCompressionMethod::Unknown5),
            7 | 8 => Some(DecmpfsCompressionMethod::Lzvn),
            9 | 10 => Some(DecmpfsCompressionMethod::Raw),
            11 | 12 => Some(DecmpfsCompressionMethod::Lzfse),
            13 | 14 => Some(DecmpfsCompressionMethod::LzBitmap),
            _ => None,
        }
    }

    /// Reads the fork descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..4] != DECMPFS_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.compression_method = bytes_to_u32_le!(data, 4);
        self.uncompressed_data_size = bytes_to_u64_le!(data, 8);

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
            0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe0, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x06,
        ];
    }

    #[test]
    fn test_get_compression_method() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = DecmpfsHeader::new();
        test_struct.read_data(&test_data)?;

        let compression_method: Option<DecmpfsCompressionMethod> =
            test_struct.get_compression_method();
        assert_eq!(compression_method, Some(DecmpfsCompressionMethod::Lzvn));

        Ok(())
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = DecmpfsHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.compression_method, 7);
        assert_eq!(test_struct.uncompressed_data_size, 16);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = DecmpfsHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = DecmpfsHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = DecmpfsHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.compression_method, 7);
        assert_eq!(test_struct.uncompressed_data_size, 16);

        Ok(())
    }
}
