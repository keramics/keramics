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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;

use super::block_allocation_table::{PdiBlockAllocationTable, PdiBlockAllocationTableEntry};
use super::block_range::{PdiBlockRange, PdiBlockRangeType};
use super::sparse_file_header::PdiSparseFileHeader;

/// Parallels Disk Image (PDI) sparse file.
pub struct PdiSparseFile {
    /// Data stream.
    pub(super) data_stream: Option<DataStreamReference>,

    /// Block size.
    block_size: u64,

    /// Block allocation table.
    block_allocation_table: PdiBlockAllocationTable,

    /// Block tree.
    pub(super) block_tree: BlockTree<PdiBlockRange>,
}

impl PdiSparseFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            block_size: 0,
            block_allocation_table: PdiBlockAllocationTable::new(),
            block_tree: BlockTree::<PdiBlockRange>::new(0, 0, 0),
        }
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut file_header: PdiSparseFileHeader = PdiSparseFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        self.block_size = (file_header.sectors_per_block as u64) * 512;
        self.block_allocation_table
            .set_range(64, file_header.number_of_blocks);

        let block_tree_data_size: u64 = (file_header.number_of_blocks as u64) * self.block_size;
        self.block_tree = BlockTree::<PdiBlockRange>::new(block_tree_data_size, 0, self.block_size);
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads a specific block allocation table entry and fills the block tree.
    pub fn read_block_allocation_table_entry(
        &mut self,
        extent_offset: u64,
    ) -> Result<(), ErrorTrace> {
        let block_index: u64 = extent_offset / self.block_size;

        if block_index > (u32::MAX as u64) {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid extent offset: {} (0x{:08x}) value out of bounds",
                extent_offset, extent_offset
            )));
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let entry: PdiBlockAllocationTableEntry = match self
            .block_allocation_table
            .read_entry(data_stream, block_index as u32)
        {
            Ok(sector_number) => sector_number,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read block allocation entry: {}", block_index)
                );
                return Err(error);
            }
        };
        let block_data_offset: u64 = (entry.sector_number as u64) * 512;
        let block_extent_offset: u64 = block_index * self.block_size;

        let range_type: PdiBlockRangeType = if entry.sector_number == 0 {
            PdiBlockRangeType::InParentOrSparse
        } else {
            PdiBlockRangeType::InFile
        };
        let block_range: PdiBlockRange = PdiBlockRange::new(
            block_extent_offset,
            block_data_offset,
            self.block_size,
            range_type,
        );
        match self
            .block_tree
            .insert_value(block_extent_offset, self.block_size, block_range)
        {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to insert block range into block tree",
                    error
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: PdiSparseFile = PdiSparseFile::new();

        let path_string: String = get_test_data_path(
            "pdi/hfsplus.hdd/hfsplus.hdd.0.{5fbaabe3-6958-40ff-92a7-860e329aab41}.hds",
        );
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file.read_data_stream(&data_stream)?;
        // TODO:

        Ok(())
    }
}
