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

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext, XzContext};
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

    /// Decompresses a block.
    fn decompress_block(&self, compressed_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        match self.compression_method {
            UdifCompressionMethod::Adc => {
                let mut adc_context: AdcContext = AdcContext::new();

                match adc_context.decompress(compressed_data, data) {
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

                match bzip2_context.decompress(compressed_data, data) {
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

                match lzfse_context.decompress(compressed_data, data) {
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
                let mut xz_context: XzContext = XzContext::new();

                match xz_context.decompress(compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress xz/LZMA data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Zlib => {
                _ = crate::zlib_decompress!(
                    &compressed_data,
                    data,
                    "Unable to decompress zlib/DEFLATE data"
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

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    /// Valid xz stream that decompresses to b"test".
    const TEST_XZ_DATA: [u8; 56] = [
        0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00, 0x00, 0x01, 0x69, 0x22, 0xde, 0x36, 0x02, 0x00, 0x21,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x37, 0x27, 0x97, 0xd6, 0x01, 0x00, 0x03, 0x74, 0x65, 0x73,
        0x74, 0x00, 0x0c, 0x7e, 0x7f, 0xd8, 0x00, 0x01, 0x18, 0x04, 0x6b, 0xe9, 0xf0, 0xa5, 0x90,
        0x42, 0x99, 0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x59, 0x5a,
    ];

    #[test]
    fn test_decompress_block_with_lzma() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let block_reader: UdifBlockReader =
            UdifBlockReader::new(&data_stream, &[], &UdifCompressionMethod::Lzma, 4);

        let mut uncompressed_data: [u8; 4] = [0; 4];
        block_reader.decompress_block(&TEST_XZ_DATA, &mut uncompressed_data)?;
        assert_eq!(&uncompressed_data, b"test");

        let mut invalid_data: Vec<u8> = TEST_XZ_DATA.to_vec();
        invalid_data[0] = 0x00;
        let result: Result<(), ErrorTrace> =
            block_reader.decompress_block(&invalid_data, &mut uncompressed_data);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_decompress_block_with_errors() {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut uncompressed_data: [u8; 4] = [0; 4];

        let methods: [UdifCompressionMethod; 5] = [
            UdifCompressionMethod::Adc,
            UdifCompressionMethod::Bzip2,
            UdifCompressionMethod::Lzfse,
            UdifCompressionMethod::Zlib,
            UdifCompressionMethod::None,
        ];
        for compression_method in methods {
            let block_reader: UdifBlockReader =
                UdifBlockReader::new(&data_stream, &[], &compression_method, 4);
            let result: Result<(), ErrorTrace> =
                block_reader.decompress_block(&[0xff, 0xff], &mut uncompressed_data);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_read_data_from_blocks_with_lzma() -> Result<(), ErrorTrace> {
        let mut segment_data: Vec<u8> = TEST_XZ_DATA.to_vec();
        segment_data.extend_from_slice(b"raw!");
        let data_stream: DataStreamReference = open_fake_data_stream(&segment_data);

        let block_ranges: [UdifBlockRange; 3] = [
            UdifBlockRange::new(
                0,
                0,
                4,
                TEST_XZ_DATA.len() as u32,
                UdifBlockRangeType::Compressed,
            ),
            UdifBlockRange::new(
                4,
                TEST_XZ_DATA.len() as u64,
                4,
                0,
                UdifBlockRangeType::InFile,
            ),
            UdifBlockRange::new(8, 0, 4, 0, UdifBlockRangeType::Sparse),
        ];
        let mut block_reader: UdifBlockReader = UdifBlockReader::new(
            &data_stream,
            &block_ranges,
            &UdifCompressionMethod::Lzma,
            12,
        );
        assert_eq!(block_reader.get_size(), 12);

        let mut data: [u8; 12] = [0xff; 12];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 0)?;
        assert_eq!(read_count, 12);
        assert_eq!(&data, b"testraw!\x00\x00\x00\x00");

        // Second read exercises cached compressed block.
        let mut cached_data: [u8; 4] = [0; 4];
        let read_count: usize = block_reader.read_data_from_blocks(&mut cached_data, 0)?;
        assert_eq!(read_count, 4);
        assert_eq!(&cached_data, b"test");

        // Missing block range returns error.
        let result: Result<usize, ErrorTrace> =
            block_reader.read_data_from_blocks(&mut cached_data, 100);
        assert!(result.is_err());

        // Corrupted LZMA block returns error.
        let mut bad_segment_data: Vec<u8> = TEST_XZ_DATA.to_vec();
        bad_segment_data[0] = 0x00;
        let bad_stream: DataStreamReference = open_fake_data_stream(&bad_segment_data);
        let mut bad_reader: UdifBlockReader = UdifBlockReader::new(
            &bad_stream,
            &block_ranges[0..1],
            &UdifCompressionMethod::Lzma,
            4,
        );
        let result: Result<usize, ErrorTrace> =
            bad_reader.read_data_from_blocks(&mut cached_data, 0);
        assert!(result.is_err());

        Ok(())
    }
}
