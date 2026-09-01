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

use super::block_range::{ExtBlockRange, ExtBlockRangeType};

/// Extended File System (ext) block reader.
pub struct ExtBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Block ranges.
    block_ranges: Vec<ExtBlockRange>,

    /// The size.
    size: u64,
}

impl ExtBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        block_size: u32,
        block_ranges: &[ExtBlockRange],
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            block_ranges: block_ranges.to_vec(),
            size,
        }
    }
}

impl BlockReader for ExtBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let block_number: u64 = current_offset / (self.block_size as u64);

        let mut range_index: usize = match self.block_ranges.binary_search_by(|block_range| {
            let range_end_block_number: u64 =
                block_range.logical_block_number + block_range.number_of_blocks;

            if block_number >= range_end_block_number {
                Ordering::Less
            } else if block_number < block_range.logical_block_number {
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
            let block_range: &ExtBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 =
                current_offset - (block_range.logical_block_number * (self.block_size as u64));
            let range_remainder_size: u64 =
                (block_range.number_of_blocks * (self.block_size as u64)) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match block_range.range_type {
                ExtBlockRangeType::InFile => {
                    let range_physical_offset: u64 =
                        block_range.physical_block_number * (self.block_size as u64);

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_physical_offset + range_relative_offset)
                    );
                }
                ExtBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            range_index += 1;
        }
        Ok(data_offset)
    }
}
