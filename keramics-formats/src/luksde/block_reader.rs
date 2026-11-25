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
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::lru_cache::LruCache;
use crate::traits::BlockReader;

use super::encryption_context::LuksEncryptionContext;

/// Linux Unified Key Setup (LUKS) Disk Encryption block reader.
pub struct LuksBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Encrypted data offset.
    encrypted_data_offset: u64,

    /// Encryption context.
    encryption_context: Arc<LuksEncryptionContext>,

    /// Initialization vector tweak.
    initialization_vector_tweak: u64,

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
        encryption_context: &Arc<LuksEncryptionContext>,
        initialization_vector_tweak: u64,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            bytes_per_sector,
            encrypted_data_offset,
            encryption_context: encryption_context.clone(),
            initialization_vector_tweak,
            sector_cache: LruCache::new(64),
            size,
        }
    }

    /// Retrieves decrypted sector data from the cache, reading and caching it if needed.
    fn get_sector_data(
        &mut self,
        sector_number: u64,
        sector_offset: u64,
    ) -> Result<&[u8], ErrorTrace> {
        if !self.sector_cache.contains(&sector_number) {
            let sector_data_offset: u64 = self.encrypted_data_offset + sector_offset;

            let sector_data: Vec<u8> = match self.read_sector(sector_number, sector_data_offset) {
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
        match self.sector_cache.get(&sector_number) {
            Some(data) => Ok(data),
            None => Err(keramics_core::error_trace_new!(format!(
                "Unable to retrieve sector: {} from cache",
                sector_number
            ))),
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
        // dm-crypt counts initialization vector sector offsets in 512-byte sectors even for larger sector sizes.
        let initialization_vector_sector_number: u64 = self
            .initialization_vector_tweak
            .wrapping_add(sector_number * ((self.bytes_per_sector as u64) / 512));

        match self.encryption_context.decrypt_sector(
            initialization_vector_sector_number,
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

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut sector_number: u64 = current_offset / (self.bytes_per_sector as u64);
        let mut sector_offset: u64 = sector_number * (self.bytes_per_sector as u64);

        while data_offset < read_size && current_offset < self.size {
            let range_relative_offset: u64 = current_offset - sector_offset;
            let range_remainder_size: u64 = (self.bytes_per_sector as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let range_data: &[u8] = match self.get_sector_data(sector_number, sector_offset) {
                Ok(data) => data,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve sector: {} data", sector_number)
                    );
                    return Err(error);
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

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use super::super::encryption::LuksEncryption;
    use super::super::encryption_type::LuksEncryptionType;

    fn get_encryption_context() -> Result<Arc<LuksEncryptionContext>, ErrorTrace> {
        let mut encryption_type: LuksEncryptionType = LuksEncryptionType::new();
        encryption_type.set_encryption_string("aes-cbc-plain64");
        encryption_type.key_size = 16;

        let key: [u8; 16] = [0x11; 16];
        let context: LuksEncryptionContext =
            LuksEncryption::get_encryption_context(&encryption_type, &key)?.unwrap();
        Ok(Arc::new(context))
    }

    #[test]
    fn test_get_size_and_read_data_from_blocks() -> Result<(), ErrorTrace> {
        let encryption_context: Arc<LuksEncryptionContext> = get_encryption_context()?;
        let raw_data: Vec<u8> = vec![0x22; 1536];
        let data_stream: DataStreamReference = open_fake_data_stream(&raw_data);

        let mut block_reader: LuksBlockReader =
            LuksBlockReader::new(&data_stream, 512, 512, &encryption_context, 4, 1024);

        let size: u64 = block_reader.get_size();
        assert_eq!(size, 1024);

        // Test with read spanning two sectors (and non-zero tweak).
        let mut buffer: [u8; 64] = [0; 64];
        let read_count: usize = block_reader.read_data_from_blocks(&mut buffer, 480)?;
        assert_eq!(read_count, 64);

        // Test with cached sector hit.
        let mut cached_buffer: [u8; 32] = [0; 32];
        let cached_read_count: usize =
            block_reader.read_data_from_blocks(&mut cached_buffer, 480)?;
        assert_eq!(cached_read_count, 32);
        assert_eq!(&cached_buffer, &buffer[0..32]);

        // Test with read crossing end of volume.
        let mut tail_buffer: [u8; 32] = [0; 32];
        let tail_read_count: usize = block_reader.read_data_from_blocks(&mut tail_buffer, 1008)?;
        assert_eq!(tail_read_count, 16);

        // Test with read at or past end of volume.
        let eof_read_count: usize = block_reader.read_data_from_blocks(&mut tail_buffer, 1024)?;
        assert_eq!(eof_read_count, 0);

        // Test with empty output buffer.
        let mut empty_buffer: [u8; 0] = [];
        let empty_read_count: usize = block_reader.read_data_from_blocks(&mut empty_buffer, 0)?;
        assert_eq!(empty_read_count, 0);

        // Test with 4096-byte sectors using 512-byte initialization vector sector offsets.
        let large_sector_raw: Vec<u8> = vec![0x33; 8192];
        let large_sector_stream: DataStreamReference = open_fake_data_stream(&large_sector_raw);
        let mut large_sector_reader: LuksBlockReader =
            LuksBlockReader::new(&large_sector_stream, 4096, 0, &encryption_context, 0, 8192);
        let mut decrypted_sector_1: Vec<u8> = vec![0; 4096];
        let read_count: usize =
            large_sector_reader.read_data_from_blocks(&mut decrypted_sector_1, 4096)?;
        assert_eq!(read_count, 4096);

        let mut expected_sector_1: Vec<u8> = vec![0; 4096];
        encryption_context.decrypt_sector(
            8,
            &large_sector_raw[4096..8192],
            &mut expected_sector_1,
        )?;
        assert_eq!(decrypted_sector_1, expected_sector_1);

        // Test with truncated underlying data stream.
        let short_stream: DataStreamReference = open_fake_data_stream(&[0; 128]);
        let mut short_reader: LuksBlockReader =
            LuksBlockReader::new(&short_stream, 512, 512, &encryption_context, 0, 1024);
        let result: Result<usize, ErrorTrace> = short_reader.read_data_from_blocks(&mut buffer, 0);
        assert!(result.is_err());

        Ok(())
    }
}
