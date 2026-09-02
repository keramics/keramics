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

use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::block_range::{EwfBlockRange, EwfBlockRangeType};
use super::enums::EwfNamingSchema;
use super::file::EwfFile;
use super::segment_file::EwfSegmentFile;

/// Expert Witness Compression Format (EWF) block reader.
pub struct EwfBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Name.
    name: String,

    /// Segment file naming schema.
    naming_schema: Option<EwfNamingSchema>,

    /// Segment file cache.
    segment_file_cache: LruCache<u16, EwfFile>,

    /// Chunk size.
    chunk_size: u32,

    /// Block ranges.
    block_ranges: Vec<EwfBlockRange>,

    /// Decompressed chunk cache.
    chunk_cache: LruCache<u64, Vec<u8>>,

    /// Size.
    size: u64,
}

impl EwfBlockReader {
    /// Creates a new block reader.
    pub fn new(
        file_resolver: &FileResolverReference,
        name: &str,
        naming_schema: Option<&EwfNamingSchema>,
        chunk_size: u32,
        block_ranges: &[EwfBlockRange],
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            name: name.to_string(),
            naming_schema: naming_schema.cloned(),
            segment_file_cache: LruCache::new(16),
            chunk_size,
            block_ranges: block_ranges.to_vec(),
            chunk_cache: LruCache::new(64),
            size,
        }
    }

    /// Opens a segment file.
    fn open_segment_file(&self, segment_file_name: &String) -> Result<EwfFile, ErrorTrace> {
        let path_components: [PathComponent; 1] = [PathComponent::from(segment_file_name)];

        let data_stream: DataStreamReference =
            match self.file_resolver.get_data_stream(&path_components) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing segment file: {}",
                        segment_file_name
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
        let mut segment_file: EwfFile = EwfFile::new();

        match segment_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read segment file: {}", segment_file_name)
                );
                return Err(error);
            }
        }
        Ok(segment_file)
    }
}

impl BlockReader for EwfBlockReader {
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
            let media_end_offset: u64 = block_range.media_offset + (self.chunk_size as u64);

            if current_offset >= media_end_offset {
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
                    "Missing block range for media offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &EwfBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            if !self
                .segment_file_cache
                .contains(&block_range.segment_number)
            {
                let segment_file_name: String = match EwfSegmentFile::get_file_name(
                    &self.name,
                    block_range.segment_number,
                    self.naming_schema.as_ref(),
                ) {
                    Ok(name) => name,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to determine file name of segment number: {}",
                                block_range.segment_number
                            )
                        );
                        return Err(error);
                    }
                };
                let segment_file: EwfFile = match self.open_segment_file(&segment_file_name) {
                    Ok(ewf_file) => ewf_file,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open segment file: {}", segment_file_name)
                        );
                        return Err(error);
                    }
                };
                self.segment_file_cache
                    .insert(block_range.segment_number, segment_file);
            }
            let segment_file: &mut EwfFile =
                match self.segment_file_cache.get_mut(&block_range.segment_number) {
                    Some(file) => file,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve segment file: {} from cache",
                            block_range.segment_number
                        )));
                    }
                };
            let range_relative_offset: u64 = current_offset - block_range.media_offset;
            let range_remainder_size: u64 = (self.chunk_size as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match block_range.range_type {
                EwfBlockRangeType::Compressed => {
                    let chunk_media_offset: u64 =
                        (current_offset / (self.chunk_size as u64)) * (self.chunk_size as u64);

                    if !self.chunk_cache.contains(&chunk_media_offset) {
                        let compressed_chunk_offset: u64 = block_range.data_offset;

                        let mut block_data: Vec<u8> = vec![0; self.chunk_size as usize];

                        match segment_file.read_compressed_chunk(
                            compressed_chunk_offset,
                            block_range.data_size,
                            &mut block_data,
                        ) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to read compressed chunk from segment file: {} at offset: {} (0x{:08x})",
                                        block_range.segment_number,
                                        compressed_chunk_offset,
                                        compressed_chunk_offset,
                                    )
                                );
                                return Err(error);
                            }
                        }
                        self.chunk_cache.insert(chunk_media_offset, block_data);
                    }
                    let range_data: &[u8] = match self.chunk_cache.get(&chunk_media_offset) {
                        Some(data) => data,
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to retrieve data from cache"
                            ));
                        }
                    };
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);
                }
                EwfBlockRangeType::Corrupt => {
                    data[data_offset..data_end_offset].fill(0);
                }
                EwfBlockRangeType::InFile => {
                    let chunk_data_offset: u64 = block_range.data_offset + range_relative_offset;

                    match segment_file.read_exact_at_position(
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(chunk_data_offset),
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read uncompressed chunk from segment file: {} at offset: {} (0x{:08x})",
                                    block_range.segment_number,
                                    chunk_data_offset,
                                    chunk_data_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                    // TODO: read full chunk and calculate and compare checksum
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            range_index += 1;
        }
        Ok(data_offset)
    }
}
