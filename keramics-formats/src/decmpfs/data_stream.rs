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
        if self.size > 0 && self.block_offsets.is_empty() {
            self.compute_block_offsets()?;
        }

        let mut bytes_read: usize = 0;
        let mut current_offset: u64 = self.current_offset;

        while bytes_read < data.len() {
            if current_offset >= self.size {
                break;
            }

            let block_index: u32 = (current_offset / Self::BLOCK_SIZE as u64) as u32;

            if block_index >= self.number_of_compressed_blocks {
                break;
            }

            let compressed_offset: u64 = self.block_offsets[block_index as usize] as u64;
            let compressed_length: u32 = self.block_offsets[(block_index + 1) as usize]
                - self.block_offsets[block_index as usize];

            if compressed_length == 0 {
                break;
            }

            let range_offset: u64 = block_index as u64 * Self::BLOCK_SIZE as u64;
            let range_size: u64 = std::cmp::min(Self::BLOCK_SIZE as u64, self.size - range_offset);

            let block_relative_offset: u64 = current_offset - range_offset;
            let block_remainder_size: u64 = range_size - block_relative_offset;

            let block_read_size: usize =
                std::cmp::min(data.len() - bytes_read, block_remainder_size as usize);

            if block_read_size == 0 {
                break;
            }

            let data_end_offset: usize = bytes_read + block_read_size;

            let copied_size = if range_size == (compressed_length as u64) {
                let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                    Some(stream) => stream,
                    None => {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                };
                let read_offset: u64 = compressed_offset + block_relative_offset;

                keramics_core::data_stream_read_exact_at_position!(
                    data_stream,
                    &mut data[bytes_read..data_end_offset],
                    SeekFrom::Start(read_offset)
                );
                block_read_size
            } else {
                if !self.block_cache.contains(&compressed_offset) {
                    let mut decompressed: Vec<u8> = vec![0u8; Self::BLOCK_SIZE];
                    let decompressed_size = match self.read_compressed_block(
                        compressed_offset,
                        compressed_length as usize,
                        &mut decompressed,
                    ) {
                        Ok(size) => size,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read compressed block at offset: {} (0x{:08x})",
                                    compressed_offset, compressed_offset
                                )
                            );
                            return Err(error);
                        }
                    };
                    decompressed.truncate(decompressed_size);
                    self.block_cache.insert(compressed_offset, decompressed);
                }
                let block_data: &Vec<u8> = match self.block_cache.get(&compressed_offset) {
                    Some(data) => data,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve data from cache"
                        ));
                    }
                };

                let block_data_offset: usize = block_relative_offset as usize;
                let block_data_end_offset: usize =
                    std::cmp::min(block_data_offset + block_read_size, block_data.len());

                if block_data_offset >= block_data.len() {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset within decompressed block"
                    ));
                }

                let copy_size: usize = block_data_end_offset - block_data_offset;
                let remaining_output: usize = data_end_offset - bytes_read;
                let final_copy_size: usize = std::cmp::min(copy_size, remaining_output);

                data[bytes_read..bytes_read + final_copy_size].copy_from_slice(
                    &block_data[block_data_offset..block_data_offset + final_copy_size],
                );

                final_copy_size
            };

            bytes_read += copied_size;
            current_offset += copied_size as u64;
        }

        self.current_offset = current_offset;
        Ok(bytes_read)
    }

    /// Reads and decompresses a compressed block.
    fn read_compressed_block(
        &self,
        block_offset: u64,
        compressed_length: usize,
        decompressed: &mut [u8],
    ) -> Result<usize, ErrorTrace> {
        let compressed_data: Vec<u8> = match self.data_stream.as_ref() {
            Some(stream) => {
                let mut data = vec![0u8; compressed_length];
                match stream.write() {
                    Ok(mut ds) => {
                        match ds.read_at_position(&mut data, SeekFrom::Start(block_offset)) {
                            Ok(_) => data,
                            Err(err) => {
                                return Err(keramics_core::error_trace_new_with_error!(
                                    "Unable to read compressed block from stream",
                                    err
                                ));
                            }
                        }
                    }
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain write lock on data stream",
                            error
                        ));
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!("Data stream is not opened"));
            }
        };

        self.decompress_data(&compressed_data, decompressed)
    }

    /// Helper macro to read a LE u32 from a buffer at a given offset.
    fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
        bytes_to_u32_le!(buf, offset)
    }

    /// Gets compressed block offsets.
    fn compute_block_offsets(&mut self) -> Result<(), ErrorTrace> {
        if !self.block_offsets.is_empty() {
            return Ok(());
        }

        // Read first 4 bytes for header signature
        let mut header_buf: Vec<u8> = vec![0u8; 4];
        match &self.data_stream {
            Some(stream) => {
                let read_result = match stream.write() {
                    Ok(mut ds) => ds.read_at_position(&mut header_buf, SeekFrom::Start(0)),
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
            }
            None => return Err(keramics_core::error_trace_new!("Data stream is not opened")),
        }

        if &header_buf[..4] == b"fpmc" {
            self.number_of_compressed_blocks = 1;
            self.block_offsets = vec![16, self.compressed_size as u32];
            return Ok(());
        }

        match self.compression_method {
            DecmpfsCompressionMethod::Deflate => {
                let compressed_descriptors_offset: u32 = Self::read_u32_le(&header_buf, 0);
                if compressed_descriptors_offset != 0x00000100 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed descriptors offset"
                    ));
                }

                // Read the compressed data descriptors offset (4 bytes at offset 4)
                let mut descriptors_offset_bytes: Vec<u8> = vec![0u8; 4];
                match &self.data_stream {
                    Some(stream) => {
                        let read_result = match stream.write() {
                            Ok(mut ds) => ds.read_at_position(
                                &mut descriptors_offset_bytes,
                                SeekFrom::Start(4),
                            ),
                            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                                "Unable to obtain write lock on data stream",
                                error
                            )),
                        };
                        match read_result {
                            Ok(read) => {
                                if read != 4 {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unable to read descriptors offset"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read descriptors offset"
                                );
                                return Err(error);
                            }
                        }
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                }
                let compressed_descriptors_offset_val: u32 =
                    Self::read_u32_le(&descriptors_offset_bytes, 0);
                if compressed_descriptors_offset_val != 0x00000100 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed data descriptors offset"
                    ));
                }

                // Read block count (4 bytes at offset 260 in the descriptors)
                let mut block_count_bytes: Vec<u8> = vec![0u8; 4];
                match &self.data_stream {
                    Some(stream) => {
                        let read_result = match stream.write() {
                            Ok(mut ds) => {
                                ds.read_at_position(&mut block_count_bytes, SeekFrom::Start(260))
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
                                        "Unable to read block count"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read block count"
                                );
                                return Err(error);
                            }
                        }
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                }
                let num_blocks: u32 = Self::read_u32_le(&block_count_bytes, 0);
                if num_blocks > (u32::MAX / 8) {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid number of compressed blocks"
                    ));
                }
                self.number_of_compressed_blocks = num_blocks;

                // Read first block offset (4 bytes at offset 264 in the descriptors)
                let mut first_block_bytes: Vec<u8> = vec![0u8; 4];
                match &self.data_stream {
                    Some(stream) => {
                        let read_result = match stream.write() {
                            Ok(mut ds) => {
                                ds.read_at_position(&mut first_block_bytes, SeekFrom::Start(264))
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
                                        "Unable to read first block offset"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read first block offset"
                                );
                                return Err(error);
                            }
                        }
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                }
                let first_block_offset: u32 = Self::read_u32_le(&first_block_bytes, 0);
                if first_block_offset <= 8 || first_block_offset > (Self::BLOCK_SIZE + 1) as u32 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid first compressed block offset"
                    ));
                }

                let descriptors_region_offset: u32 = 0x00000100 + 4;
                self.block_offsets = vec![0; num_blocks as usize + 1];
                self.block_offsets[0] = first_block_offset + descriptors_region_offset;

                let mut prev_abs_offset: u32 = first_block_offset + descriptors_region_offset;

                // Read remaining block descriptors one at a time from the descriptors region
                if num_blocks > 1 {
                    let mut descriptor_buf: Vec<u8> = vec![0u8; 4];
                    for i in 1..num_blocks as usize {
                        let deser_offset: u32 = 4 + ((i - 1) as u32 * 4);
                        let read_result = match &self.data_stream {
                            Some(stream) => match stream.write() {
                                Ok(mut ds) => ds.read_at_position(
                                    &mut descriptor_buf,
                                    SeekFrom::Start(deser_offset as u64),
                                ),
                                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                                    "Unable to obtain write lock on data stream",
                                    error
                                )),
                            },
                            None => {
                                Err(keramics_core::error_trace_new!("Data stream is not opened"))
                            }
                        };
                        match read_result {
                            Ok(read) => {
                                if read != 4 {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unable to read block descriptor"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read block descriptor"
                                );
                                return Err(error);
                            }
                        }
                        let raw_offset: u32 = Self::read_u32_le(&descriptor_buf, 0);
                        let abs: u32 = raw_offset + descriptors_region_offset;

                        if prev_abs_offset > abs
                            || (abs - prev_abs_offset) as usize > Self::BLOCK_SIZE + 1
                        {
                            return Err(keramics_core::error_trace_new!(
                                "Invalid compressed block offset"
                            ));
                        }
                        self.block_offsets[i] = abs;
                        prev_abs_offset = abs;
                    }
                }

                // Read compressed footer
                let compressed_footer_offset: u32 = Self::read_u32_be(&header_buf, 4);
                let compressed_footer_size: u32 = Self::read_u32_be(&header_buf, 12);

                if compressed_footer_size == 0 {
                    let last_offset = self.block_offsets.last().copied().unwrap_or(0) as u64;
                    if last_offset > self.compressed_size {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid compressed block offset"
                        ));
                    }
                    self.block_offsets.push(self.compressed_size as u32);
                    return Ok(());
                }

                if compressed_footer_size as usize > Self::BLOCK_SIZE + 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid compressed footer size"
                    ));
                }

                let mut footer_buf: Vec<u8> = vec![0u8; compressed_footer_size as usize];
                match &self.data_stream {
                    Some(stream) => {
                        let read_result = match stream.write() {
                            Ok(mut ds) => ds.read_at_position(
                                &mut footer_buf,
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
                    }
                    None => {
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
                let block_offset: u32 = Self::read_u32_le(&header_buf, 0);
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

                // Read block index data from the start of the stream
                let block_index_data_size: usize = (num_blocks as usize) * 4;
                let mut block_index_data: Vec<u8> = vec![0u8; block_index_data_size];
                match &self.data_stream {
                    Some(stream) => {
                        let read_result = match stream.write() {
                            Ok(mut ds) => {
                                ds.read_at_position(&mut block_index_data, SeekFrom::Start(0))
                            }
                            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                                "Unable to obtain write lock on data stream",
                                error
                            )),
                        };
                        match read_result {
                            Ok(read) => {
                                if read != block_index_data_size {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unable to read block index data"
                                    ));
                                }
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read block index data"
                                );
                                return Err(error);
                            }
                        }
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Data stream is not opened"));
                    }
                }

                for i in 1..num_blocks as usize {
                    let idx: usize = i * 4;
                    let next_offset: u32 = Self::read_u32_le(&block_index_data, idx);

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

    /// Helper to read a BE u32 from a buffer at a given offset.
    fn read_u32_be(buf: &[u8], offset: usize) -> u32 {
        bytes_to_u32_be!(buf, offset)
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
    fn get_test_data() -> Vec<u8> {
        return vec![
            0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe0, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x06,
        ];
    }

    fn get_lzvn_stream() -> Result<DecmpfsDataStream, ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut stream: DecmpfsDataStream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Lzvn);
        stream.open(&data_stream, 16)?;

        Ok(stream)
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut stream: DecmpfsDataStream = DecmpfsDataStream::new(DecmpfsCompressionMethod::Lzvn);

        stream.open(&data_stream, 16)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks

    #[test]
    fn test_read_compressed_data() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let mut data: Vec<u8> = vec![0; 16];
        stream.read(&mut data)?;

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let size: u64 = stream.get_size()?;
        assert_eq!(size, 16);

        Ok(())
    }

    // TODO: add tests for get_offset.

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let offset: u64 = stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let offset: u64 = stream.seek(SeekFrom::End(-8))?;
        assert_eq!(offset, 8);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let offset: u64 = stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let result: Result<u64, ErrorTrace> = stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        let offset: u64 = stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, 16 + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;
        stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut stream: DecmpfsDataStream = get_lzvn_stream()?;

        stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
