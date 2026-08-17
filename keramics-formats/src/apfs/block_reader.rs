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

use crate::traits::BlockReader;

use super::extent::ApfsExtent;

/// Apple File System (APFS) block reader.
pub struct ApfsBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Extents.
    extents: Vec<ApfsExtent>,

    /// The size.
    size: u64,
}

impl ApfsBlockReader {
    /// Creates a new block stream.
    pub(super) fn new(data_stream: &DataStreamReference, block_size: u32, size: u64) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            extents: Vec::new(),
            size,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(&mut self, extents: Vec<ApfsExtent>) -> Result<(), ErrorTrace> {
        self.extents = extents;

        Ok(())
    }
}

impl BlockReader for ApfsBlockReader {
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
            let extent_end_offset: u64 = extent.logical_offset + extent.size;

            if current_offset >= extent_end_offset {
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
                    "Missing extent for media offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let extent: &ApfsExtent = match self.extents.get(extent_index) {
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

            let range_physical_offset: u64 =
                (extent.physical_block_number as u64) * (self.block_size as u64);

            keramics_core::data_stream_read_exact_at_position!(
                &self.data_stream,
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(range_physical_offset + range_relative_offset)
            );
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            extent_index += 1;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut block_reader = ApfsBlockReader::new(&data_stream, 4096, 11358);

        let extents: Vec<ApfsExtent> = vec![ApfsExtent {
            logical_offset: 0,
            size: 12288,
            physical_block_number: 95,
            encryption_identifier: 0,
        }];
        block_reader.open(extents)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
}
