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

use crate::mediator::{Mediator, MediatorReference};
use crate::vfs::VfsDataStreamReference;

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
        data_stream: &VfsDataStreamReference,
        block_ranges: &mut Vec<ExtBlockRange>,
    ) -> io::Result<()> {
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
        data_stream: &VfsDataStreamReference,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> io::Result<()> {
        let data_size: usize = data.len();
        let mut extents_header: ExtExtentsHeader = ExtExtentsHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(ExtExtentsHeader::debug_read_data(&data[0..12]));
        }
        extents_header.read_data(&data[0..12])?;

        if extents_header.depth >= parent_depth {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid depth: {} value out of bounds",
                    extents_header.depth,
                ),
            ));
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
                    io::SeekFrom::Start(sub_node_offset),
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

                let range_type: ExtBlockRangeType = if entry.physical_block_number == 0 {
                    ExtBlockRangeType::Sparse
                } else {
                    ExtBlockRangeType::InFile
                };
                let block_range: ExtBlockRange = ExtBlockRange::new(
                    entry.logical_block_number as u64,
                    entry.physical_block_number,
                    entry.number_of_blocks as u64,
                    range_type,
                );
                block_ranges.push(block_range);

                *logical_block_number =
                    (entry.logical_block_number as u64) + (entry.number_of_blocks as u64);
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
        data_stream: &VfsDataStreamReference,
        position: io::SeekFrom,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> io::Result<()> {
        let mut data: Vec<u8> = vec![0; self.block_size as usize];

        let offset: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
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

// TODO: add tests.
