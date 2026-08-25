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

use super::encryption::LuksEncryptionContext;

/// Linux Unified Key Setup (LUKS) Disk Encryption block reader.
pub struct LuksBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Encrypted data offset.
    encrypted_data_offset: u64,

    /// Encryption context.
    encryption_context: LuksEncryptionContext,

    /// Decrypted sector cache.
    sector_cache: LruCache<u64, Vec<u8>>,

    /// Size.
    size: u64,
}

impl LuksBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        encrypted_data_offset: u64,
        encryption_context: &LuksEncryptionContext,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            bytes_per_sector,
            encrypted_data_offset,
            encryption_context: encryption_context.clone(),
            sector_cache: LruCache::new(64),
            size,
        }
    }

    /// Reads and decrypts a sector.
    fn read_sector(
        &mut self,
        sector_number: u64,
        sector_data_offset: u64,
    ) -> Result<Vec<u8>, ErrorTrace> {
        let mut encrypted_data: Vec<u8> = vec![0; self.bytes_per_sector as usize];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut encrypted_data,
            SeekFrom::Start(sector_data_offset)
        );
        let mut sector_data: Vec<u8> = vec![0; self.bytes_per_sector as usize];

        match self.encryption_context.decrypt_sector(
            sector_number,
            &encrypted_data,
            &mut sector_data,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt sector: {}", sector_number)
                );
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!(
            "LuksSectorData",
            sector_data_offset,
            &sector_data,
            self.bytes_per_sector
        );

        Ok(sector_data)
    }
}

impl BlockReader for LuksBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut sector_number: u64 = current_offset / (self.bytes_per_sector as u64);
        let mut sector_offset: u64 = sector_number * (self.bytes_per_sector as u64);

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let range_relative_offset: u64 = current_offset - sector_offset;
            let range_remainder_size: u64 = (self.bytes_per_sector as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            if !self.sector_cache.contains(&sector_number) {
                let sector_data_offset: u64 = self.encrypted_data_offset + sector_offset;

                let sector_data: Vec<u8> = match self.read_sector(sector_number, sector_data_offset)
                {
                    Ok(data) => data,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read sector: {}", sector_number)
                        );
                        return Err(error);
                    }
                };
                self.sector_cache.insert(sector_number, sector_data);
            }
            let range_data: &[u8] = match self.sector_cache.get(&sector_number) {
                Some(data) => data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve sector: {} from cache",
                        sector_number
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
            sector_offset += self.bytes_per_sector as u64;
            sector_number += 1;
        }
        Ok(data_offset)
    }
}
