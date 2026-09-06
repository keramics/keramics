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

use std::cmp::{Ordering, min};
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::lru_cache::LruCache;
use crate::traits::BlockReader;

use super::block_range::{BdeBlockRange, BdeBlockRangeType};
use super::encryption_context::BdeEncryptionContext;

/// BitLocker disk encryption (BDE) block reader.
pub struct BdeBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Block ranges.
    block_ranges: Vec<BdeBlockRange>,

    /// Encryption context.
    encryption_context: BdeEncryptionContext,

    /// Decrypted sector cache.
    sector_cache: LruCache<u64, Vec<u8>>,

    /// Size.
    size: u64,
}

impl BdeBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        block_ranges: &[BdeBlockRange],
        encryption_context: &BdeEncryptionContext,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            bytes_per_sector,
            block_ranges: block_ranges.to_vec(),
            encryption_context: encryption_context.clone(),
            sector_cache: LruCache::new(64),
            size,
        }
    }

    /// Reads and decrypts a sector.
    fn read_sector(
        &mut self,
        sector_number: u64,
        sector_physical_offset: u64,
    ) -> Result<Vec<u8>, ErrorTrace> {
        let mut encrypted_data: Vec<u8> = vec![0; self.bytes_per_sector as usize];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut encrypted_data,
            SeekFrom::Start(sector_physical_offset)
        );
        let mut sector_data: Vec<u8> = vec![0; self.bytes_per_sector as usize];

        match self.encryption_context.decrypt_sector(
            sector_physical_offset,
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
            "BdeSectorData",
            sector_physical_offset,
            &sector_data,
            self.bytes_per_sector
        );
        Ok(sector_data)
    }
}

impl BlockReader for BdeBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut range_index: usize = match self.block_ranges.binary_search_by(|block_range| {
            let range_end_offset: u64 = block_range.logical_offset + block_range.size;

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < block_range.logical_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(range_index) => range_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing block range for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        // TODO: virtualize vista sector 0

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &BdeBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            let mut range_relative_offset: u64 = current_offset - block_range.logical_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match block_range.range_type {
                BdeBlockRangeType::Encrypted => {
                    let range_data_end_offset: usize = data_offset + range_read_size;

                    let mut sector_number: u64 = current_offset / (self.bytes_per_sector as u64);
                    let sector_logical_offset: u64 = sector_number * (self.bytes_per_sector as u64);
                    let mut sector_data_offset: usize =
                        (current_offset - sector_logical_offset) as usize;

                    let mut sector_physical_offset: u64 = (block_range.physical_offset
                        + range_relative_offset)
                        - (sector_data_offset as u64);
                    let mut sector_remainder_size: usize =
                        (self.bytes_per_sector as usize) - sector_data_offset;

                    while data_offset < range_data_end_offset {
                        if !self.sector_cache.contains(&sector_number) {
                            let sector_data: Vec<u8> =
                                match self.read_sector(sector_number, sector_physical_offset) {
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
                        let sector_data: &[u8] = match self.sector_cache.get(&sector_number) {
                            Some(data) => data,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unable to retrieve sector: {} from cache",
                                    sector_number
                                )));
                            }
                        };
                        let sector_read_size: usize =
                            min(read_size - data_offset, sector_remainder_size);

                        let data_end_offset: usize = data_offset + sector_read_size;
                        let sector_data_end_offset: usize = sector_data_offset + sector_read_size;

                        data[data_offset..data_end_offset].copy_from_slice(
                            &sector_data[sector_data_offset..sector_data_end_offset],
                        );
                        data_offset = data_end_offset;
                        current_offset += sector_read_size as u64;
                        sector_number += 1;
                        sector_physical_offset += self.bytes_per_sector as u64;
                        sector_data_offset = 0;
                        sector_remainder_size = self.bytes_per_sector as usize;
                    }
                }
                BdeBlockRangeType::InFile => {
                    let range_physical_offset: u64 =
                        block_range.physical_offset + range_relative_offset;

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_physical_offset)
                    );
                }
                BdeBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            range_index += 1;
        }
        Ok(data_offset)
    }
}
