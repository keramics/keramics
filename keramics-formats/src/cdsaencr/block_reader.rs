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

use std::cmp::min;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::lru_cache::LruCache;
use crate::traits::BlockReader;

use super::encryption_context::CdsaEncrEncryptionContext;

/// Mac OS Encrypted Encoding (cdsaencr) block reader.
pub struct CdsaEncrBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Data fork offset.
    data_fork_offset: u64,

    /// Block size.
    block_size: u32,

    /// Encryption context.
    encryption_context: CdsaEncrEncryptionContext,

    /// Decrypted block cache.
    block_cache: LruCache<u32, Vec<u8>>,

    /// Size.
    size: u64,
}

impl CdsaEncrBlockReader {
    /// Creates a block reader.
    pub fn new(
        data_stream: &DataStreamReference,
        data_fork_offset: u64,
        block_size: u32,
        encryption_context: &CdsaEncrEncryptionContext,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            data_fork_offset,
            block_size,
            encryption_context: encryption_context.clone(),
            block_cache: LruCache::new(64),
            size,
        }
    }

    /// Reads and decrypts a block.
    fn read_block(
        &mut self,
        block_number: u32,
        block_data_offset: u64,
    ) -> Result<Vec<u8>, ErrorTrace> {
        let mut encrypted_data: Vec<u8> = vec![0; self.block_size as usize];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut encrypted_data,
            SeekFrom::Start(block_data_offset)
        );
        let mut block_data: Vec<u8> = vec![0; self.block_size as usize];

        match self
            .encryption_context
            .decrypt_block(block_number, &encrypted_data, &mut block_data)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt block: {}", block_number)
                );
                return Err(error);
            }
        }
        Ok(block_data)
    }
}

impl BlockReader for CdsaEncrBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut block_number: u64 = current_offset / (self.block_size as u64);
        let mut block_offset: u64 = block_number * (self.block_size as u64);

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let range_relative_offset: u64 = current_offset - block_offset;
            let range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            if block_number > u32::MAX as u64 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid block number value out of bounds"
                ));
            }
            if !self.block_cache.contains(&(block_number as u32)) {
                let block_data_offset: u64 = self.data_fork_offset + block_offset;

                let block_data: Vec<u8> =
                    match self.read_block(block_number as u32, block_data_offset) {
                        Ok(data) => data,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read block: {}", block_number)
                            );
                            return Err(error);
                        }
                    };
                self.block_cache.insert(block_number as u32, block_data);
            }
            let range_data: &[u8] = match self.block_cache.get(&(block_number as u32)) {
                Some(data) => data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block: {} from cache",
                        block_number
                    )));
                }
            };
            let data_end_offset: usize = data_offset + range_read_size;
            let range_data_offset: usize = range_relative_offset as usize;
            let range_data_end_offset: usize = range_data_offset + range_read_size;

            data[data_offset..data_end_offset]
                .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);

            data_offset = data_end_offset;

            current_offset += range_read_size as u64;
            block_offset += self.block_size as u64;
            block_number += 1;
        }
        Ok(data_offset)
    }
}
