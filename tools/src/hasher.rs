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
use std::io::Read;

use keramics::hashes::{
    DigestHashContext, Md5Context, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};
use keramics::vfs::VfsDataStreamReference;

#[derive(Clone)]
pub enum DigestHashType {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha512,
}

/// Calculate message digest hashes of data streams.
pub struct DigestHasher {
    /// Digest hash type.
    hash_type: DigestHashType,
}

impl DigestHasher {
    /// Creates a new digest hasher.
    pub fn new(hash_type: &DigestHashType) -> Self {
        Self {
            hash_type: hash_type.clone(),
        }
    }

    /// Calculates a digest hash from a data stream.
    pub fn calculate_hash_from_data_stream(
        &self,
        vfs_data_stream: &VfsDataStreamReference,
    ) -> io::Result<Vec<u8>> {
        let mut hash_context: Box<dyn DigestHashContext> = match self.hash_type {
            DigestHashType::Md5 => Box::new(Md5Context::new()),
            DigestHashType::Sha1 => Box::new(Sha1Context::new()),
            DigestHashType::Sha224 => Box::new(Sha224Context::new()),
            DigestHashType::Sha256 => Box::new(Sha256Context::new()),
            DigestHashType::Sha512 => Box::new(Sha512Context::new()),
        };
        let mut data: [u8; 65536] = [0; 65536];

        match vfs_data_stream.with_write_lock() {
            Ok(mut data_stream) => {
                while let Ok(read_count) = data_stream.read(&mut data) {
                    if read_count == 0 {
                        break;
                    }
                    hash_context.update(&data[0..read_count]);
                }
            }
            Err(error) => return Err(keramics::error_to_io_error!(error)),
        };
        Ok(hash_context.finalize())
    }

    /// Calculates a digest hash from a buffered reader.
    pub fn calculate_hash_from_reader(&self, reader: &mut dyn Read) -> Vec<u8> {
        let mut hash_context: Box<dyn DigestHashContext> = match self.hash_type {
            DigestHashType::Md5 => Box::new(Md5Context::new()),
            DigestHashType::Sha1 => Box::new(Sha1Context::new()),
            DigestHashType::Sha224 => Box::new(Sha224Context::new()),
            DigestHashType::Sha256 => Box::new(Sha256Context::new()),
            DigestHashType::Sha512 => Box::new(Sha512Context::new()),
        };
        let mut data: [u8; 65536] = [0; 65536];

        while let Ok(read_count) = reader.read(&mut data) {
            if read_count == 0 {
                break;
            }
            hash_context.update(&data[0..read_count]);
        }
        hash_context.finalize()
    }
}

// TODO: add tests
