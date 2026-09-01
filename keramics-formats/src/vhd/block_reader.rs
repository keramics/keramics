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

use super::block_allocation_table::{VhdBlockAllocationTable, VhdBlockAllocationTableEntry};
use super::block_range::{VhdBlockRange, VhdBlockRangeType};
use super::enums::VhdDiskType;
use super::sector_bitmap::VhdSectorBitmap;

/// Virtual Hard Disk (VHD) (dynamic and differential disk) block reader.
pub struct VhdBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Disk type.
    disk_type: VhdDiskType,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Sector bitmap size.
    sector_bitmap_size: u32,

    /// Block allocation table.
    block_allocation_table: VhdBlockAllocationTable,

    /// Block tree.
    block_tree: BlockTree<VhdBlockRange>,

    /// Parent data stream.
    parent_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl VhdBlockReader {
    /// Creates a block reader.
    pub fn new(
        data_stream: &DataStreamReference,
        disk_type: &VhdDiskType,
        bytes_per_sector: u16,
        block_size: u32,
        block_allocation_table_offset: u64,
        number_of_blocks: u32,
        parent_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        let sectors_per_block: u32 = block_size / (bytes_per_sector as u32);
        let sector_bitmap_size: u32 = sectors_per_block
            .div_ceil(8)
            .next_multiple_of(bytes_per_sector as u32);

        Self {
            data_stream: data_stream.clone(),
            disk_type: disk_type.clone(),
            bytes_per_sector,
            block_size,
            sector_bitmap_size,
            block_allocation_table: VhdBlockAllocationTable::new(
                block_allocation_table_offset,
                number_of_blocks,
            ),
            block_tree: BlockTree::<VhdBlockRange>::new(
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
        let entry: VhdBlockAllocationTableEntry = match self
            .block_allocation_table
            .read_entry(&self.data_stream, block_number as u32)
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
        if entry.sector_number != 0xffffffff {
            match self.read_sector_bitmap(block_number, entry.sector_number) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read sector bitmap");
                    return Err(error);
                }
            }
        } else {
            let block_logical_offset: u64 = block_number * (self.block_size as u64);

            let block_range: VhdBlockRange = if self.disk_type == VhdDiskType::Dynamic {
                VhdBlockRange::new(
                    block_logical_offset,
                    0,
                    self.block_size as u64,
                    VhdBlockRangeType::Sparse,
                )
            } else {
                VhdBlockRange::new(
                    block_logical_offset,
                    0,
                    self.block_size as u64,
                    VhdBlockRangeType::InParent,
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
        sector_number: u32,
    ) -> Result<(), ErrorTrace> {
        let sector_bitmap_offset: u64 = (sector_number as u64) * (self.bytes_per_sector as u64);

        let mut sector_bitmap: VhdSectorBitmap =
            VhdSectorBitmap::new(self.sector_bitmap_size as usize, self.bytes_per_sector);

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
        let mut range_data_offset: u64 = sector_bitmap_offset + (self.sector_bitmap_size as u64);

        for bitmap_range in sector_bitmap.ranges.iter() {
            let block_range: VhdBlockRange = if bitmap_range.is_set {
                VhdBlockRange::new(
                    range_logical_offset,
                    range_data_offset,
                    bitmap_range.size,
                    VhdBlockRangeType::InFile,
                )
            } else if self.disk_type == VhdDiskType::Dynamic {
                VhdBlockRange::new(
                    range_logical_offset,
                    0,
                    bitmap_range.size,
                    VhdBlockRangeType::Sparse,
                )
            } else {
                VhdBlockRange::new(
                    range_logical_offset,
                    0,
                    bitmap_range.size,
                    VhdBlockRangeType::InParent,
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

impl BlockReader for VhdBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_number: u64 = current_offset / (self.block_size as u64);

            let mut result: Result<Option<&VhdBlockRange>, ErrorTrace> =
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
            let block_range: &VhdBlockRange = match result {
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
                VhdBlockRangeType::InFile => {
                    let physical_offset: u64 = block_range.physical_offset + range_relative_offset;

                    keramics_core::data_stream_read_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(physical_offset)
                    )
                }
                VhdBlockRangeType::InParent => match &self.parent_data_stream {
                    Some(data_stream) => {
                        keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(current_offset)
                        )
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing parent data stream"
                        ));
                    }
                },
                VhdBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            current_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }
}
