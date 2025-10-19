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

use keramics_checksums::Adler32Context;
use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "md5_hash", data_type = "[u8; 16]", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 16]", format = "hex"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// Expert Witness Compression Format (EWF) hash.
pub struct EwfHash {
    /// MD5 hash.
    pub md5_hash: [u8; 16],
}

impl EwfHash {
    /// Creates a new hash.
    pub fn new() -> Self {
        Self { md5_hash: [0; 16] }
    }

    /// Reads the hash from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 36 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported EWF hash data size"
            ));
        }
        let stored_checksum: u32 = bytes_to_u32_le!(data, 32);

        let mut adler32_context: Adler32Context = Adler32Context::new(1);
        adler32_context.update(&data[0..32]);
        let calculated_checksum: u32 = adler32_context.finalize();

        if stored_checksum != calculated_checksum {
            return Err(keramics_core::error_trace_new!(format!(
                "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                stored_checksum, calculated_checksum
            )));
        }
        self.md5_hash.copy_from_slice(&data[0..16]);

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
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x4b, 0x08, 0x9c, 0xca,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfHash::new();
        test_struct.read_data(&test_data)?;

        let expected_md5_hash: [u8; 16] = [
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97,
        ];
        assert_eq!(test_struct.md5_hash, expected_md5_hash);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfHash::new();
        let result = test_struct.read_data(&test_data[0..35]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_checksum_mismatch() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[32] = 0xff;

        let mut test_struct = EwfHash::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = EwfHash::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        let expected_md5_hash: [u8; 16] = [
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97,
        ];
        assert_eq!(test_struct.md5_hash, expected_md5_hash);

        Ok(())
    }
}
