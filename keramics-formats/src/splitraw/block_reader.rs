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

use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::enums::SplitRawNamingSchema;
use super::segment_file::SplitRawSegmentFile;

/// Split raw storage media image block reader.
pub struct SplitRawBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Name.
    name: String,

    /// Segment file naming schema.
    naming_schema: SplitRawNamingSchema,

    /// Name first segment number.
    name_first_segment_number: u16,

    /// Name suffix size.
    name_suffix_size: usize,

    /// Number of segment files.
    number_of_segment_files: u16,

    /// Segment size.
    segment_size: u64,

    /// Segment file cache.
    segment_file_cache: LruCache<u16, DataStreamReference>,

    /// Size.
    size: u64,
}

impl SplitRawBlockReader {
    /// Creates a new block reader.
    pub fn new(
        file_resolver: &FileResolverReference,
        name: &str,
        naming_schema: &SplitRawNamingSchema,
        name_first_segment_number: u16,
        name_suffix_size: usize,
        number_of_segment_files: u16,
        segment_size: u64,
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            name: name.to_string(),
            naming_schema: naming_schema.clone(),
            name_first_segment_number,
            name_suffix_size,
            number_of_segment_files,
            segment_size,
            segment_file_cache: LruCache::new(16),
            size,
        }
    }
}

impl BlockReader for SplitRawBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let safe_segment_number: u64 = current_offset / self.segment_size;
        let segment_offset: u64 = safe_segment_number * self.segment_size;
        let mut range_relative_offset: u64 = current_offset - segment_offset;
        let mut range_remainder_size: u64 = self.segment_size - range_relative_offset;

        if safe_segment_number >= u16::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid segment number: {} value out of bounds",
                safe_segment_number
            )));
        }
        let mut segment_number: u16 = (safe_segment_number + 1) as u16;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            if !self.segment_file_cache.contains(&segment_number) {
                let segment_file_name: String = match SplitRawSegmentFile::get_file_name(
                    &self.name,
                    segment_number,
                    self.number_of_segment_files,
                    &self.naming_schema,
                    self.name_first_segment_number,
                    self.name_suffix_size,
                ) {
                    Ok(name) => name,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to determine file name of segment number: {}",
                                segment_number
                            )
                        );
                        return Err(error);
                    }
                };
                let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

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
                self.segment_file_cache.insert(segment_number, data_stream);
            }
            let data_stream: &DataStreamReference =
                match self.segment_file_cache.get(&segment_number) {
                    Some(file) => file,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve segment file: {} from cache",
                            segment_number
                        )));
                    }
                };
            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            let read_count: usize = keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(range_relative_offset)
            );
            if read_count == 0 {
                break;
            }
            data_offset += read_count;
            current_offset += read_count as u64;

            segment_number += 1;
            range_relative_offset = 0;
            range_remainder_size = self.segment_size;
        }
        Ok(data_offset)
    }
}
