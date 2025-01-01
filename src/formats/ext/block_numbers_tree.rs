/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;

use crate::bytes_to_u32_le;
use crate::mediator::{Mediator, MediatorReference};
use crate::vfs::VfsDataStreamReference;

use super::block_range::{ExtBlockRange, ExtBlockRangeType};

/// Extended File System (ext) block numbers tree.
pub struct ExtBlockNumbersTree {
    /// Mediator.
    mediator: MediatorReference,

    /// Block size.
    block_size: u32,

    /// Number of blocks.
    number_of_blocks: u64,
}

impl ExtBlockNumbersTree {
    /// Creates new block numbers tree.
    pub fn new(block_size: u32, number_of_blocks: u64) -> Self {
        Self {
            mediator: Mediator::current(),
            block_size: block_size,
            number_of_blocks: number_of_blocks,
        }
    }

    /// Reads the block numbers tree from an inode data reference.
    pub fn read_data_reference(
        &mut self,
        data_reference: &[u8],
        data_stream: &VfsDataStreamReference,
        block_ranges: &mut Vec<ExtBlockRange>,
    ) -> io::Result<()> {
        let mut logical_block_number: u64 = 0;

        self.read_node_data(
            &data_reference[0..48],
            data_stream,
            &mut logical_block_number,
            block_ranges,
            0,
        )?;
        let block_number: u32 = bytes_to_u32_le!(data_reference, 48);
        if block_number != 0 {
            let sub_node_offset: u64 = (block_number as u64) * (self.block_size as u64);

            self.read_node_at_position(
                data_stream,
                io::SeekFrom::Start(sub_node_offset),
                &mut logical_block_number,
                block_ranges,
                1,
            )?;
        }
        let block_number: u32 = bytes_to_u32_le!(data_reference, 52);
        if block_number != 0 {
            let sub_node_offset: u64 = (block_number as u64) * (self.block_size as u64);

            self.read_node_at_position(
                data_stream,
                io::SeekFrom::Start(sub_node_offset),
                &mut logical_block_number,
                block_ranges,
                2,
            )?;
        }
        let block_number: u32 = bytes_to_u32_le!(data_reference, 56);
        if block_number != 0 {
            let sub_node_offset: u64 = (block_number as u64) * (self.block_size as u64);

            self.read_node_at_position(
                data_stream,
                io::SeekFrom::Start(sub_node_offset),
                &mut logical_block_number,
                block_ranges,
                3,
            )?;
        }
        Ok(())
    }

    /// Reads the block numbers tree node from a buffer.
    fn read_node_data(
        &mut self,
        data: &[u8],
        data_stream: &VfsDataStreamReference,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        depth: u16,
    ) -> io::Result<()> {
        let data_size: usize = data.len();

        let number_of_entries: usize = data_size / 4;

        let mut data_offset: usize = 0;

        if depth > 0 {
            for _ in 0..number_of_entries {
                if *logical_block_number >= self.number_of_blocks {
                    break;
                }
                let block_number: u32 = bytes_to_u32_le!(data, data_offset);
                data_offset += 4;

                if block_number != 0 {
                    let sub_node_offset: u64 = (block_number as u64) * (self.block_size as u64);

                    self.read_node_at_position(
                        data_stream,
                        io::SeekFrom::Start(sub_node_offset),
                        logical_block_number,
                        block_ranges,
                        depth - 1,
                    )?;
                }
            }
        } else {
            let mut number_of_ranges: usize = block_ranges.len();

            let mut range_block_number: u64 = 0;
            let mut range_number_of_blocks: u64 = 0;
            if number_of_ranges > 0 {
                let last_block_range: &ExtBlockRange = &block_ranges[number_of_ranges - 1];
                range_block_number =
                    last_block_range.physical_block_number + last_block_range.number_of_blocks;
                range_number_of_blocks = last_block_range.number_of_blocks;
            }
            for _ in 0..number_of_entries {
                if *logical_block_number >= self.number_of_blocks {
                    break;
                }
                let block_number: u32 = bytes_to_u32_le!(data, data_offset);
                data_offset += 4;

                if number_of_ranges == 0 || (block_number as u64) != range_block_number {
                    if number_of_ranges > 0 {
                        let last_block_range: &mut ExtBlockRange =
                            &mut block_ranges[number_of_ranges - 1];
                        last_block_range.number_of_blocks = range_number_of_blocks;
                    }
                    range_block_number = block_number as u64;
                    range_number_of_blocks = 0;

                    let range_type: ExtBlockRangeType = if block_number == 0 {
                        ExtBlockRangeType::Sparse
                    } else {
                        ExtBlockRangeType::InFile
                    };
                    let block_range: ExtBlockRange = ExtBlockRange::new(
                        *logical_block_number,
                        range_block_number,
                        1,
                        range_type,
                    );
                    block_ranges.push(block_range);
                    number_of_ranges += 1;
                }
                *logical_block_number += 1;

                if block_number > 0 {
                    range_block_number += 1;
                }
                range_number_of_blocks += 1;
            }
            if number_of_ranges > 0 {
                let last_block_range: &mut ExtBlockRange = &mut block_ranges[number_of_ranges - 1];
                last_block_range.number_of_blocks = range_number_of_blocks;
            }
        }
        Ok(())
    }

    /// Reads the block numbers tree node from a specific position in a data stream.
    fn read_node_at_position(
        &mut self,
        data_stream: &VfsDataStreamReference,
        position: io::SeekFrom,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        depth: u16,
    ) -> io::Result<()> {
        let mut data: Vec<u8> = vec![0; self.block_size as usize];

        let offset: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "ExtBlockNumbersTreeNode data of size: {} at offset: {} (0x{:08x})\n",
                self.block_size, offset, offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.read_node_data(
            &data,
            data_stream,
            logical_block_number,
            block_ranges,
            depth,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::SharedValue;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data_reference() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtBlockNumbersTree::new(1024, 2);
        let test_data_stream: VfsDataStreamReference = SharedValue::none();
        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();
        test_struct.read_data_reference(&test_data, &test_data_stream, &mut block_ranges)?;

        assert_eq!(block_ranges.len(), 2);

        Ok(())
    }

    // TODO: add tests for read_node_data
    // TODO: add tests for read_node_at_position
}
