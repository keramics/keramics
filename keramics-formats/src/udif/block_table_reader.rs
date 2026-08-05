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

use keramics_core::ErrorTrace;

use crate::block_tree::BlockTree;

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::block_table::UdifBlockTable;
use super::enums::UdifCompressionMethod;

/// Universal Disk Image Format (UDIF) block table reader.
#[derive(Debug)]
pub struct UdifBlockTableReader {
    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Data fork offset.
    data_fork_offset: u64,

    /// Data fork end offset.
    data_fork_end_offset: u64,

    /// Block ranges.
    block_ranges: Vec<UdifBlockRange>,

    /// Current media sector.
    media_sector: u64,

    /// Current media offset.
    media_offset: u64,

    /// Compressed entry type.
    compressed_entry_type: u32,
}

impl UdifBlockTableReader {
    const MAXIMUM_NUMBER_OF_SECTORS: u64 = u64::MAX / 512;

    /// Creates a new block table reader.
    pub fn new(bytes_per_sector: u16, data_fork_offset: u64, data_fork_size: u64) -> Self {
        Self {
            bytes_per_sector,
            data_fork_offset,
            data_fork_end_offset: data_fork_offset + data_fork_size,
            block_ranges: Vec::new(),
            media_sector: 0,
            media_offset: 0,
            compressed_entry_type: 0,
        }
    }

    /// Retrieves the block tree.
    pub fn get_block_tree(&mut self) -> Result<BlockTree<UdifBlockRange>, ErrorTrace> {
        let block_tree_data_size: u64 = self.media_sector * (self.bytes_per_sector as u64);

        let mut block_tree: BlockTree<UdifBlockRange> =
            BlockTree::<UdifBlockRange>::new(block_tree_data_size, 0, self.bytes_per_sector as u64);

        while let Some(block_range) = self.block_ranges.pop() {
            match block_tree.insert_value(block_range.media_offset, block_range.size, block_range) {
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
        Ok(block_tree)
    }

    /// Retrieves the compression method.
    pub fn get_compression_method(&self) -> UdifCompressionMethod {
        match self.compressed_entry_type {
            0x80000004 => UdifCompressionMethod::Adc,
            0x80000005 => UdifCompressionMethod::Zlib,
            0x80000006 => UdifCompressionMethod::Bzip2,
            0x80000007 => UdifCompressionMethod::Lzfse,
            0x80000008 => UdifCompressionMethod::Lzma,
            _ => UdifCompressionMethod::None,
        }
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_offset
    }

    /// Processes a block table.
    pub fn process_block_table(&mut self, block_table: &UdifBlockTable) -> Result<(), ErrorTrace> {
        if block_table.start_sector != self.media_sector {
            return Err(keramics_core::error_trace_new!(
                "Unsupported block table - start sector value out of bounds"
            ));
        }
        for (entry_index, block_table_entry) in block_table.entries.iter().enumerate() {
            if block_table_entry.entry_type == 0xffffffff {
                break;
            }
            if block_table_entry.entry_type == 0x7ffffffe {
                continue;
            }
            if block_table_entry.start_sector
                > Self::MAXIMUM_NUMBER_OF_SECTORS - block_table.start_sector
                || block_table.start_sector + block_table_entry.start_sector != self.media_sector
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported block table entry: {} - start sector value out of bounds",
                    entry_index
                )));
            }
            if block_table_entry.number_of_sectors == 0
                || block_table_entry.number_of_sectors > Self::MAXIMUM_NUMBER_OF_SECTORS
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported block table entry: {} - number of sectors value out of bounds",
                    entry_index
                )));
            }
            if block_table_entry.entry_type != 0x00000000
                && block_table_entry.entry_type != 0x00000002
            {
                if block_table_entry.data_offset < self.data_fork_offset
                    || block_table_entry.data_offset >= self.data_fork_end_offset
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported block table entry: {} - data offset value out of bounds",
                        entry_index
                    )));
                }
                if block_table_entry.data_size
                    > self.data_fork_end_offset - block_table_entry.data_offset
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported block table entry: {} - data size value out of bounds",
                        entry_index
                    )));
                }
            }
            let media_size: u64 =
                block_table_entry.number_of_sectors * (self.bytes_per_sector as u64);

            let block_range: UdifBlockRange = match block_table_entry.entry_type {
                0x00000000 | 0x00000002 => UdifBlockRange::new(
                    self.media_offset,
                    0,
                    media_size,
                    0,
                    UdifBlockRangeType::Sparse,
                ),
                0x00000001 => UdifBlockRange::new(
                    self.media_offset,
                    block_table_entry.data_offset,
                    media_size,
                    0,
                    UdifBlockRangeType::InFile,
                ),
                0x80000004..0x80000008 => {
                    if block_table_entry.number_of_sectors > 2048 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported compressed block table entry: {} - number of sectors value out of bounds",
                            entry_index
                        )));
                    }
                    if self.compressed_entry_type == 0 {
                        self.compressed_entry_type = block_table_entry.entry_type;
                    } else if block_table_entry.entry_type != self.compressed_entry_type {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported mixed compression methods"
                        ));
                    }
                    UdifBlockRange::new(
                        self.media_offset,
                        block_table_entry.data_offset,
                        media_size,
                        block_table_entry.data_size as u32,
                        UdifBlockRangeType::Compressed,
                    )
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported block table entry type: 0x{:08x}",
                        block_table_entry.entry_type
                    )));
                }
            };
            self.block_ranges.push(block_range);

            self.media_offset += media_size;
            self.media_sector += block_table_entry.number_of_sectors;
        }
        Ok(())
    }
}
