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

use crate::block_tree::BlockTree;
use crate::traits::BlockReader;

use super::block_range::HfsBlockRange;

/// Hierarchical File System (HFS) block reader.
pub struct HfsBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Block tree.
    block_tree: BlockTree<HfsBlockRange>,

    /// The size.
    size: u64,
}

impl HfsBlockReader {
    /// Creates a new block stream.
    pub(super) fn new(data_stream: &DataStreamReference, block_size: u32, size: u64) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            block_tree: BlockTree::<HfsBlockRange>::new(0, 0, 0),
            size,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(
        &mut self,
        number_of_blocks: u64,
        block_ranges: &[HfsBlockRange],
    ) -> Result<(), ErrorTrace> {
        let block_tree_data_size: u64 = number_of_blocks * (self.block_size as u64);
        self.block_tree =
            BlockTree::<HfsBlockRange>::new(block_tree_data_size, 0, self.block_size as u64);

        for (range_index, block_range) in block_ranges.iter().enumerate() {
            let range_logical_offset: u64 =
                (block_range.logical_block_number as u64) * (self.block_size as u64);

            if range_logical_offset >= block_tree_data_size {
                break;
            }
            let range_size: u64 = min(
                (block_range.number_of_blocks as u64) * (self.block_size as u64),
                block_tree_data_size - range_logical_offset,
            );
            match self.block_tree.insert_value(
                range_logical_offset,
                range_size,
                block_range.clone(),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to insert block range: {} [{} - {}] into block tree",
                            range_index,
                            range_logical_offset,
                            range_logical_offset + range_size
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}

impl BlockReader for HfsBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &HfsBlockRange = match self.block_tree.get_value(current_offset) {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {} (0x{:08x})",
                        current_offset, current_offset
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                            current_offset, current_offset
                        )
                    );
                    return Err(error);
                }
            };
            let range_logical_offset: u64 =
                (block_range.logical_block_number as u64) * (self.block_size as u64);
            let range_size: u64 = (block_range.number_of_blocks as u64) * (self.block_size as u64);

            let range_relative_offset: u64 = current_offset - range_logical_offset;
            let range_remainder_size: u64 = range_size - range_relative_offset;

            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            let range_physical_offset: u64 =
                (block_range.physical_block_number as u64) * (self.block_size as u64);

            let read_count: usize = keramics_core::data_stream_read_at_position!(
                &self.data_stream,
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(range_physical_offset + range_relative_offset)
            );
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

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut block_reader = HfsBlockReader::new(&data_stream, 4096, 11358);

        let block_ranges: Vec<HfsBlockRange> = vec![HfsBlockRange {
            logical_block_number: 0,
            physical_block_number: 275,
            number_of_blocks: 3,
        }];
        block_reader.open(26, &block_ranges)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
}
