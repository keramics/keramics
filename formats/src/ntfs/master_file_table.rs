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

use core::DataStreamReference;

use crate::block_tree::BlockTree;

use super::block_range::{NtfsBlockRange, NtfsBlockRangeType};
use super::data_run::NtfsDataRunType;
use super::mft_attribute::NtfsMftAttribute;
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
            io::SeekFrom::Start(mft_entry_offset),
        )?;
        Ok(mft_entry)
    }

    /// Initializes the master file table.
    pub fn initialize(
        &mut self,
        cluster_block_size: u32,
        mft_entry_size: u32,
        data_attribute: &NtfsMftAttribute,
    ) -> io::Result<()> {
        if mft_entry_size > cluster_block_size || cluster_block_size % mft_entry_size != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported MFT entry size: {} value not a multitude of cluster block size: {}.",
                    mft_entry_size,
                    cluster_block_size
                ),
            ));
        }
        self.cluster_block_size = cluster_block_size;
        self.mft_entry_size = mft_entry_size;

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
        let number_of_mft_entries: u64 = data_attribute.data_size.div_ceil(mft_entry_size as u64);
        let block_tree_size: u64 = number_of_mft_entries * (cluster_block_size as u64);
        self.block_tree =
            BlockTree::<NtfsBlockRange>::new(block_tree_size, 0, mft_entry_size as u64);

        let mut virtual_cluster_number: u64 = 0;
        let mut virtual_cluster_offset: u64 = 0;

        for cluster_group in data_attribute.data_cluster_groups.iter() {
            if virtual_cluster_number != cluster_group.first_vcn {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Cluster group first VNC: {} does not match expected value.",
                        cluster_group.first_vcn
                    ),
                ));
            }
            for data_run in cluster_group.data_runs.iter() {
                let range_size: u64 = data_run.number_of_blocks * (cluster_block_size as u64);

                let range_type: NtfsBlockRangeType = match &data_run.run_type {
                    NtfsDataRunType::InFile => NtfsBlockRangeType::InFile,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported data run type.",
                        ))
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
                    Err(error) => return Err(core::error_to_io_error!(error)),
                };
                virtual_cluster_number += data_run.number_of_blocks as u64;
                virtual_cluster_offset += range_size;
            }
            if virtual_cluster_number != cluster_group.last_vcn + 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Cluster group last VNC: {} does not match expected value.",
                        cluster_group.last_vcn
                    ),
                ));
            }
        }
        self.number_of_entries = number_of_mft_entries;

        Ok(())
    }
}

// TODO: add tests.
