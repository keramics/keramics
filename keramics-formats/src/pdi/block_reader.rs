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

use super::block_range::{PdiBlockRange, PdiBlockRangeType};
use super::enums::PdiExtentType;
use super::extent_file::PdiExtentFile;
use super::image_extent::PdiImageExtent;
use super::sparse_file::PdiSparseFile;

/// Parallels Disk Image (PDI) block reader.
pub struct PdiBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Extents.
    extents: Vec<PdiImageExtent>,

    /// Extent file cache.
    extent_file_cache: LruCache<u64, PdiExtentFile>,

    /// Parent layer.
    parent_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl PdiBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        extents: &[PdiImageExtent],
        parent_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            extents: extents.to_vec(),
            extent_file_cache: LruCache::new(16),
            parent_data_stream,
            size,
        }
    }

    /// Retrieves a specific extent file.
    fn get_extent_file(&mut self, extent_index: usize) -> Result<&mut PdiExtentFile, ErrorTrace> {
        let lookup_extent_index: u64 = extent_index as u64;

        if !self.extent_file_cache.contains(&lookup_extent_index) {
            let extent: &PdiImageExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {}",
                        extent_index
                    )));
                }
            };
            let path_components: [PathComponent; 1] =
                [PathComponent::from(extent.file_name.as_str())];

            let data_stream: DataStreamReference =
                match self.file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing extent file: {}",
                            extent.file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open extent file: {}", extent.file_name)
                        );
                        return Err(error);
                    }
                };
            let extent_file: PdiExtentFile = match &extent.extent_type {
                PdiExtentType::Sparse => {
                    let mut sparse_file: PdiSparseFile = PdiSparseFile::new();

                    match sparse_file.read_data_stream(&data_stream) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open sparse extent file: {}", extent.file_name)
                            );
                            return Err(error);
                        }
                    }
                    PdiExtentFile::Sparse(sparse_file)
                }
                PdiExtentType::Raw => PdiExtentFile::Raw(data_stream),
            };
            self.extent_file_cache
                .insert(lookup_extent_index, extent_file);
        }
        match self.extent_file_cache.get_mut(&lookup_extent_index) {
            Some(extent_file) => Ok(extent_file),
            None => Err(keramics_core::error_trace_new!(format!(
                "Unable to retrieve extent: {} from cache",
                extent_index
            ))),
        }
    }
}

impl BlockReader for PdiBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            if current_offset >= extent.end_offset {
                Ordering::Less
            } else if current_offset < extent.start_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for media offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        let extent: &PdiImageExtent = match self.extents.get(extent_index) {
            Some(extent) => extent,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve extent: {} for media offset: {} (0x{:08x})",
                    extent_index, current_offset, current_offset
                )));
            }
        };
        let mut extent_offset: u64 = current_offset - extent.start_offset;
        let mut extent_size: u64 = extent.size;

        while data_offset < read_size {
            let extent_file: &mut PdiExtentFile = match self.get_extent_file(extent_index) {
                Ok(extent_file) => extent_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve extent file: {}", extent_index)
                    );
                    return Err(error);
                }
            };
            let extent_remainder_size: u64 = extent_size - extent_offset;
            let extent_read_size: usize =
                min(read_size - data_offset, extent_remainder_size as usize);

            let range_read_count: usize = match extent_file {
                PdiExtentFile::Raw(data_stream) => {
                    let data_end_offset: usize = data_offset + extent_read_size;

                    keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(extent_offset)
                    )
                }
                PdiExtentFile::Sparse(sparse_file) => {
                    let mut result: Result<Option<&PdiBlockRange>, ErrorTrace> =
                        sparse_file.block_tree.get_value(extent_offset);

                    if result == Ok(None) {
                        match sparse_file.read_block_allocation_table_entry(extent_offset) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read block allocation table entry"
                                );
                                return Err(error);
                            }
                        }
                        result = sparse_file.block_tree.get_value(extent_offset);
                    }
                    let block_range: &PdiBlockRange = match result {
                        Ok(Some(block_range)) => block_range,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing block range for offset: {} (0x{:08x})",
                                extent_offset, extent_offset
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve block range for offset: {} (0x{:08x})",
                                    extent_offset, extent_offset
                                )
                            );
                            return Err(error);
                        }
                    };
                    let range_relative_offset: u64 = extent_offset - block_range.extent_offset;
                    let range_remainder_size: u64 = block_range.size - range_relative_offset;
                    let range_read_size: usize =
                        min(extent_read_size, range_remainder_size as usize);
                    let data_end_offset: usize = data_offset + range_read_size;

                    match block_range.range_type {
                        PdiBlockRangeType::InFile => match sparse_file.data_stream.as_ref() {
                            Some(data_stream) => {
                                keramics_core::data_stream_read_at_position!(
                                    data_stream,
                                    &mut data[data_offset..data_end_offset],
                                    SeekFrom::Start(
                                        block_range.data_offset + range_relative_offset
                                    )
                                )
                            }
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing extent file: {} data stream",
                                    extent_index
                                )));
                            }
                        },
                        PdiBlockRangeType::InParentOrSparse => {
                            match self.parent_data_stream.as_ref() {
                                Some(parent_data_stream) => {
                                    keramics_core::data_stream_read_at_position!(
                                        parent_data_stream,
                                        &mut data[data_offset..data_end_offset],
                                        SeekFrom::Start(current_offset)
                                    )
                                }
                                None => {
                                    data[data_offset..data_end_offset].fill(0);

                                    range_read_size
                                }
                            }
                        }
                    }
                }
            };
            data_offset += range_read_count;
            extent_offset += range_read_count as u64;
            current_offset += range_read_count as u64;

            if current_offset >= self.size {
                break;
            }
            if extent_offset >= extent_size {
                extent_index += 1;

                extent_offset = 0;
                extent_size = match self.extents.get(extent_index) {
                    Some(extent) => extent.size,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve extent: {}",
                            extent_index
                        )));
                    }
                };
            }
        }
        Ok(data_offset)
    }
}
