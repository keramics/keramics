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

use keramics_compression::{LzfseContext, LzvnContext, ZlibContext};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::{bytes_to_u32_be, bytes_to_u32_le};

use super::enums::DecmpfsCompressionMethod;

use crate::lru_cache::LruCache;

/// Apple File System Compression (decmpfs) data stream.
pub struct DecmpfsDataStream {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Compression method.
    compression_method: DecmpfsCompressionMethod,

    /// The compressed size.
    compressed_size: u64,

    /// The current offset.
    current_offset: u64,

    /// The size.
    size: u64,

    /// The block offsets.
    block_offsets: Vec<u32>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// The compressed segment data buffer.
    compressed_segment_data: Vec<u8>,

    /// The decompressed block data.
    decompressed_block_data: Option<Vec<u8>>,

    /// The current compressed block index.
    current_compressed_block_index: u32,

    /// The number of compressed blocks.
    number_of_compressed_blocks: u32,
}

impl DecmpfsDataStream {
    const BLOCK_SIZE: usize = 65536;

    /// Creates a new compressed stream.
    pub(crate) fn new(compression_method: DecmpfsCompressionMethod) -> Self {
        Self {
            data_stream: None,
            compression_method,
            compressed_size: 0,
            current_offset: 0,
            size: 0,
            block_offsets: Vec::new(),
            block_cache: LruCache::new(8),
            compressed_segment_data: vec![0u8; Self::BLOCK_SIZE + 1],
            decompressed_block_data: None,
            current_compressed_block_index: u32::MAX,
            number_of_compressed_blocks: 0,
        }
    }

    /// Opens a block stream.
    pub(crate) fn open(
        &mut self,
        data_stream: &DataStreamReference,
        size: u64,
    ) -> Result<(), ErrorTrace> {
        self.compressed_size = match data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => size,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve size");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        self.data_stream = Some(data_stream.clone());
        self.size = size;

        Ok(())
    }

    /// Reads media data based on the compressed blocks.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let mut bytes_read: usize = 0;

        if self.block_offsets.is_empty() {
            self.compute_block_offsets()?;
        }

        if self.compression_method == DecmpfsCompressionMethod::Unknown5 {
            let remaining: u64 = self.size - self.current_offset;
            let read_size = std::cmp::min(data.len(), remaining as usize);
            data[..read_size].fill(0);
            self.current_offset += read_size as u64;
            return Ok(read_size);
        }

        while bytes_read < data.len() {
            let block_index: u32 = ((self.current_offset as u64) / Self::BLOCK_SIZE as u64) as u32;
            let offset_in_block: usize =
                ((self.current_offset as u64) % Self::BLOCK_SIZE as u64) as usize;

            if block_index >= self.number_of_compressed_blocks {
                break;
            }

            if self.current_compressed_block_index != block_index {
                let compressed_offset: u64 = self.block_offsets[block_index as usize] as u64;
                let compressed_length: u32 = self.block_offsets[(block_index + 1) as usize]
                    - self.block_offsets[block_index as usize];

                if compressed_length == 0 {
                    self.current_compressed_block_index = block_index;
                    continue;
                }

                if let Some(ref stream) = self.data_stream {
                    let read_result = match stream.write() {
                        Ok(mut ds) => ds.read_at_position(
                            &mut self.compressed_segment_data[..compressed_length as usize],
                            SeekFrom::Start(compressed_offset),
                        ),
                        Err(error) => Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain write lock on data stream",
                            error
                        )),
                    };
                    match read_result {
                        Ok(read) => {
                            if read != compressed_length as usize {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unable to read compressed block: expected {}, got {}",
                                    compressed_length, read
                                )));
                            }
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read compressed block at offset: {}",
                                    compressed_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                } else {
                    return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                }

                let mut decompressed = vec![0u8; Self::BLOCK_SIZE];
                let decompressed_size = match self.decompress_data(
                    &self.compressed_segment_data[..compressed_length as usize],
                    &mut decompressed,
                ) {
                    Ok(size) => size,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decompress data");
                        return Err(error);
                    }
                };
                decompressed.truncate(decompressed_size);

                self.decompressed_block_data = Some(decompressed);
                self.current_compressed_block_index = block_index;
            }

            let decompressed_data = match &self.decompressed_block_data {
                Some(d) => d.as_slice(),
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Decompressed block is empty"
                    ));
                }
            };

            if offset_in_block >= decompressed_data.len() {
                return Err(keramics_core::error_trace_new!(
                    "Invalid offset within decompressed block"
                ));
            }

            let remaining_in_block: usize = decompressed_data.len() - offset_in_block;
            let remaining_in_output: usize = data.len() - bytes_read;
            let copy_size: usize = std::cmp::min(remaining_in_block, remaining_in_output);

            data[bytes_read..bytes_read + copy_size]
                .copy_from_slice(&decompressed_data[offset_in_block..offset_in_block + copy_size]);

            bytes_read += copy_size;

            if offset_in_block + copy_size >= decompressed_data.len() {
                self.current_compressed_block_index = u32::MAX;
            }
        }

        Ok(bytes_read)
    }

    /// Gets compressed block offsets.
    fn compute_block_offsets(&mut self) -> Result<(), ErrorTrace> {
        if !self.block_offsets.is_empty() {
            return Ok(());
        }

        if let Some(ref stream) = self.data_stream {
            let read_result = match stream.write() {
                Ok(mut ds) => {
                    ds.read_at_position(&mut self.compressed_segment_data[..4], SeekFrom::Start(0))
                }
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                )),
            };
            match read_result {
                Ok(read) => {
                    if read != 4 {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to read header signature"
                        ));
                    }
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read header");
                    return Err(error);
                }
            }
        } else {
            return Err(keramics_core::error_trace_new!("Data stream is not opened"));
        }

        if &self.compressed_segment_data[..4] == b"fpmc" {
            self.number_of_compressed_blocks = 1;
            self.block_offsets = vec![16, self.compressed_size as u32];
            return Ok(());
        }

        match self.compression_method {
            DecmpfsCompressionMethod::Deflate => {
                let compressed_descriptors_offset: u32 =
                    bytes_to_u32_be!(&self.compressed_segment_data, 0);

                if compressed_descriptors_offset != 0x00000100 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed descriptors offset"
                    ));
                }

                let read_size: usize = 16 - 4;
                let read_result = if let Some(ref stream) = self.data_stream {
                    match stream.write() {
                        Ok(mut ds) => ds.read_at_position(
                            &mut self.compressed_segment_data[4..4 + read_size],
                            SeekFrom::Start(4),
                        ),
                        Err(error) => Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain write lock on data stream",
                            error
                        )),
                    }
                } else {
                    Err(keramics_core::error_trace_new!("Data stream is not opened"))
                };
                match read_result {
                    Ok(read) => {
                        if read != read_size {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to read compressed header data"
                            ));
                        }
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read compressed header data"
                        );
                        return Err(error);
                    }
                }

                let num_blocks: u32 = bytes_to_u32_le!(&self.compressed_segment_data, 260);

                if num_blocks > (u32::MAX / 8) {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid number of compressed blocks"
                    ));
                }

                self.number_of_compressed_blocks = num_blocks;

                let mut segment_offset: usize = 264;
                let block_offset: u32 =
                    bytes_to_u32_le!(&self.compressed_segment_data, segment_offset);
                segment_offset += 4;

                let descriptors_offset: u32 = 0x00000100 + 4;
                let descriptor_size: usize = 8;

                if block_offset <= descriptor_size as u32
                    || block_offset > (Self::BLOCK_SIZE + 1) as u32
                {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid first compressed block offset"
                    ));
                }

                let abs_offset: u32 = block_offset + descriptors_offset;
                self.block_offsets = vec![0; num_blocks as usize + 1];
                self.block_offsets[0] = abs_offset;
                let mut prev_offset: u32 = abs_offset;

                if num_blocks > 1 {
                    let remaining_blocks: usize = ((num_blocks - 1) as usize - 1) * descriptor_size;
                    if remaining_blocks > 0 {
                        let read_len =
                            std::cmp::min(remaining_blocks, Self::BLOCK_SIZE + 1 - segment_offset);
                        if let Some(ref stream) = self.data_stream {
                            let read_result = match stream.write() {
                                Ok(mut ds) => ds.read_at_position(
                                    &mut self.compressed_segment_data
                                        [segment_offset..segment_offset + read_len],
                                    SeekFrom::Start(segment_offset as u64),
                                ),
                                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                                    "Unable to obtain write lock on data stream",
                                    error
                                )),
                            };
                            match read_result {
                                Ok(_) => {}
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to read compressed block descriptors"
                                    );
                                    return Err(error);
                                }
                            }
                        } else {
                            return Err(keramics_core::error_trace_new!(
                                "Data stream is not opened"
                            ));
                        }
                    }
                }

                for i in 1..num_blocks as usize {
                    let offset: u32 =
                        bytes_to_u32_le!(&self.compressed_segment_data, segment_offset);
                    segment_offset += 4;

                    let abs: u32 = offset + descriptors_offset;

                    if prev_offset > abs || (abs - prev_offset) as usize > Self::BLOCK_SIZE + 1 {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid compressed block offset"
                        ));
                    }

                    self.block_offsets[i] = abs;
                    prev_offset = abs;

                    segment_offset += 4;
                }

                let compressed_footer_offset: u32 =
                    bytes_to_u32_be!(&self.compressed_segment_data, 4);
                let compressed_footer_size: u32 =
                    bytes_to_u32_be!(&self.compressed_segment_data, 12);

                if compressed_footer_size as usize > Self::BLOCK_SIZE + 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed footer size"
                    ));
                }

                if compressed_footer_size > 0 {
                    if let Some(ref stream) = self.data_stream {
                        let read_result = match stream.write() {
                            Ok(mut ds) => ds.read_at_position(
                                &mut self.compressed_segment_data
                                    [..compressed_footer_size as usize],
                                SeekFrom::Start(compressed_footer_offset as u64),
                            ),
                            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                                "Unable to obtain write lock on data stream",
                                error
                            )),
                        };
                        match read_result {
                            Ok(read) => {
                                if read != compressed_footer_size as usize {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unable to read compressed footer data"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read compressed footer data"
                                );
                                return Err(error);
                            }
                        }
                    } else {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                }

                let last_offset = self.block_offsets.last().copied().unwrap_or(0) as u64;
                if last_offset > self.compressed_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed block offset"
                    ));
                }

                self.block_offsets.push(self.compressed_size as u32);
                Ok(())
            }
            DecmpfsCompressionMethod::Lzvn => {
                let block_offset: u32 = bytes_to_u32_le!(&self.compressed_segment_data, 0);

                if block_offset <= 4 || block_offset > (Self::BLOCK_SIZE + 1) as u32 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed block offset"
                    ));
                }

                self.number_of_compressed_blocks = block_offset / 4;

                self.block_offsets = vec![0; self.number_of_compressed_blocks as usize + 1];

                let num_blocks: u32 = self.number_of_compressed_blocks;
                self.block_offsets[0] = block_offset;
                let mut prev: u32 = block_offset;

                for i in 1..num_blocks as usize {
                    let idx: usize = i * 4;
                    if idx + 4 > self.compressed_segment_data.len() {
                        return Err(keramics_core::error_trace_new!(
                            "Insufficient data for block index"
                        ));
                    }
                    let next_offset: u32 = bytes_to_u32_le!(&self.compressed_segment_data, idx);

                    if next_offset <= 4 || next_offset > (Self::BLOCK_SIZE + 1) as u32 {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid compressed block index"
                        ));
                    }

                    if prev > next_offset || (next_offset - prev) as usize > Self::BLOCK_SIZE + 1 {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid compressed block offset"
                        ));
                    }

                    self.block_offsets[i] = next_offset;
                    prev = next_offset;
                }

                self.block_offsets.push(self.compressed_size as u32);
                Ok(())
            }
            _ => Err(keramics_core::error_trace_new!(
                "Unsupported compression method for block offsets"
            )),
        }
    }

    /// Decompresses compressed data.
    fn decompress_data(
        &self,
        compressed: &[u8],
        decompressed: &mut [u8],
    ) -> Result<usize, ErrorTrace> {
        match self.compression_method {
            DecmpfsCompressionMethod::Deflate => {
                let mut context = ZlibContext::new();
                if let Err(mut error) = context.decompress(compressed, decompressed) {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to decompress deflate data"
                    );
                    return Err(error);
                }
                Ok(context.uncompressed_data_size)
            }
            DecmpfsCompressionMethod::Lzfse => {
                let mut context = LzfseContext::new();
                if let Err(mut error) = context.decompress(compressed, decompressed) {
                    keramics_core::error_trace_add_frame!(error, "Unable to decompress lzfse data");
                    return Err(error);
                }
                Ok(context.uncompressed_data_size)
            }
            DecmpfsCompressionMethod::Lzvn => {
                let mut context = LzvnContext::new();
                if let Err(mut error) = context.decompress(compressed, decompressed) {
                    keramics_core::error_trace_add_frame!(error, "Unable to decompress lzvn data");
                    return Err(error);
                }
                Ok(context.uncompressed_data_size)
            }
            DecmpfsCompressionMethod::Unknown5 => {
                decompressed[..compressed.len()].copy_from_slice(compressed);
                Ok(compressed.len())
            }
        }
    }
}

impl DataStream for DecmpfsDataStream {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = match self.read_data_from_blocks(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                return Err(error);
            }
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset value out of bounds"
                    ));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    /// LZVN compressed data from libfshfs test suite.
    /// This is a single-block LZVN compressed stream with 16 bytes of uncompressed data.
    const LZVN_COMPRESSED_DATA1: &[u8] = &[
        0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xe0, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0x0c, 0x0d, 0x0e, 0x0f, 0x06,
    ];

    /// Creates a single-block LZVN compressed data stream with the expected uncompressed size.
    fn create_lzvn_data_stream(expected_size: u64) -> (DataStreamReference, DecmpfsDataStream) {
        let fake_data = open_fake_data_stream(LZVN_COMPRESSED_DATA1);
        let mut decmpfs = DecmpfsDataStream::new(DecmpfsCompressionMethod::Lzvn);
        decmpfs.open(&fake_data, expected_size).unwrap();
        (fake_data, decmpfs)
    }

    /// Tests creating a new DecmpfsDataStream with LZVN compression.
    #[test]
    fn test_new_lzvn_stream() {
        let stream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Lzvn);
        assert_eq!(stream.compression_method, DecmpfsCompressionMethod::Lzvn);
        assert_eq!(stream.block_offsets.len(), 0);
        assert_eq!(stream.current_compressed_block_index, u32::MAX);
        assert_eq!(stream.number_of_compressed_blocks, 0);
    }

    /// Tests opening a data stream with LZVN compression (from C test fshfs_test_compressed_data_handle_initialize).
    #[test]
    fn test_open_lzvn_stream() {
        let fake_data = open_fake_data_stream(LZVN_COMPRESSED_DATA1);
        let mut stream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Lzvn);
        let result = stream.open(&fake_data, 16);
        assert!(result.is_ok());
        assert_eq!(stream.compressed_size, 35);
        assert_eq!(stream.size, 16);
    }

    /// Tests getting the offset of an LZVN stream (from C test fshfs_test_compressed_data_handle_seek_segment_offset).
    #[test]
    fn test_get_offset_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let offset = stream.get_offset().unwrap();
        assert_eq!(offset, 0);
    }

    /// Tests getting the size of an LZVN stream.
    #[test]
    fn test_get_size_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let size = stream.get_size().unwrap();
        assert_eq!(size, 16);
    }

    /// Tests seeking to start of an LZVN stream (from C test fshfs_test_compressed_data_handle_seek_segment_offset).
    #[test]
    fn test_seek_start_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let offset = stream.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(offset, 0);
    }

    /// Tests seeking from end of an LZVN stream (from C test fshfs_test_compressed_data_handle_seek_segment_offset).
    #[test]
    fn test_seek_end_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let offset = stream.seek(SeekFrom::End(-8)).unwrap();
        assert_eq!(offset, 8);
    }

    /// Tests seeking by current position of an LZVN stream (from C test fshfs_test_compressed_data_handle_seek_segment_offset).
    #[test]
    fn test_seek_current_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        stream.seek(SeekFrom::Start(4)).unwrap();
        let offset = stream.seek(SeekFrom::Current(2)).unwrap();
        assert_eq!(offset, 6);
    }

    /// Tests seeking to a specific position in an LZVN stream.
    #[test]
    fn test_seek_to_position() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let offset = stream.seek(SeekFrom::Start(10)).unwrap();
        assert_eq!(offset, 10);
    }

    /// Tests reading LZVN decompressed data (from C test fshfs_test_compressed_data_handle_read_segment_data).
    /// The test data has "fpmc" signature, so it's a single block starting at offset 16.
    /// The LZVN compressed data at offset 16 decodes to 16 bytes of data [0x00, 0x01, ..., 0x0f].
    #[test]
    fn test_read_lzvn_data() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);

        let mut buffer = [0u8; 16];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 16);

        // LZVN decode: 0xe0 = LiteralLarge opcode, 0x00 = literal_size - 16, so literal_size = 16
        // Then 16 literal bytes: 0x00, 0x01, 0x02, ..., 0x0f
        // Then 0x06 = EndOfStream
        let expected: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        assert_eq!(buffer, expected);
    }

    /// Tests reading a partial LZVN decompressed data.
    #[test]
    fn test_read_lzvn_partial_data() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        stream.seek(SeekFrom::Start(4)).unwrap();

        let mut buffer = [0u8; 8];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 8);

        let expected: [u8; 8] = [0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b];
        assert_eq!(buffer, expected);
    }

    /// Tests reading beyond the uncompressed data boundary.
    #[test]
    fn test_read_past_boundary() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        stream.seek(SeekFrom::End(-2)).unwrap();

        let mut buffer = [0u8; 8];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 2);
        assert_eq!(buffer[0], 0x0e);
        assert_eq!(buffer[1], 0x0f);
    }

    /// Tests reading past the end returns zero.
    #[test]
    fn test_read_past_end() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        stream.seek(SeekFrom::End(0)).unwrap();

        let mut buffer = [0u8; 4];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 0);
        assert_eq!(buffer, [0u8; 4]);
    }

    /// Tests reading small chunks iteratively.
    #[test]
    fn test_read_lzvn_chunks() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);

        let expected: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];

        let mut total_read = 0;
        let chunk_size = 4;
        for chunk in expected.chunks(chunk_size) {
            let mut buffer = [0u8; 4];
            let read_count = stream.read(&mut buffer).unwrap();
            assert_eq!(read_count, chunk_size);
            assert_eq!(&buffer[..read_count], chunk);
            total_read += read_count;
        }
        assert_eq!(total_read, 16);
    }

    /// Tests seeking and reading at different positions.
    #[test]
    fn test_seek_and_read_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);

        // Read first 4 bytes
        let mut buffer1 = [0u8; 4];
        stream.read(&mut buffer1).unwrap();
        assert_eq!(buffer1, [0x00, 0x01, 0x02, 0x03]);
        assert_eq!(stream.get_offset().unwrap(), 4);

        // Seek to middle
        stream.seek(SeekFrom::Start(8)).unwrap();
        assert_eq!(stream.get_offset().unwrap(), 8);

        // Read next 4 bytes
        let mut buffer2 = [0u8; 4];
        stream.read(&mut buffer2).unwrap();
        assert_eq!(buffer2, [0x08, 0x09, 0x0a, 0x0b]);
    }

    /// Tests seeking before zero returns an error (from C test fshfs_test_compressed_data_handle_seek_segment_offset).
    #[test]
    fn test_seek_before_zero() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let result = stream.seek(SeekFrom::Current(-1));
        assert!(result.is_err());
    }

    /// Tests Unknown5 compression method returns zero-filled data (from C test fshfs_test_compressed_data_handle_read_segment_data).
    #[test]
    fn test_read_unknown5_data() {
        let compressed_data = [0x66, 0x70, 0x6d, 0x63]; // "fpmc" signature
        let fake_data = open_fake_data_stream(&compressed_data);
        let mut stream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Unknown5);
        stream.open(&fake_data, 16).unwrap();

        let mut buffer = [0xFFu8; 16];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 16);

        // Unknown5 compression fills with zeros
        assert_eq!(buffer, [0u8; 16]);
    }

    /// Tests Unknown5 compression with partial reads.
    #[test]
    fn test_read_unknown5_partial() {
        let compressed_data = [0x66, 0x70, 0x6d, 0x63]; // "fpmc" signature
        let fake_data = open_fake_data_stream(&compressed_data);
        let mut stream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Unknown5);
        stream.open(&fake_data, 16).unwrap();

        let mut buffer = [0u8; 8];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 8);
        assert_eq!(buffer, [0u8; 8]);
    }

    /// Tests the data stream offset tracking during reads (from C test fshfs_test_compressed_data_handle_seek_segment_offset).  
    #[test]
    fn test_offset_tracking_lzvn() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);

        assert_eq!(stream.get_offset().unwrap(), 0);

        let mut buffer = [0u8; 4];
        stream.read(&mut buffer).unwrap();
        assert_eq!(stream.get_offset().unwrap(), 4);

        let mut buffer2 = [0u8; 8];
        stream.read(&mut buffer2).unwrap();
        assert_eq!(stream.get_offset().unwrap(), 12);

        let mut buffer3 = [0u8; 4];
        stream.read(&mut buffer3).unwrap();
        assert_eq!(stream.get_offset().unwrap(), 16);
    }

    /// Tests seeking past the end of the data.
    #[test]
    fn test_seek_past_end() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);
        let offset = stream.seek(SeekFrom::End(4)).unwrap();
        assert_eq!(offset, 20);
    }

    /// Tests the C test data: LZVN with "fpmc" signature, single block, size 16.
    /// This mirrors fshfs_test_compressed_data_handle_read_segment_data which reads into a 32-byte buffer.
    #[test]
    fn test_large_read_buffer() {
        let (_fake_data, mut stream) = create_lzvn_data_stream(16);

        // Mirror the C test: read into a 32-byte buffer (larger than expected size)
        let mut buffer = [0u8; 32];
        let read_count = stream.read(&mut buffer).unwrap();
        assert_eq!(read_count, 16);

        let expected: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        assert_eq!(&buffer[..16], &expected);
        // Remaining buffer should stay zero (initial state)
        assert_eq!(&buffer[16..], &[0u8; 16]);
    }
}
