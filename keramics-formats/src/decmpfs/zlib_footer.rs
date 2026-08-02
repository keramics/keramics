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

use super::constants::*;

#[derive(Debug, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "unknown1", data_type = "[u8; 24]"),
        field(name = "unknown2", data_type = "u16"),
        field(name = "unknown3", data_type = "u16"),
        field(name = "unknown4", data_type = "u16"),
        field(name = "signature", data_type = "ByteString<4>"),
        field(name = "unknown5", data_type = "u16"),
        field(name = "unknown6", data_type = "u16"),
        field(name = "unknown7", data_type = "u16"),
        field(name = "unknown8", data_type = "u16"),
        field(name = "unknown9", data_type = "[u8; 8]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Apple File System Compression (decmpfs) zlib (compressed) footer.
pub struct DecmpfsZlibFooter {}

impl DecmpfsZlibFooter {
    /// Creates a new footer.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the fork descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 50 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[30..34] != DECMPFS_FOOTER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x32,
            0x00, 0x00, 0x63, 0x6d, 0x70, 0x66, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x01, 0xff, 0xff,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = DecmpfsZlibFooter::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = DecmpfsZlibFooter::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..49]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[30] = 0xff;

        let mut test_struct = DecmpfsZlibFooter::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = DecmpfsZlibFooter::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        Ok(())
    }
}
