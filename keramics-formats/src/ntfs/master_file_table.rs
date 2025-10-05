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

use std::collections::HashSet;
use std::io;
use std::io::SeekFrom;

use keramics_core::DataStreamReference;

use crate::block_tree::BlockTree;

use super::attribute_list::NtfsAttributeList;
use super::block_range::{NtfsBlockRange, NtfsBlockRangeType};
use super::cluster_group::NtfsClusterGroup;
use super::constants::*;
use super::data_run::NtfsDataRunType;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_attributes::NtfsMftAttributes;
use super::mft_entry::NtfsMftEntry;

/// New Technologies File System (NTFS) Master File Table (MFT).
pub struct NtfsMasterFileTable {
    /// Cluster block size.
    pub cluster_block_size: u32,

    /// MFT entry size.
    mft_entry_size: u32,

    /// Number of entries.
    pub number_of_entries: u64,

    /// Block tree.
    block_tree: BlockTree<NtfsBlockRange>,
}

impl NtfsMasterFileTable {
    /// Creates a new master file table.
    pub fn new() -> Self {
        Self {
            cluster_block_size: 0,
            mft_entry_size: 0,
            number_of_entries: 0,
            block_tree: BlockTree::<NtfsBlockRange>::new(0, 0, 0),
        }
    }

    /// Adds a cluster group to the master file table.
    fn add_cluster_group(&mut self, cluster_group: &NtfsClusterGroup) -> io::Result<()> {
        let mut virtual_cluster_number: u64 = cluster_group.first_vcn;
        let mut virtual_cluster_offset: u64 =
            cluster_group.first_vcn * (self.cluster_block_size as u64);

        for data_run in cluster_group.data_runs.iter() {
            let range_size: u64 = data_run.number_of_blocks * (self.cluster_block_size as u64);

            if range_size % (self.mft_entry_size as u64) != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Unsupported data run - size: {} not a multitude of MFT entry size: {}.",
                        range_size, self.mft_entry_size,
                    ),
                ));
            }
            self.number_of_entries += range_size / (self.mft_entry_size as u64);

            let range_type: NtfsBlockRangeType = match &data_run.run_type {
                NtfsDataRunType::InFile => NtfsBlockRangeType::InFile,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported data run type.",
                    ));
                }
            };
            let block_range: NtfsBlockRange = NtfsBlockRange::new(
                virtual_cluster_offset,
                data_run.block_number,
                data_run.number_of_blocks,
                range_type,
            );
            match self
                .block_tree
                .insert_value(virtual_cluster_offset, range_size, block_range)
            {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            };
            virtual_cluster_number += data_run.number_of_blocks;
            virtual_cluster_offset += range_size;
        }
        if cluster_group.last_vcn != 0xffffffffffffffff
            && cluster_group.last_vcn + 1 != virtual_cluster_number
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Cluster group last VNC: {} does not match expected value.",
                    cluster_group.last_vcn
                ),
            ));
        }
        Ok(())
    }

    /// Retrieves a specific entry.
    pub fn get_entry(
        &self,
        data_stream: &DataStreamReference,
        entry_number: u64,
    ) -> io::Result<NtfsMftEntry> {
        let virtual_cluster_offset: u64 = entry_number * (self.mft_entry_size as u64);

        let block_range: &NtfsBlockRange = match self.block_tree.get_value(virtual_cluster_offset) {
            Some(value) => value,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Missing block range for MFT entry: {}", entry_number),
                ));
            }
        };
        let range_relative_offset: u64 =
            virtual_cluster_offset - block_range.virtual_cluster_offset;
        let mft_entry_offset: u64 = (block_range.cluster_block_number
            * (self.cluster_block_size as u64))
            + range_relative_offset;

        let mut mft_entry: NtfsMftEntry = NtfsMftEntry::new();

        mft_entry.read_at_position(
            data_stream,
            self.mft_entry_size,
            SeekFrom::Start(mft_entry_offset),
        )?;
        Ok(mft_entry)
    }

    /// Initializes the master file table.
    pub fn initialize(
        &mut self,
        cluster_block_size: u32,
        mft_entry_size: u32,
        data_stream: &DataStreamReference,
        mft_block_number: u64,
    ) -> io::Result<()> {
        let mut mft_entry: NtfsMftEntry = NtfsMftEntry::new();
        let mft_offset: u64 = mft_block_number * (cluster_block_size as u64);

        mft_entry.read_at_position(data_stream, mft_entry_size, SeekFrom::Start(mft_offset))?;
        if mft_entry.is_bad {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported marked bad MFT entry: 0.",
            ));
        }
        if !mft_entry.is_allocated {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported unallocated MFT entry: 0.",
            ));
        }
        let mut mft_attributes: NtfsMftAttributes = NtfsMftAttributes::new();
        mft_entry.read_attributes(&mut mft_attributes)?;

        let data_attribute: &NtfsMftAttribute =
            match mft_attributes.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(mft_attribute) => mft_attribute,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing $Data attribute in MFT entry: 0.",
                    ));
                }
            };
        if data_attribute.is_resident() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported resident $DATA attribute.",
            ));
        }
        if data_attribute.is_compressed() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported compressed $DATA attribute.",
            ));
        }
        let block_tree_size: u64 =
            (data_attribute.allocated_data_size / mft_entry_size as u64) * (mft_entry_size as u64);
        self.block_tree =
            BlockTree::<NtfsBlockRange>::new(block_tree_size, 0, mft_entry_size as u64);

        self.cluster_block_size = cluster_block_size;
        self.mft_entry_size = mft_entry_size;

        self.add_cluster_group(&data_attribute.data_cluster_groups[0])?;

        let mft_entries: Vec<u64> = match mft_attributes.attribute_list {
            Some(attribute_index) => {
                let mft_attribute: &NtfsMftAttribute =
                    mft_attributes.get_attribute_by_index(attribute_index)?;

                let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();
                attribute_list.read_attribute(&mft_attribute, data_stream, cluster_block_size)?;
                let mut mft_entries_set: HashSet<u64> = HashSet::new();
                for entry in attribute_list.entries.iter() {
                    let mft_entry_number: u64 = entry.file_reference & 0x0000ffffffffffff;
                    if mft_entry_number != 0 {
                        mft_entries_set.insert(mft_entry_number);
                    }
                }
                let mut mft_entries: Vec<u64> = mft_entries_set.drain().collect::<Vec<u64>>();
                mft_entries.sort();

                mft_entries
            }
            None => Vec::new(),
        };
        for mft_entry_number in mft_entries.iter() {
            let mft_entry: NtfsMftEntry = self.get_entry(data_stream, *mft_entry_number)?;
            let mut mft_attributes: NtfsMftAttributes = NtfsMftAttributes::new();
            mft_entry.read_attributes(&mut mft_attributes)?;

            match mft_attributes.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(mft_attribute) => {
                    self.add_cluster_group(&mft_attribute.data_cluster_groups[0])?
                }
                None => {}
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_mft_attribute_data() -> Vec<u8> {
        return vec![
            0x80, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x01, 0x00, 0x40, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x14, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x13, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_add_cluster_group() -> io::Result<()> {
        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut test_struct: NtfsMasterFileTable = NtfsMasterFileTable::new();

        let block_tree_size: u64 = (data_attribute.allocated_data_size / 1024) * 1024;
        test_struct.block_tree = BlockTree::<NtfsBlockRange>::new(block_tree_size, 0, 1024);

        test_struct.cluster_block_size = 4096;
        test_struct.mft_entry_size = 1024;

        test_struct.add_cluster_group(&data_attribute.data_cluster_groups[0])?;

        Ok(())
    }

    // TODO: add tests for get_entry
    // TODO: add tests for initialize
}
