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
use std::collections::HashSet;
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::BlockReader;

use super::block_allocation_table::FatBlockAllocationTable;
use super::block_range::FatBlockRange;

/// File Allocation Table (FAT) block stream.
pub struct FatBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Block ranges.
    block_ranges: Vec<FatBlockRange>,

    /// The size.
    size: u64,
}

impl FatBlockReader {
    /// Creates a new block stream.
    pub(super) fn new(data_stream: &DataStreamReference, block_size: u32, size: u32) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            block_ranges: Vec::new(),
            size: size as u64,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(
        &mut self,
        block_allocation_table: &Arc<FatBlockAllocationTable>,
        mut cluster_block_number: u32,
    ) -> Result<(), ErrorTrace> {
        let largest_cluster_block_number: u32 =
            block_allocation_table.get_largest_cluster_block_number();

        let mut read_cluster_block_numbers: HashSet<u32> = HashSet::new();
        let mut logical_offset: u64 = 0;
        let mut next_physical_offset: u64 = 0;
        let mut range_logical_offset: u64 = 0;
        let mut range_physical_offset: u64 = 0;
        let mut range_size: u32 = 0;

        while cluster_block_number >= 2 && cluster_block_number < largest_cluster_block_number {
            if read_cluster_block_numbers.contains(&cluster_block_number) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Cluster block: {} already read",
                    cluster_block_number
                )));
            }
            let physical_offset: u64 = block_allocation_table.first_cluster_offset
                + (((cluster_block_number - 2) as u64)
                    * (block_allocation_table.cluster_block_size as u64));

            if logical_offset == 0 {
                range_physical_offset = physical_offset;
            } else if physical_offset != next_physical_offset {
                let block_range: FatBlockRange =
                    FatBlockRange::new(range_logical_offset, range_physical_offset, range_size);

                self.block_ranges.push(block_range);

                range_logical_offset = logical_offset;
                range_physical_offset = physical_offset;
                range_size = 0;
            }
            logical_offset += self.block_size as u64;
            range_size += self.block_size;

            next_physical_offset = physical_offset + (self.block_size as u64);

            read_cluster_block_numbers.insert(cluster_block_number);

            cluster_block_number =
                match block_allocation_table.read_entry(&self.data_stream, cluster_block_number) {
                    Ok(entry) => entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read next cluster block number from block allocation table"
                        );
                        return Err(error);
                    }
                };
        }
        let block_range: FatBlockRange =
            FatBlockRange::new(range_logical_offset, range_physical_offset, range_size);

        self.block_ranges.push(block_range);

        Ok(())
    }
}

impl BlockReader for FatBlockReader {
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
            let range_end_offset: u64 = block_range.logical_offset + (block_range.size as u64);

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < block_range.logical_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(range_index) => range_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing block range for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &FatBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.logical_offset;
            let range_remainder_size: u64 = (block_range.size as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            let data_end_offset: usize = data_offset + range_read_size;

            keramics_core::data_stream_read_exact_at_position!(
                &self.data_stream,
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(block_range.physical_offset + range_relative_offset)
            );
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            range_index += 1;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::fat::block_allocation_table::FatBlockAllocationTable;
    use crate::fat::enums::FatFormat;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("fat/fat12.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut block_reader = FatBlockReader::new(&data_stream, 2048, 11358);

        let block_allocation_table: Arc<FatBlockAllocationTable> = Arc::new(
            FatBlockAllocationTable::new(&FatFormat::Fat12, 512, 244, 23040, 2048),
        );
        block_reader.open(&block_allocation_table, 5)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
}
