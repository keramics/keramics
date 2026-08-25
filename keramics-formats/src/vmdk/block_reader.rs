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
use keramics_types::ByteString;

use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::block_range::{VmdkBlockRange, VmdkBlockRangeType};
use super::descriptor_extent::VmdkDescriptorExtent;
use super::enums::VmdkDescriptorExtentType;
use super::extent_file::VmdkExtentFile;
use super::sparse_cowd_file::VmdkSparseCowdFile;
use super::sparse_file::VmdkSparseFile;

/// VMware Virtual Disk (VMDK) block reader.
pub struct VmdkBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Extents.
    extents: Vec<VmdkDescriptorExtent>,

    /// Extent file cache.
    extent_file_cache: LruCache<u64, VmdkExtentFile>,

    /// Decompressed grain cache.
    grain_cache: LruCache<u64, Vec<u8>>,

    /// Parent data stream.
    parent_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl VmdkBlockReader {
    /// Creates a new block reader.
    pub fn new(
        file_resolver: &FileResolverReference,
        bytes_per_sector: u16,
        extents: &[VmdkDescriptorExtent],
        parent_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            bytes_per_sector,
            extents: extents.to_vec(),
            extent_file_cache: LruCache::new(16),
            grain_cache: LruCache::new(64),
            parent_data_stream,
            size,
        }
    }
}

impl BlockReader for VmdkBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the extents.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let media_sector: u64 = current_offset / (self.bytes_per_sector as u64);

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            if media_sector >= extent.media_end_sector {
                Ordering::Less
            } else if media_sector < extent.media_start_sector {
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
        let mut extent: &VmdkDescriptorExtent = match self.extents.get(extent_index) {
            Some(extent) => extent,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve extent: {} for media offset: {} (0x{:08x})",
                    extent_index, current_offset, current_offset
                )));
            }
        };
        let extent_start_offset: u64 = extent.media_start_sector * (self.bytes_per_sector as u64);
        let mut extent_offset: u64 = current_offset - extent_start_offset;
        let mut extent_size: u64 = extent.number_of_sectors * (self.bytes_per_sector as u64);

        while data_offset < read_size {
            let extent_remainder_size: u64 = extent_size - extent_offset;
            let extent_read_size: usize =
                min(read_size - data_offset, extent_remainder_size as usize);

            let range_read_count: usize = match &extent.extent_type {
                VmdkDescriptorExtentType::Flat
                | VmdkDescriptorExtentType::Sparse
                | VmdkDescriptorExtentType::VmfsFlat
                | VmdkDescriptorExtentType::VmfsSparse => {
                    let lookup_extent_index: u64 = extent_index as u64;

                    if !self.extent_file_cache.contains(&lookup_extent_index) {
                        let extent_file_name: &ByteString = match extent.file_name.as_ref() {
                            Some(file_name) => file_name,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing extent file: {} name",
                                    extent_index
                                )));
                            }
                        };
                        let path_components: [PathComponent; 1] =
                            [PathComponent::from(extent_file_name)];
                        let data_stream: DataStreamReference =
                            match self.file_resolver.get_data_stream(&path_components) {
                                Ok(Some(data_stream)) => data_stream,
                                Ok(None) => {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Missing extent file: {}",
                                        extent_file_name
                                    )));
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!("Unable to open extent file: {}", extent_file_name)
                                    );
                                    return Err(error);
                                }
                            };
                        let extent_file: VmdkExtentFile = match &extent.extent_type {
                            VmdkDescriptorExtentType::Sparse => {
                                let mut sparse_file: VmdkSparseFile = VmdkSparseFile::new();

                                match sparse_file.read_data_stream(&data_stream) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to open sparse VMDK file"
                                        );
                                        return Err(error);
                                    }
                                }
                                VmdkExtentFile::SparseVmdk(sparse_file)
                            }
                            VmdkDescriptorExtentType::VmfsSparse => {
                                let mut sparse_file: VmdkSparseCowdFile = VmdkSparseCowdFile::new();

                                match sparse_file.read_data_stream(&data_stream) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to open sparse COWD file"
                                        );
                                        return Err(error);
                                    }
                                }
                                VmdkExtentFile::SparseCowd(sparse_file)
                            }
                            _ => VmdkExtentFile::Raw(data_stream),
                        };
                        self.extent_file_cache
                            .insert(lookup_extent_index, extent_file);
                    }
                    match self.extent_file_cache.get_mut(&lookup_extent_index) {
                        Some(VmdkExtentFile::Raw(data_stream)) => {
                            let data_end_offset: usize = data_offset + extent_read_size;

                            keramics_core::data_stream_read_at_position!(
                                data_stream,
                                &mut data[data_offset..data_end_offset],
                                SeekFrom::Start(extent_offset)
                            )
                        }
                        Some(VmdkExtentFile::SparseCowd(sparse_file)) => {
                            _ = sparse_file;
                            // TODO: read grain from sparse extent file or parent image
                            todo!();
                        }
                        Some(VmdkExtentFile::SparseVmdk(sparse_file)) => {
                            let mut result: Result<Option<&VmdkBlockRange>, ErrorTrace> =
                                sparse_file.block_tree.get_value(extent_offset);

                            if result == Ok(None) {
                                match sparse_file.read_grain_directory_entry(extent_offset) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to read grain directory entry"
                                        );
                                        return Err(error);
                                    }
                                }
                                result = sparse_file.block_tree.get_value(extent_offset);
                            }
                            let block_range: &VmdkBlockRange = match result {
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
                            let range_relative_offset: u64 =
                                extent_offset - block_range.extent_offset;
                            let range_remainder_size: u64 =
                                block_range.size - range_relative_offset;
                            let range_read_size: usize =
                                min(extent_read_size, range_remainder_size as usize);
                            let data_end_offset: usize = data_offset + range_read_size;

                            match block_range.range_type {
                                VmdkBlockRangeType::Compressed => {
                                    let grain_media_offset: u64 = (current_offset
                                        / sparse_file.grain_size)
                                        * sparse_file.grain_size;

                                    if !self.grain_cache.contains(&grain_media_offset) {
                                        let compressed_grain_offset: u64 = block_range.data_offset;

                                        let mut block_data: Vec<u8> =
                                            vec![0; sparse_file.grain_size as usize];

                                        match sparse_file.read_compressed_grain(
                                            compressed_grain_offset,
                                            &mut block_data,
                                        ) {
                                            Ok(_) => {}
                                            Err(mut error) => {
                                                keramics_core::error_trace_add_frame!(
                                                    error,
                                                    format!(
                                                        "Unable to read compressed grain from extent file: {} at offset: {} (0x{:08x})",
                                                        extent_index,
                                                        compressed_grain_offset,
                                                        compressed_grain_offset
                                                    )
                                                );
                                                return Err(error);
                                            }
                                        }
                                        self.grain_cache.insert(grain_media_offset, block_data);
                                    }
                                    let range_data: &[u8] =
                                        match self.grain_cache.get(&grain_media_offset) {
                                            Some(data) => data,
                                            None => {
                                                return Err(keramics_core::error_trace_new!(
                                                    "Unable to retrieve data from cache"
                                                ));
                                            }
                                        };
                                    let range_data_offset: usize = range_relative_offset as usize;
                                    let range_data_end_offset: usize =
                                        range_data_offset + range_read_size;

                                    data[data_offset..data_end_offset].copy_from_slice(
                                        &range_data[range_data_offset..range_data_end_offset],
                                    );

                                    range_read_size
                                }
                                VmdkBlockRangeType::InFile => {
                                    match sparse_file.data_stream.as_ref() {
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
                                    }
                                }
                                VmdkBlockRangeType::InParentOrSparse => {
                                    match &self.parent_data_stream {
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
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unable to retrieve extent file: {} from cache",
                                extent_index
                            )));
                        }
                    }
                }
                VmdkDescriptorExtentType::Zero => {
                    let data_end_offset: usize = data_offset + extent_read_size;

                    data[data_offset..data_end_offset].fill(0);

                    extent_read_size
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported extent type"
                    )));
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

                extent = match self.extents.get(extent_index) {
                    Some(extent) => extent,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing extent for offset: {} (0x{:08x})",
                            current_offset, current_offset
                        )));
                    }
                };
                extent_offset = 0;
                extent_size = extent.number_of_sectors * (self.bytes_per_sector as u64);
            }
        }
        Ok(data_offset)
    }
}
