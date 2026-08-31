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

use keramics_compression::{LzfseContext, LzvnContext};
use keramics_core::formatters::debug_format_array;
use keramics_core::{DataStreamReference, DebugTrace, ErrorTrace};
use keramics_types::{bytes_to_u32_be, bytes_to_u32_le};

use crate::traits::BlockReader;

use super::constants::*;
use super::enums::DecmpfsCompressionMethod;
use super::zlib_block_descriptor::DecmpfsZlibBlockDescriptor;
use super::zlib_footer::DecmpfsZlibFooter;
use super::zlib_header::DecmpfsZlibHeader;

use crate::lru_cache::LruCache;

/// Apple File System Compression (decmpfs) block reader.
pub struct DecmpfsBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Compression method.
    compression_method: DecmpfsCompressionMethod,

    /// The compressed size.
    compressed_size: u64,

    /// The size.
    size: u64,

    /// The block offsets.
    block_offsets: Vec<u32>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Uncompressed data marker.
    uncompressed_data_marker: Option<u8>,
}

impl DecmpfsBlockReader {
    const BLOCK_SIZE: usize = 65536;

    /// Creates a new compressed stream.
    pub(crate) fn new(
        data_stream: &DataStreamReference,
        compression_method: DecmpfsCompressionMethod,
    ) -> Self {
        let uncompressed_data_marker: Option<u8> = match compression_method {
            DecmpfsCompressionMethod::Lzfse | DecmpfsCompressionMethod::Zlib => Some(0xff),
            DecmpfsCompressionMethod::Lzvn => Some(0x06),
            DecmpfsCompressionMethod::Raw => Some(0xcc),
            _ => None,
        };
        Self {
            data_stream: data_stream.clone(),
            compression_method,
            compressed_size: 0,
            size: 0,
            block_offsets: Vec::new(),
            block_cache: LruCache::new(8),
            uncompressed_data_marker,
        }
    }

    /// Opens a block stream.
    pub(crate) fn open(&mut self, size: u64) -> Result<(), ErrorTrace> {
        self.compressed_size = match self.data_stream.write() {
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
        self.size = size;

        Ok(())
    }

    /// Reads and decompresses a compressed block.
    fn read_compressed_block(
        &self,
        block_offset: u64,
        block_size: usize,
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut compressed_data = vec![0; block_size];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut compressed_data,
            SeekFrom::Start(block_offset)
        );
        keramics_core::debug_trace_data!(
            "DecmpfsCompressedBlock",
            block_offset,
            &compressed_data,
            block_size
        );
        if let Some(uncompressed_data_marker) = self.uncompressed_data_marker {
            let mut compressed_data_size: usize = compressed_data.len();

            if compressed_data_size > 0 && compressed_data[0] == uncompressed_data_marker {
                compressed_data_size -= 1;

                data[0..compressed_data_size].copy_from_slice(&compressed_data[1..]);

                return Ok(());
            }
        }
        match self.compression_method {
            DecmpfsCompressionMethod::Lzfse => {
                let mut context = LzfseContext::new();

                match context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress LZFSE compressed block"
                        );
                        return Err(error);
                    }
                }
            }
            DecmpfsCompressionMethod::Lzvn => {
                let mut context = LzvnContext::new();

                match context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress LZVN compressed block"
                        );
                        return Err(error);
                    }
                }
            }
            DecmpfsCompressionMethod::Raw => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to decompress raw compressed block - unsupported marker byte"
                ));
            }
            DecmpfsCompressionMethod::Zlib => {
                _ = crate::zlib_decompress!(
                    &compressed_data,
                    data,
                    "Unable to decompress zlib compressed block"
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

    /// Reads the compressed block offsets.
    fn read_compressed_block_offsets(&mut self) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; 4];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        if &data == DECMPFS_HEADER_SIGNATURE {
            self.block_offsets = vec![16];

            return Ok(());
        }
        match self.compression_method {
            DecmpfsCompressionMethod::Lzfse
            | DecmpfsCompressionMethod::Lzvn
            | DecmpfsCompressionMethod::Raw => {
                let first_block_offset: u32 = bytes_to_u32_le!(&data, 0);

                if first_block_offset < 4
                    || (first_block_offset as u64) >= min(self.compressed_size, 65537)
                {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid block offset: 0 value out of bounds"
                    ));
                }
                let number_of_blocks: u32 = first_block_offset / 4;

                let mut block_descriptor_data: Vec<u8> = vec![0; (first_block_offset - 4) as usize];

                keramics_core::data_stream_read_exact_at_position!(
                    &self.data_stream,
                    &mut block_descriptor_data,
                    SeekFrom::Start(4)
                );
                keramics_core::debug_trace_data!(
                    "DecmpfsBlockDescriptors",
                    4,
                    &block_descriptor_data,
                    first_block_offset - 4,
                );
                self.block_offsets.push(first_block_offset);

                let mut data_offset: usize = 0;
                let mut last_block_offset: u32 = first_block_offset;

                for entry_index in 1..number_of_blocks {
                    let block_offset: u32 = bytes_to_u32_le!(&block_descriptor_data, data_offset);

                    data_offset += 4;

                    if block_offset < last_block_offset
                        || (block_offset as u64) > self.compressed_size
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid block descriptor: {} offset value out of bounds",
                            entry_index
                        )));
                    }
                    if block_offset - last_block_offset > 65537 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid block descriptor: {} size value out of bounds",
                            entry_index - 1
                        )));
                    }
                    self.block_offsets.push(block_offset);

                    last_block_offset = block_offset;
                }
                if self.compressed_size - (last_block_offset as u64) > 65537 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid block descriptor: {} size value out of bounds",
                        number_of_blocks - 1
                    )));
                }
                DebugTrace::static_scope(|debug_trace| {
                    debug_trace.print_start("DecmpfsLzvnBlockDescriptors");
                    debug_trace.print_field("number_of_blocks", number_of_blocks);
                    debug_trace.print_field(
                        "block_offsets",
                        debug_format_array(
                            self.block_offsets
                                .iter()
                                .map(|&element| element.to_string())
                                .collect::<Vec<String>>()
                                .as_slice(),
                        ),
                    );
                    debug_trace.print_end();
                });
                Ok(())
            }
            DecmpfsCompressionMethod::Zlib => {
                let signature: u32 = bytes_to_u32_be!(data, 0);

                if signature != 256 || self.compressed_size < 264 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported zlib compressed data"
                    ));
                }
                let mut zlib_header: DecmpfsZlibHeader = DecmpfsZlibHeader::new();

                match zlib_header.read_at_position(&self.data_stream, SeekFrom::Start(0)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read zlib header");
                        return Err(error);
                    }
                }
                if zlib_header.footer_offset < 264
                    || (zlib_header.footer_offset as u64) >= self.compressed_size
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid zlib footer offset: {} (0x{:08x}) value out bounds",
                        zlib_header.footer_offset, zlib_header.footer_offset
                    )));
                }
                if zlib_header.footer_size != 50 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported zlib footer size: {}",
                        zlib_header.footer_size
                    )));
                }
                if (zlib_header.footer_size as u64)
                    > (self.compressed_size - (zlib_header.footer_offset as u64))
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid zlib footer size: {} value out of bounds",
                        zlib_header.footer_size
                    )));
                }
                keramics_core::data_stream_read_exact_at_position!(
                    &self.data_stream,
                    &mut data,
                    SeekFrom::Start(260)
                );
                let number_of_blocks: u32 = bytes_to_u32_le!(&data, 0);

                if number_of_blocks > ((zlib_header.footer_offset - 264) / 8) {
                    return Err(keramics_core::error_trace_new!("Invalid number of blocks"));
                }
                let block_descriptor_data_size: usize = (number_of_blocks as usize) * 8;

                let mut block_descriptor_data: Vec<u8> = vec![0; block_descriptor_data_size];

                keramics_core::data_stream_read_exact_at_position!(
                    &self.data_stream,
                    &mut block_descriptor_data,
                    SeekFrom::Start(264)
                );
                keramics_core::debug_trace_data!(
                    "DecmpfsBlockDescriptors",
                    264,
                    &block_descriptor_data,
                    block_descriptor_data_size,
                );
                let mut data_offset: usize = 0;
                let mut next_block_offset: u32 = 8;

                for entry_index in 0..number_of_blocks {
                    keramics_core::debug_trace_structure!(
                        DecmpfsZlibBlockDescriptor::debug_read_data(
                            &block_descriptor_data[data_offset..]
                        )
                    );
                    let mut zlib_block_descriptor: DecmpfsZlibBlockDescriptor =
                        DecmpfsZlibBlockDescriptor::new();

                    match zlib_block_descriptor.read_data(&block_descriptor_data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read zlib block descriptor: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                    data_offset += 8;

                    if zlib_block_descriptor.offset < next_block_offset {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid block descriptor: {} offset value out of bounds",
                            entry_index
                        )));
                    }
                    if zlib_block_descriptor.size > 65537 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid block descriptor: {} size value out of bounds",
                            entry_index
                        )));
                    }
                    self.block_offsets.push(zlib_block_descriptor.offset + 260);

                    next_block_offset = zlib_block_descriptor.offset + zlib_block_descriptor.size;
                }
                if (next_block_offset as u64) > self.compressed_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid block descriptor: {} size value out of bounds",
                        number_of_blocks - 1
                    )));
                }
                if zlib_header.footer_offset < next_block_offset + 260 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid footer offset value out of bounds"
                    ));
                }
                let mut zlib_footer: DecmpfsZlibFooter = DecmpfsZlibFooter::new();

                match zlib_footer.read_at_position(
                    &self.data_stream,
                    SeekFrom::Start(zlib_header.footer_offset as u64),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read zlib footer");
                        return Err(error);
                    }
                }
                DebugTrace::static_scope(|debug_trace| {
                    debug_trace.print_start("DecmpfsZlibBlockDescriptors");
                    debug_trace.print_field("number_of_blocks", number_of_blocks);
                    debug_trace.print_field(
                        "block_offsets",
                        debug_format_array(
                            self.block_offsets
                                .iter()
                                .map(|&element| element.to_string())
                                .collect::<Vec<String>>()
                                .as_slice(),
                        ),
                    );
                    debug_trace.print_end();
                });
                Ok(())
            }
            _ => Err(keramics_core::error_trace_new!(
                "Unsupported compression method"
            )),
        }
    }
}

impl BlockReader for DecmpfsBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the compressed blocks.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        if self.size > 0 && self.block_offsets.is_empty() {
            match self.read_compressed_block_offsets() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read compressed block offsets"
                    );
                    return Err(error);
                }
            }
        }
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();
        let mut current_offset: u64 = offset;

        while data_offset < data_size {
            if current_offset >= self.size {
                break;
            }
            let read_count: usize = if self.compression_method == DecmpfsCompressionMethod::Unknown5
            {
                data[data_offset..data_size].fill(0);

                data_size
            } else {
                let block_index: u64 = current_offset / (Self::BLOCK_SIZE as u64);
                let block_offset: u64 = self.block_offsets[block_index as usize] as u64;

                let next_block_index: usize = (block_index as usize) + 1;
                let next_block_offset: u64 = if next_block_index < self.block_offsets.len() {
                    self.block_offsets[next_block_index] as u64
                } else {
                    self.compressed_size
                };
                let block_size: usize = (next_block_offset - block_offset) as usize;

                let range_offset: u64 = block_index * (Self::BLOCK_SIZE as u64);
                let range_size: u64 = min(Self::BLOCK_SIZE as u64, self.size - range_offset);
                let block_relative_offset: u64 = current_offset - range_offset;
                let block_remainder_size: u64 = range_size - block_relative_offset;

                let block_read_size: usize =
                    min(read_size - data_offset, block_remainder_size as usize);
                if block_read_size == 0 {
                    break;
                }
                let data_end_offset: usize = data_offset + block_read_size;

                if !self.block_cache.contains(&block_offset) {
                    let mut data: Vec<u8> = vec![0; Self::BLOCK_SIZE];

                    match self.read_compressed_block(block_offset, block_size, &mut data) {
                        Ok(size) => size,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read compressed block at offset: {} (0x{:08x})",
                                    block_offset, block_offset
                                )
                            );
                            return Err(error);
                        }
                    };
                    self.block_cache.insert(block_offset, data);
                }
                let block_data: &[u8] = match self.block_cache.get(&block_offset) {
                    Some(data) => data,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve data from cache"
                        ));
                    }
                };
                let block_data_offset: usize = block_relative_offset as usize;
                let block_data_end_offset: usize = block_data_offset + block_read_size;

                data[data_offset..data_end_offset]
                    .copy_from_slice(&block_data[block_data_offset..block_data_end_offset]);

                block_read_size
            };
            if read_count == 0 {
                break;
            }
            data_offset += read_count;
            current_offset += read_count as u64;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data_lzvn() -> Vec<u8> {
        vec![
            0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe0, 0x03, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73,
            0x73, 0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a, 0x06, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ]
    }

    fn get_test_data_zlib() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x6d, 0x70, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_lzvn();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(16)?;

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_with_lzfse() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x0b, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73,
            0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzfse);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 20, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x0b, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x62, 0x76, 0x78, 0x2d, 0x13, 0x00, 0x00, 0x00, 0x4d, 0x79, 0x20, 0x63,
            0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65,
            0x0a, 0x62, 0x76, 0x78, 0x24,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzfse);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 31, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_with_lzvn() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x06, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73,
            0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 20, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        let test_data: Vec<u8> = get_test_data_lzvn();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 29, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_with_raw() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x09, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xcc, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73,
            0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Raw);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 20, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x09, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x11, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73,
            0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Raw);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block(16, 20, &mut data);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_with_zlib() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x03, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73,
            0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 20, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        let test_data: Vec<u8> = vec![
            0x66, 0x70, 0x6d, 0x63, 0x03, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x78, 0x9c, 0xf3, 0xad, 0x54, 0x48, 0xce, 0xcf, 0x2d, 0x28, 0x4a, 0x2d,
            0x2e, 0x4e, 0x4d, 0x51, 0x48, 0xcb, 0xcc, 0x49, 0xe5, 0x02, 0x00, 0x47, 0x59, 0x06,
            0xe6,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);

        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        let mut data: Vec<u8> = vec![0; 32];
        block_reader.read_compressed_block(16, 27, &mut data)?;

        assert_eq!(&data[0..19], b"My compressed file\n");

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_offsets_with_lzvn() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc2, 0x14, 0x41, 0x44,
            0x00, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        assert_eq!(block_reader.block_offsets, vec![8, 16]);

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_zlib();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19)?;
        block_reader.read_compressed_block_offsets()?;

        assert_eq!(block_reader.block_offsets, vec![268]);

        Ok(())
    }

    #[test]
    fn test_read_compressed_block_offsets_with_lzvn_and_first_offset_out_of_bounds() {
        let test_data: Vec<u8> = vec![0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_lzvn_and_successive_block_offset_out_of_bounds() {
        let test_data: Vec<u8> = vec![
            0x08, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc2, 0x14, 0x41, 0x44,
            0x00, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_unsupported_header() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[0] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_invalid_number_of_block_descriptors() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[260] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_invalid_descriptor_offset() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[264] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_invalid_descriptor_size() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[268] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_invalid_footer_offset() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[4] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_compressed_block_offsets_with_zlib_and_invalid_footer_size() {
        let mut test_data: Vec<u8> = get_test_data_zlib();
        test_data[12] = 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Zlib);
        block_reader.open(19).unwrap();

        let result: Result<(), ErrorTrace> = block_reader.read_compressed_block_offsets();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_from_blocks() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_lzvn();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19)?;

        let mut data: Vec<u8> = vec![0; 19];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 0)?;

        assert_eq!(read_count, 19);
        assert_eq!(&data[0..19], b"My compressed file\n");

        let mut data: Vec<u8> = vec![0; 5];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 14)?;

        assert_eq!(read_count, 5);
        assert_eq!(&data[0..5], b"file\n");

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_beyond_size() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_lzvn();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19)?;

        let mut data: Vec<u8> = vec![0; 10];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 19)?;

        assert_eq!(read_count, 0);

        Ok(())
    }
}
