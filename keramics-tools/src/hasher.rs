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

use std::io::Read;

use keramics_core::ErrorTrace;
use keramics_core::formatters::format_as_string;
use keramics_hashes::{
    DigestHashContext, Md5Context, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};

use crate::enums::DigestHashType;

/// Calculate message digest hashes of data streams.
pub struct DigestHasher {
    /// Digest hash type.
    hash_type: DigestHashType,
}

impl DigestHasher {
    const READ_BUFFER_SIZE: usize = 65536;

    /// Creates a new digest hasher.
    pub fn new(hash_type: &DigestHashType) -> Self {
        Self {
            hash_type: hash_type.clone(),
        }
    }

    /// Calculates a digest hash from a buffered reader.
    pub fn calculate_hash_from_reader(&self, reader: &mut dyn Read) -> Result<String, ErrorTrace> {
        let mut hash_context: Box<dyn DigestHashContext> = match self.hash_type {
            DigestHashType::Md5 => Box::new(Md5Context::new()),
            DigestHashType::Sha1 => Box::new(Sha1Context::new()),
            DigestHashType::Sha224 => Box::new(Sha224Context::new()),
            DigestHashType::Sha256 => Box::new(Sha256Context::new()),
            DigestHashType::Sha512 => Box::new(Sha512Context::new()),
        };
        let mut data: [u8; DigestHasher::READ_BUFFER_SIZE] = [0; DigestHasher::READ_BUFFER_SIZE];

        loop {
            let read_count = match reader.read(&mut data) {
                Ok(read_count) => read_count,
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to read from reader",
                        error
                    ));
                }
            };
            if read_count == 0 {
                break;
            }
            hash_context.update(&data[0..read_count]);
        }
        let hash: Vec<u8> = hash_context.finalize();

        Ok(format_as_string(&hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn test_calculate_md5() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ];
        let mut reader: Cursor<&[u8]> = Cursor::new(&test_data);

        let hasher: DigestHasher = DigestHasher::new(&DigestHashType::Md5);
        let md5: String = hasher.calculate_hash_from_reader(&mut reader)?;
        assert_eq!(md5, "f19106bcf25fa9cabc1b5ac91c726001");

        Ok(())
    }
}
