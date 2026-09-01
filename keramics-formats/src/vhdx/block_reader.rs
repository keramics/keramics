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

use super::block_allocation_table::{VhdxBlockAllocationTable, VhdxBlockAllocationTableEntry};
use super::block_range::{VhdxBlockRange, VhdxBlockRangeType};
use super::enums::VhdxDiskType;
use super::sector_bitmap::VhdxSectorBitmap;

/// Virtual Hard Disk version 2 (VHDX) block reader.
pub struct VhdxBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Disk type.
    disk_type: VhdxDiskType,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Number of entries per chunk;
    entries_per_chunk: u64,

    /// Sector bitmap size.
    sector_bitmap_size: u32,

    /// Block allocation table.
    block_allocation_table: VhdxBlockAllocationTable,

    /// Block tree.
    block_tree: BlockTree<VhdxBlockRange>,

    /// Parent data stream.
    parent_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl VhdxBlockReader {
    /// Creates a block reader.
    pub fn new(
        data_stream: &DataStreamReference,
        disk_type: &VhdxDiskType,
        bytes_per_sector: u16,
        block_size: u32,
        block_allocation_table_offset: u64,
        block_allocation_table_size: u32,
        parent_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        let sectors_per_block: u32 = block_size / (bytes_per_sector as u32);
        let sector_bitmap_size: u32 =
            (sectors_per_block / 8).next_multiple_of(bytes_per_sector as u32);
        let entries_per_chunk: u64 = ((1 << 23) * (bytes_per_sector as u64)) / (block_size as u64);
        let number_of_blocks: u32 = block_allocation_table_size.div_ceil(8);

        Self {
            data_stream: data_stream.clone(),
            disk_type: disk_type.clone(),
            bytes_per_sector,
            block_size,
            entries_per_chunk,
            sector_bitmap_size,
            block_allocation_table: VhdxBlockAllocationTable::new(
                block_allocation_table_offset,
                number_of_blocks,
            ),
            block_tree: BlockTree::<VhdxBlockRange>::new(
                size,
                sectors_per_block as u64,
                bytes_per_sector as u64,
            ),
            parent_data_stream,
            size,
        }
    }

    /// Reads a specific block allocation entry and fills the block tree.
    fn read_block_allocation_entry(&mut self, block_number: u64) -> Result<(), ErrorTrace> {
        let table_entry: u64 = if self.disk_type == VhdxDiskType::Fixed {
            block_number
        } else {
            ((block_number / self.entries_per_chunk) * (self.entries_per_chunk + 1))
                + (block_number % self.entries_per_chunk)
        };
        let entry: VhdxBlockAllocationTableEntry = match self
            .block_allocation_table
            .read_entry(&self.data_stream, table_entry as u32)
        {
            Ok(entry) => entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block allocation table entry"
                );
                return Err(error);
            }
        };
        if self.disk_type == VhdxDiskType::Differential && entry.block_state != 6 {
            match self.read_sector_bitmap(block_number, entry.block_offset) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read sector bitmap");
                    return Err(error);
                }
            }
        } else {
            let block_logical_offset: u64 = block_number * (self.block_size as u64);

            let block_range: VhdxBlockRange = if entry.block_state < 6 {
                VhdxBlockRange::new(
                    block_logical_offset,
                    0,
                    self.block_size as u64,
                    VhdxBlockRangeType::Sparse,
                )
            } else {
                VhdxBlockRange::new(
                    block_logical_offset,
                    entry.block_offset,
                    self.block_size as u64,
                    VhdxBlockRangeType::InFile,
                )
            };
            match self.block_tree.insert_value(
                block_logical_offset,
                self.block_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to insert block range into block tree"
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads a specific sector bitmap and fills the block tree.
    fn read_sector_bitmap(
        &mut self,
        block_number: u64,
        block_offset: u64,
    ) -> Result<(), ErrorTrace> {
        let table_entry: u64 =
            (1 + (block_number / self.entries_per_chunk)) * (self.entries_per_chunk + 1) - 1;

        let entry: VhdxBlockAllocationTableEntry = match self
            .block_allocation_table
            .read_entry(&self.data_stream, table_entry as u32)
        {
            Ok(entry) => entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block allocation table entry"
                );
                return Err(error);
            }
        };
        let sector_bitmap_offset: u64 = entry.block_offset
            + ((block_number % self.entries_per_chunk) * self.sector_bitmap_size as u64);

        let mut sector_bitmap: VhdxSectorBitmap =
            VhdxSectorBitmap::new(self.sector_bitmap_size as usize, self.bytes_per_sector);

        match sector_bitmap
            .read_at_position(&self.data_stream, SeekFrom::Start(sector_bitmap_offset))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read sector bitmap");
                return Err(error);
            }
        }
        let mut range_logical_offset: u64 = block_number * (self.block_size as u64);
        let mut range_data_offset: u64 = block_offset;

        for bitmap_range in sector_bitmap.ranges.iter() {
            let block_range: VhdxBlockRange = if bitmap_range.is_set {
                VhdxBlockRange::new(
                    range_logical_offset,
                    range_data_offset,
                    bitmap_range.size,
                    VhdxBlockRangeType::InFile,
                )
            } else {
                VhdxBlockRange::new(
                    range_logical_offset,
                    0,
                    bitmap_range.size,
                    VhdxBlockRangeType::InParent,
                )
            };
            match self
                .block_tree
                .insert_value(range_logical_offset, bitmap_range.size, block_range)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to insert block range into block tree"
                    );
                    return Err(error);
                }
            }
            range_logical_offset += bitmap_range.size;
            range_data_offset += bitmap_range.size;
        }
        Ok(())
    }
}

impl BlockReader for VhdxBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut block_number: u64 = current_offset / (self.block_size as u64);

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let mut result: Result<Option<&VhdxBlockRange>, ErrorTrace> =
                self.block_tree.get_value(current_offset);

            if result == Ok(None) {
                match self.read_block_allocation_entry(block_number) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read block allocation entry: {}", block_number)
                        );
                        return Err(error);
                    }
                }
                result = self.block_tree.get_value(current_offset);
            }
            let block_range: &VhdxBlockRange = match result {
                Ok(Some(block_range)) => block_range,
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
                            current_offset, current_offset,
                        )
                    );
                    return Err(error);
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.logical_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            let range_read_count: usize = match block_range.range_type {
                VhdxBlockRangeType::InFile => {
                    let physical_offset: u64 = block_range.physical_offset + range_relative_offset;

                    keramics_core::data_stream_read_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(physical_offset)
                    )
                }
                VhdxBlockRangeType::InParent => match &self.parent_data_stream {
                    Some(parent_data_stream) => {
                        keramics_core::data_stream_read_at_position!(
                            parent_data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(current_offset)
                        )
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing parent file"));
                    }
                },
                VhdxBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            current_offset += range_read_count as u64;

            block_number += 1;
        }
        Ok(data_offset)
    }
}
