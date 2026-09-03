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

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext};
use keramics_core::{DataStreamReference, ErrorTrace};

use crate::lru_cache::LruCache;
use crate::traits::BlockReader;

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::enums::UdifCompressionMethod;

/// Universal Disk Image Format (UDIF) block reader.
pub struct UdifBlockReader {
    /// Segments data stream.
    segments_data_stream: DataStreamReference,

    /// Block ranges.
    block_ranges: Vec<UdifBlockRange>,

    /// Compression method.
    compression_method: UdifCompressionMethod,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Size.
    size: u64,
}

impl UdifBlockReader {
    /// Creates a new storage media image.
    pub fn new(
        segments_data_stream: &DataStreamReference,
        block_ranges: &[UdifBlockRange],
        compression_method: &UdifCompressionMethod,
        size: u64,
    ) -> Self {
        Self {
            segments_data_stream: segments_data_stream.clone(),
            block_ranges: block_ranges.to_vec(),
            compression_method: compression_method.clone(),
            block_cache: LruCache::new(64),
            size,
        }
    }

    /// Decompressed a block.
    fn decompress_block(&self, compressed_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        match self.compression_method {
            UdifCompressionMethod::Adc => {
                let mut adc_context: AdcContext = AdcContext::new();

                match adc_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress ADC data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Bzip2 => {
                let mut bzip2_context: Bzip2Context = Bzip2Context::new();

                match bzip2_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress bzip2 data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Lzfse => {
                let mut lzfse_context: LzfseContext = LzfseContext::new();

                match lzfse_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress LZFSE data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Lzma => {
                // TODO: add support for UdifCompressionMethod::Lzma,
                todo!();
            }
            UdifCompressionMethod::Zlib => {
                _ = crate::zlib_decompress!(
                    &compressed_data,
                    data,
                    "Unable to decompress zlib data"
                );
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported compression method"
                ));
            }
        }
        Ok(())
    }
}

impl BlockReader for UdifBlockReader {
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
            let range_end_offset: u64 = block_range.media_offset + block_range.size;

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < block_range.media_offset {
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
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &UdifBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.media_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            match block_range.range_type {
                UdifBlockRangeType::Compressed => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    if !self.block_cache.contains(&block_range.data_offset) {
                        let mut compressed_data: Vec<u8> =
                            vec![0; block_range.compressed_data_size as usize];

                        keramics_core::data_stream_read_exact_at_position!(
                            &self.segments_data_stream,
                            &mut compressed_data,
                            SeekFrom::Start(block_range.data_offset),
                        );
                        let mut block_data: Vec<u8> = vec![0; block_range.size as usize];

                        match self.decompress_block(&compressed_data, &mut block_data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to decompress block at offset: {} (0x{:08x})",
                                        block_range.data_offset, block_range.data_offset
                                    )
                                );
                                return Err(error);
                            }
                        }
                        self.block_cache.insert(block_range.data_offset, block_data);
                    }
                    let range_data: &[u8] = match self.block_cache.get(&block_range.data_offset) {
                        Some(block_data) => block_data,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unable to retrieve data from cache"
                            )));
                        }
                    };
                    if range_data.len() != (block_range.size as usize) {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve block range data",
                        ));
                    }
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);
                }
                UdifBlockRangeType::InFile => {
                    let range_data_offset: u64 = block_range.data_offset + range_relative_offset;

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.segments_data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_data_offset),
                    );
                }
                UdifBlockRangeType::Sparse => {
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
