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

use std::io::SeekFrom;

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};

use super::block_range::{ExtBlockRange, ExtBlockRangeType};
use super::extent_descriptor::ExtExtentDescriptor;
use super::extent_index::ExtExtentIndex;
use super::extents_footer::ExtExtentsFooter;
use super::extents_header::ExtExtentsHeader;

/// Extended File System (ext) extents tree.
pub struct ExtExtentsTree {
    /// Mediator.
    mediator: MediatorReference,

    /// Block size.
    block_size: u32,

    /// Number of blocks.
    number_of_blocks: u64,
}

impl ExtExtentsTree {
    /// Creates new extents tree.
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
        data_stream: &DataStreamReference,
        block_ranges: &mut Vec<ExtBlockRange>,
    ) -> Result<(), ErrorTrace> {
        let mut logical_block_number: u64 = 0;

        self.read_node_data(
            data_reference,
            data_stream,
            &mut logical_block_number,
            block_ranges,
            6,
        )?;
        if logical_block_number < self.number_of_blocks {
            let block_range: ExtBlockRange = ExtBlockRange::new(
                logical_block_number,
                0,
                self.number_of_blocks - logical_block_number,
                ExtBlockRangeType::Sparse,
            );
            block_ranges.push(block_range);
        }
        Ok(())
    }

    /// Reads the extents tree node from a buffer.
    fn read_node_data(
        &mut self,
        data: &[u8],
        data_stream: &DataStreamReference,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();
        let mut extents_header: ExtExtentsHeader = ExtExtentsHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(ExtExtentsHeader::debug_read_data(&data[0..12]));
        }
        extents_header.read_data(&data[0..12])?;

        if extents_header.depth >= parent_depth {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid depth: {} value out of bounds",
                extents_header.depth,
            )));
        }
        let mut data_offset: usize = 12;

        if extents_header.depth > 0 {
            for _ in 0..extents_header.number_of_entries {
                let data_end_offset: usize = data_offset + 12;

                let mut entry: ExtExtentIndex = ExtExtentIndex::new();

                if self.mediator.debug_output {
                    self.mediator.debug_print(ExtExtentIndex::debug_read_data(
                        &data[data_offset..data_end_offset],
                    ));
                }
                entry.read_data(&data[data_offset..data_end_offset])?;
                data_offset = data_end_offset;

                let sub_node_offset: u64 = entry.physical_block_number * (self.block_size as u64);

                self.read_node_at_position(
                    data_stream,
                    SeekFrom::Start(sub_node_offset),
                    logical_block_number,
                    block_ranges,
                    extents_header.depth,
                )?;
            }
        } else {
            for _ in 0..extents_header.number_of_entries {
                let data_end_offset: usize = data_offset + 12;

                let mut entry: ExtExtentDescriptor = ExtExtentDescriptor::new();

                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(ExtExtentDescriptor::debug_read_data(
                            &data[data_offset..data_end_offset],
                        ));
                }
                entry.read_data(&data[data_offset..data_end_offset])?;
                data_offset = data_end_offset;

                if entry.logical_block_number as u64 > *logical_block_number {
                    let number_of_blocks: u64 =
                        (entry.logical_block_number as u64) - *logical_block_number;

                    let block_range: ExtBlockRange = ExtBlockRange::new(
                        *logical_block_number,
                        0,
                        number_of_blocks,
                        ExtBlockRangeType::Sparse,
                    );
                    block_ranges.push(block_range);

                    *logical_block_number = entry.logical_block_number as u64;
                }
                let mut number_of_blocks: u64 = entry.number_of_blocks as u64;
                let mut range_type: ExtBlockRangeType = if entry.physical_block_number == 0 {
                    ExtBlockRangeType::Sparse
                } else {
                    ExtBlockRangeType::InFile
                };
                if number_of_blocks > 32768 {
                    number_of_blocks -= 32768;
                    range_type = ExtBlockRangeType::Sparse;
                }
                let block_range: ExtBlockRange = ExtBlockRange::new(
                    entry.logical_block_number as u64,
                    entry.physical_block_number,
                    number_of_blocks,
                    range_type,
                );
                block_ranges.push(block_range);

                *logical_block_number = (entry.logical_block_number as u64) + number_of_blocks;
            }
        }
        if data_size - data_offset >= 4 {
            let mut extents_footer: ExtExtentsFooter = ExtExtentsFooter::new();

            let data_end_offset: usize = data_offset + 4;
            if self.mediator.debug_output {
                self.mediator.debug_print(ExtExtentsFooter::debug_read_data(
                    &data[data_offset..data_end_offset],
                ));
            }
            extents_footer.read_data(&data[data_offset..data_end_offset])?;
        }
        Ok(())
    }

    /// Reads the extents tree node from a specific position in a data stream.
    fn read_node_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; self.block_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "ExtExtentsTreeNode data of size: {} at offset: {} (0x{:08x})\n",
                self.block_size, offset, offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.read_node_data(
            &data,
            data_stream,
            logical_block_number,
            block_ranges,
            parent_depth,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![];
    }

    #[test]
    fn test_read_data_reference() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtExtentsTree::new(1024, 16);

        let test_data: Vec<u8> = get_test_data();
        let test_data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();

        let test_data: Vec<u8> = vec![
            0x0a, 0xf3, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x53, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        test_struct.read_data_reference(&test_data, &test_data_stream, &mut block_ranges)?;

        assert_eq!(block_ranges.len(), 2);

        let block_range: &ExtBlockRange = &block_ranges[0];
        assert_eq!(block_range.logical_block_number, 0);
        assert_eq!(block_range.physical_block_number, 1363);
        assert_eq!(block_range.number_of_blocks, 12);
        assert!(block_range.range_type == ExtBlockRangeType::InFile);

        let block_range: &ExtBlockRange = &block_ranges[1];
        assert_eq!(block_range.logical_block_number, 12);
        assert_eq!(block_range.physical_block_number, 0);
        assert_eq!(block_range.number_of_blocks, 4);
        assert!(block_range.range_type == ExtBlockRangeType::Sparse);

        Ok(())
    }

    #[test]
    fn test_read_node_data() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtExtentsTree::new(1024, 16);

        let test_data: Vec<u8> = get_test_data();
        let test_data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut logical_block_number: u64 = 0;
        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();

        let test_data: Vec<u8> = vec![
            0x0a, 0xf3, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x53, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        test_struct.read_node_data(
            &test_data,
            &test_data_stream,
            &mut logical_block_number,
            &mut block_ranges,
            6,
        )?;
        assert_eq!(block_ranges.len(), 1);

        let block_range: &ExtBlockRange = &block_ranges[0];
        assert_eq!(block_range.logical_block_number, 0);
        assert_eq!(block_range.physical_block_number, 1363);
        assert_eq!(block_range.number_of_blocks, 12);
        assert!(block_range.range_type == ExtBlockRangeType::InFile);

        Ok(())
    }

    // TODO: add tests for read_node_data with depth > 0
    // TODO: add tests for read_node_at_position
}
