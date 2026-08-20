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
use crate::path_component::PathComponent;
use crate::traits::BlockReader;

use super::data_file_descriptor::LinuxLvmDataFileDescriptor;
use super::extent::{LinuxLvmExtent, LinuxLvmExtentValues};

/// Linux Logical Volume Manager (LVM) block reader.
pub struct LinuxLvmBlockReader {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Data file descriptors.
    data_file_descriptors: Vec<LinuxLvmDataFileDescriptor>,

    /// Extents.
    extents: Vec<LinuxLvmExtent>,

    /// The size.
    size: u64,
}

impl LinuxLvmBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        data_file_descriptors: &[LinuxLvmDataFileDescriptor],
        extents: &[LinuxLvmExtent],
        size: u64,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            data_file_descriptors: data_file_descriptors.to_vec(),
            extents: extents.to_vec(),
            size: size,
        }
    }
}

impl BlockReader for LinuxLvmBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the extents.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            let range_end_offset: u64 = extent.logical_offset + extent.size;

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < extent.logical_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let extent: &LinuxLvmExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {} for offset: {} (0x{:08x})",
                        extent_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - extent.logical_offset;
            let range_remainder_size: u64 = extent.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match &extent.values {
                LinuxLvmExtentValues::Stripe {
                    physical_offset,
                    physical_volume_index,
                } => {
                    // TODO: cache data streams
                    let data_file_descriptor: &LinuxLvmDataFileDescriptor =
                        match self.data_file_descriptors.get(*physical_volume_index) {
                            Some(data_file_descriptor) => data_file_descriptor,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing physical volume: {} data file descriptor",
                                    physical_volume_index,
                                )));
                            }
                        };

                    let path_components: [PathComponent; 1] =
                        [data_file_descriptor.file_name.clone()];

                    let data_stream: DataStreamReference = match self
                        .file_resolver
                        .get_data_stream(&path_components)
                    {
                        Ok(Some(data_stream)) => data_stream,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing data stream: {}",
                                data_file_descriptor.file_name
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open file: {}", data_file_descriptor.file_name)
                            );
                            return Err(error);
                        }
                    };

                    keramics_core::data_stream_read_exact_at_position!(
                        &data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(physical_offset + range_relative_offset)
                    );
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            extent_index += 1;
        }
        Ok(data_offset)
    }
}
