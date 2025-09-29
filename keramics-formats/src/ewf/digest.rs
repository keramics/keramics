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
use keramics_types::bytes_to_u32_le;
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "md5_hash", data_type = "[u8; 16]", format = "hex"),
        field(name = "sha1_hash", data_type = "[u8; 20]", format = "hex"),
        field(name = "padding1", data_type = "[u8; 40]"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// Expert Witness Compression Format (EWF) digest.
pub struct EwfDigest {
    /// MD5 hash.
    pub md5_hash: [u8; 16],

    /// SHA1 hash.
    pub sha1_hash: [u8; 20],
}

impl EwfDigest {
    /// Creates a new digest.
    pub fn new() -> Self {
        Self {
            md5_hash: [0; 16],
            sha1_hash: [0; 20],
        }
    }

    /// Reads the digest from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 80 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF digest data size"),
            ));
        }
        let stored_checksum: u32 = bytes_to_u32_le!(data, 76);

        let mut adler32_context: Adler32Context = Adler32Context::new(1);
        adler32_context.update(&data[0..76]);
        let calculated_checksum: u32 = adler32_context.finalize();

        if stored_checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} EWF digest checksums",
                    stored_checksum, calculated_checksum
                ),
            ));
        }
        self.md5_hash.copy_from_slice(&data[0..16]);
        self.sha1_hash.copy_from_slice(&data[16..36]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97, 0x8a, 0xc0, 0x09, 0x25, 0xfa, 0x09, 0xa8, 0x99, 0x83, 0x9b, 0xda, 0x5f,
            0x7f, 0xbf, 0xa5, 0xa3, 0x57, 0xec, 0x0e, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9c, 0x12, 0x28, 0x3f,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfDigest::new();
        test_struct.read_data(&test_data)?;

        let expected_md5_hash: [u8; 16] = [
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97,
        ];
        assert_eq!(test_struct.md5_hash, expected_md5_hash);

        let expected_sha1_hash: [u8; 20] = [
            0x8a, 0xc0, 0x09, 0x25, 0xfa, 0x09, 0xa8, 0x99, 0x83, 0x9b, 0xda, 0x5f, 0x7f, 0xbf,
            0xa5, 0xa3, 0x57, 0xec, 0x0e, 0x67,
        ];
        assert_eq!(test_struct.sha1_hash, expected_sha1_hash);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfDigest::new();
        let result = test_struct.read_data(&test_data[0..79]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_checksum_mismatch() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[76] = 0xff;

        let mut test_struct = EwfDigest::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = EwfDigest::new();
        test_struct.read_at_position(&data_stream, io::SeekFrom::Start(0))?;

        let expected_md5_hash: [u8; 16] = [
            0x03, 0xc9, 0xd5, 0x33, 0x9a, 0xbf, 0x1e, 0xbd, 0xc1, 0x44, 0xb9, 0xed, 0x3d, 0x7e,
            0x45, 0x97,
        ];
        assert_eq!(test_struct.md5_hash, expected_md5_hash);

        let expected_sha1_hash: [u8; 20] = [
            0x8a, 0xc0, 0x09, 0x25, 0xfa, 0x09, 0xa8, 0x99, 0x83, 0x9b, 0xda, 0x5f, 0x7f, 0xbf,
            0xa5, 0xa3, 0x57, 0xec, 0x0e, 0x67,
        ];
        assert_eq!(test_struct.sha1_hash, expected_sha1_hash);

        Ok(())
    }
}
