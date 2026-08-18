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

use std::cmp::Ordering;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::block_range::{NtfsBlockRange, NtfsBlockRangeType};
use super::data_run::NtfsDataRunType;
use super::index_entry::NtfsIndexEntry;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) index.
pub struct NtfsIndex {
    /// Cluster block size.
    pub cluster_block_size: u32,

    /// Index entry size.
    pub index_entry_size: u32,

    /// Block ranges.
    block_ranges: Vec<NtfsBlockRange>,
}

impl NtfsIndex {
    /// Creates a new index.
    pub fn new(cluster_block_size: u32) -> Self {
        Self {
            cluster_block_size,
            index_entry_size: 0,
            block_ranges: Vec::new(),
        }
    }

    /// Retrieves a specific index entry.
    pub fn get_entry_at_cluster_block(
        &self,
        data_stream: &DataStreamReference,
        virtual_cluster_number: u64,
    ) -> Result<NtfsIndexEntry, ErrorTrace> {
        let virtual_cluster_offset: u64 = virtual_cluster_number * (self.cluster_block_size as u64);

        let range_index: usize = match self.block_ranges.binary_search_by(|block_range| {
            let range_end_offset: u64 = block_range.virtual_cluster_offset + block_range.size;

            if virtual_cluster_offset >= range_end_offset {
                Ordering::Less
            } else if virtual_cluster_offset < block_range.virtual_cluster_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(range_index) => range_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing block range for offset: {} (0x{:08x})",
                    virtual_cluster_offset, virtual_cluster_offset
                )));
            }
        };
        let block_range: &NtfsBlockRange = match self.block_ranges.get(range_index) {
            Some(block_range) => block_range,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                    range_index, virtual_cluster_offset, virtual_cluster_offset,
                )));
            }
        };
        let range_relative_offset: u64 =
            virtual_cluster_offset - block_range.virtual_cluster_offset;
        let index_entry_offset: u64 = (block_range.cluster_block_number
            * (self.cluster_block_size as u64))
            + range_relative_offset;

        let remaining_range_size: u64 = block_range.size - range_relative_offset;
        if remaining_range_size < (self.index_entry_size as u64) {
            return Err(keramics_core::error_trace_new!(format!(
                "Block range too small for index entry of size: {}",
                self.index_entry_size
            )));
        }
        let mut index_entry: NtfsIndexEntry = NtfsIndexEntry::new();

        match index_entry.read_at_position(
            data_stream,
            self.index_entry_size,
            SeekFrom::Start(index_entry_offset),
        ) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read index entry at offset: {} (0x{:08x})",
                        index_entry_offset, index_entry_offset
                    )
                );
                return Err(error);
            }
        }
        Ok(index_entry)
    }

    /// Initializes the index.
    pub fn initialize(
        &mut self,
        index_entry_size: u32,
        index_allocation_attribute: &NtfsMftAttribute,
    ) -> Result<(), ErrorTrace> {
        if index_allocation_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported resident $INDEX_ALLOCATION attribute"
            ));
        }
        if index_allocation_attribute.is_compressed() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported compressed $INDEX_ALLOCATION attribute"
            ));
        }
        let mut virtual_cluster_number: u64 = 0;
        let mut virtual_cluster_offset: u64 = 0;

        for cluster_group in index_allocation_attribute.data_cluster_groups.iter() {
            if cluster_group.first_vcn != virtual_cluster_number {
                return Err(keramics_core::error_trace_new!(format!(
                    "$INDEX_ALLOCATION attribute cluster group first VNC: {} does not match expected value: {}",
                    cluster_group.first_vcn, virtual_cluster_number
                )));
            }
            for data_run in cluster_group.data_runs.iter() {
                let range_size: u64 = data_run.number_of_blocks * (self.cluster_block_size as u64);

                let range_type: NtfsBlockRangeType = match &data_run.run_type {
                    NtfsDataRunType::InFile => NtfsBlockRangeType::InFile,
                    _ => {
                        return Err(keramics_core::error_trace_new!("Unsupported data run type"));
                    }
                };
                let block_range: NtfsBlockRange = NtfsBlockRange::new(
                    virtual_cluster_offset,
                    data_run.block_number,
                    range_size,
                    range_type,
                );
                self.block_ranges.push(block_range);

                virtual_cluster_number += data_run.number_of_blocks as u64;
                virtual_cluster_offset += range_size;
            }
            if cluster_group.last_vcn != 0xffffffffffffffff
                && cluster_group.last_vcn + 1 != virtual_cluster_number
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Cluster group last VNC: {} does not match expected value",
                    cluster_group.last_vcn
                )));
            }
        }
        self.index_entry_size = index_entry_size;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_test_mft_attribute_data() -> Vec<u8> {
        return vec![
            0xa0, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x04, 0x40, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x49, 0x00, 0x33, 0x00,
            0x30, 0x00, 0x21, 0x01, 0x85, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_get_entry_at_cluster_block() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut index_allocation_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        index_allocation_attribute.read_data(&test_mft_attribute_data)?;

        let mut index: NtfsIndex = NtfsIndex::new(4096);
        index.initialize(4096, &index_allocation_attribute)?;

        index.get_entry_at_cluster_block(&data_stream, 0)?;

        Ok(())
    }

    #[test]
    fn test_initialize() -> Result<(), ErrorTrace> {
        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut index_allocation_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        index_allocation_attribute.read_data(&test_mft_attribute_data)?;

        let mut index: NtfsIndex = NtfsIndex::new(4096);
        index.initialize(4096, &index_allocation_attribute)?;

        Ok(())
    }
}
