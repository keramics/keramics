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

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential};
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::file::UdifFile;
use super::segment_range::UdifSegmentRange;

/// Universal Disk Image Format (UDIF) block reader.
pub struct UdifBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Name.
    name: String,

    /// Segment ranges.
    segment_ranges: Vec<UdifSegmentRange>,

    /// Segment file cache.
    segment_file_cache: LruCache<u32, UdifFile>,

    /// Credentials.
    credentials: Vec<CdsaEncrCredential>,

    /// The size.
    size: u64,
}

impl UdifBlockReader {
    /// Creates a new segment stream.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        name: &str,
        segment_ranges: &[UdifSegmentRange],
        credentials: &[CdsaEncrCredential],
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            name: name.to_string(),
            segment_ranges: segment_ranges.to_vec(),
            segment_file_cache: LruCache::new(16),
            credentials: credentials.to_vec(),
            size,
        }
    }

    /// Determines the segment file name.
    fn get_segment_file_name(name: &String, segment_number: u32) -> String {
        if segment_number == 1 {
            format!("{}.dmg", name)
        } else {
            format!("{}.{:03}.dmgpart", name, segment_number)
        }
    }

    /// Opens a segment file.
    fn open_segment_file(&self, segment_number: u32) -> Result<UdifFile, ErrorTrace> {
        let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

        let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

        let mut data_stream: DataStreamReference =
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
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to open encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            match cdsaencr_container.unlock(&self.credentials) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to unlock encrypted container of segment file: {}",
                        segment_file_name,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Failed to unlock encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            data_stream = match cdsaencr_container.get_data_stream() {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing encrypted container data stream",
                    ));
                }
            };
        }
        let mut segment_file: UdifFile = UdifFile::new();

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

        let mut segment_index: usize = match self.segment_ranges.binary_search_by(|segment_range| {
            if current_offset >= segment_range.end_offset {
                Ordering::Less
            } else if current_offset < segment_range.start_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing segment ragnage for segment offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let segment_range: &UdifSegmentRange = match self.segment_ranges.get(segment_index) {
                Some(segment_range) => segment_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment range for offset: {} (0x{:08x})",
                        current_offset, current_offset
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - segment_range.start_offset;
            let range_remainder_size: u64 = segment_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            if !self
                .segment_file_cache
                .contains(&segment_range.segment_number)
            {
                let segment_file: UdifFile =
                    match self.open_segment_file(segment_range.segment_number) {
                        Ok(udif_file) => udif_file,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to open segment file: {}",
                                    segment_range.segment_number
                                )
                            );
                            return Err(error);
                        }
                    };
                self.segment_file_cache
                    .insert(segment_range.segment_number, segment_file);
            }
            let segment_file: &mut UdifFile = match self
                .segment_file_cache
                .get_mut(&segment_range.segment_number)
            {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_range.segment_number
                    )));
                }
            };
            match segment_file
                .read_exact_at_position(&mut data[data_offset..data_end_offset], current_offset)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read from segment file: {} at offset: {} (0x{:08x})",
                            segment_range.segment_number, current_offset, current_offset
                        )
                    );
                    return Err(error);
                }
            }
            data_offset += range_read_size;
            current_offset += range_read_size as u64;
            segment_index += 1;
        }
        Ok(data_offset)
    }
}
