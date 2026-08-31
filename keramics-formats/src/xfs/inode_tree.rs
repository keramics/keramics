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

use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::btree_node::XfsBtreeNode;
use super::constants::*;
use super::inode::XfsInode;
use super::inode_tree_branch_key::XfsInodeTreeBranchKey;
use super::inode_tree_branch_value::XfsInodeTreeBranchValue;
use super::inode_tree_leaf_record::XfsInodeTreeLeafRecord;
use super::superblock::XfsSuperblock;

/// X File System (XFS) inode tree
pub struct XfsInodeTree {
    /// Format version.
    pub format_version: u16,

    /// Value to indicate bigtime date and time values are used.
    has_bigtime: bool,

    /// Value to indicate 64-bit number of data extents and 32-bit number of attributes extents are used.
    has_64bit_number_of_extents: bool,

    /// Allocation group size.
    pub allocation_group_size: u32,

    /// Block size.
    pub block_size: u32,

    /// Inode size.
    inode_size: u16,

    /// Absolute inode number bit shift.
    absolute_inode_number_bit_shift: u64,

    /// Relative inode number bit mask.
    relative_inode_number_bit_mask: u64,

    /// Root block numbers.
    pub root_block_numbers: Vec<u32>,

    /// Directory block size.
    pub directory_block_size: u32,

    /// Number of relative block number bits.
    pub number_of_relative_block_number_bits: u32,

    /// Root directory (absolute) inode number.
    pub root_directory_inode_number: u64,
}

impl XfsInodeTree {
    /// Creates a new inode tree.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            has_bigtime: false,
            has_64bit_number_of_extents: false,
            allocation_group_size: 0,
            block_size: 0,
            inode_size: 0,
            absolute_inode_number_bit_shift: 0,
            relative_inode_number_bit_mask: 0,
            root_block_numbers: Vec::new(),
            directory_block_size: 0,
            number_of_relative_block_number_bits: 0,
            root_directory_inode_number: 0,
        }
    }

    /// Retrieves a specific inode.
    pub fn get_inode_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        absolute_inode_number: u64,
    ) -> Result<Option<XfsInode>, ErrorTrace> {
        let allocation_group_index: u64 =
            absolute_inode_number >> self.absolute_inode_number_bit_shift;
        let allocation_group_block_number: u64 =
            allocation_group_index * (self.allocation_group_size as u64);

        let root_block_number: u32 =
            match self.root_block_numbers.get(allocation_group_index as usize) {
                Some(block_number) => *block_number,
                None => return Ok(None),
            };
        let relative_inode_number: u64 =
            absolute_inode_number & self.relative_inode_number_bit_mask;

        if relative_inode_number > (u32::MAX as u64) {
            return Err(keramics_core::error_trace_new!(
                "Invalid relative inode number value out of bounds"
            ));
        }
        let mut read_block_numbers: HashSet<u32> = HashSet::new();

        match self.get_inode_by_identifier_from_node(
            data_stream,
            allocation_group_index,
            allocation_group_block_number,
            root_block_number,
            relative_inode_number as u32,
            &mut read_block_numbers,
        ) {
            Ok(true) => {
                let inode_offset: u64 = (allocation_group_block_number * (self.block_size as u64))
                    + (relative_inode_number * (self.inode_size as u64));

                let mut inode: XfsInode = XfsInode::new();

                match inode.read_at_position(
                    self.format_version,
                    self.has_bigtime,
                    self.has_64bit_number_of_extents,
                    data_stream,
                    self.inode_size,
                    SeekFrom::Start(inode_offset),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read inode: {}", absolute_inode_number),
                        );
                        return Err(error);
                    }
                }
                match inode.read_extents(
                    self.format_version,
                    self.allocation_group_size,
                    self.number_of_relative_block_number_bits,
                    data_stream,
                    self.block_size,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read extents of inode: {}", absolute_inode_number),
                        );
                        return Err(error);
                    }
                }
                Ok(Some(inode))
            }
            Ok(false) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve inode from allocation group: {} root node",
                        allocation_group_index
                    ),
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific inode from a branch node.
    fn get_inode_by_identifier_from_branch_node(
        &self,
        data_stream: &DataStreamReference,
        allocation_group_index: u64,
        allocation_group_block_number: u64,
        relative_inode_number: u32,
        btree_node: &XfsBtreeNode,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<bool, ErrorTrace> {
        let data_size: usize = btree_node.data.len();

        let records_data_end_offset: usize =
            btree_node.records_offset + ((btree_node.number_of_records as usize) * 8);

        if records_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of records value out of bounds"
            ));
        }
        let number_of_key_value_pairs: usize = (data_size - btree_node.records_offset) / 8;
        let values_data_offset: usize = btree_node.records_offset + (number_of_key_value_pairs * 4);

        let mut record_index: u16 = 0;
        let mut key_data_offset: usize = btree_node.records_offset;

        while record_index < btree_node.number_of_records {
            keramics_core::debug_trace_structure!(XfsInodeTreeBranchKey::debug_read_data(
                &btree_node.data[key_data_offset..]
            ));
            let mut branch_key: XfsInodeTreeBranchKey = XfsInodeTreeBranchKey::new();

            match branch_key.read_data(&btree_node.data[key_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read inode B-tree branch record: {} key",
                            record_index
                        ),
                    );
                    return Err(error);
                }
            }
            key_data_offset += 4;

            if branch_key.inode_number > relative_inode_number {
                break;
            }
            record_index += 1;
        }
        if record_index > 0 {
            let last_record_index: usize = (record_index as usize) - 1;
            let value_data_offset: usize = values_data_offset + (last_record_index * 4);

            keramics_core::debug_trace_structure!(XfsInodeTreeBranchValue::debug_read_data(
                &btree_node.data[value_data_offset..]
            ));
            let mut branch_value: XfsInodeTreeBranchValue = XfsInodeTreeBranchValue::new();

            match branch_value.read_data(&btree_node.data[value_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read inode B-tree branch record: {} value",
                            last_record_index
                        ),
                    );
                    return Err(error);
                }
            }
            if branch_value.block_number == 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid inode B-tree branch record: {} value - sub block number value out of bounds",
                    last_record_index
                )));
            }
            match self.get_inode_by_identifier_from_node(
                data_stream,
                allocation_group_index,
                allocation_group_block_number,
                branch_value.block_number,
                relative_inode_number,
                read_block_numbers,
            ) {
                Ok(result) => return Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve inode record from allocation group: {} node: {}",
                            allocation_group_index, branch_value.block_number
                        ),
                    );
                    return Err(error);
                }
            }
        }
        Ok(false)
    }

    /// Retrieves a specific inode from a leaf node.
    fn get_inode_by_identifier_from_leaf_node(
        &self,
        relative_inode_number: u32,
        btree_node: &XfsBtreeNode,
    ) -> Result<bool, ErrorTrace> {
        let data_size: usize = btree_node.data.len();

        let records_data_end_offset: usize =
            btree_node.records_offset + ((btree_node.number_of_records as usize) * 16);

        if records_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of records value out of bounds"
            ));
        }
        // TODO: for branch node stop at keys_data_end_offset
        for (record_index, data_offset) in (btree_node.records_offset..records_data_end_offset)
            .step_by(16)
            .enumerate()
        {
            keramics_core::debug_trace_structure!(XfsInodeTreeLeafRecord::debug_read_data(
                &btree_node.data[data_offset..]
            ));
            let mut leaf_record: XfsInodeTreeLeafRecord = XfsInodeTreeLeafRecord::new();

            match leaf_record.read_data(&btree_node.data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read inode B-tree leaf record: {}", record_index),
                    );
                    return Err(error);
                }
            }
            if relative_inode_number >= leaf_record.inode_number
                && relative_inode_number < leaf_record.inode_number + 64
            {
                // TODO: check chunk_allocation_bitmap
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Retrieves a specific inode from a node.
    fn get_inode_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        allocation_group_index: u64,
        allocation_group_block_number: u64,
        relative_block_number: u32,
        relative_inode_number: u32,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<bool, ErrorTrace> {
        if read_block_numbers.contains(&relative_block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Inode tree node: {} already read",
                relative_block_number
            )));
        }
        let btree_node_offset: u64 = (allocation_group_block_number
            + (relative_block_number as u64))
            * (self.block_size as u64);

        let mut btree_node: XfsBtreeNode = XfsBtreeNode::new();

        match btree_node.read_at_position(
            self.format_version,
            32,
            data_stream,
            self.block_size,
            SeekFrom::Start(btree_node_offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read B-tree node: {} at offset: {} (0x{:08x})",
                        relative_block_number, btree_node_offset, btree_node_offset
                    )
                );
                return Err(error);
            }
        }
        read_block_numbers.insert(relative_block_number);

        if &btree_node.signature != &XFS_INODE_TREE_SIGNATURE
            && &btree_node.signature != &XFS_INODE_TREE_V5_SIGNATURE
        {
            return Err(keramics_core::error_trace_new!(
                "Unsupported inode B-tree node signature"
            ));
        }
        if btree_node.is_branch() {
            match self.get_inode_by_identifier_from_branch_node(
                data_stream,
                allocation_group_index,
                allocation_group_block_number,
                relative_inode_number,
                &btree_node,
                read_block_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve inode record from branch node: {}",
                            relative_block_number,
                        ),
                    );
                    Err(error)
                }
            }
        } else {
            match self.get_inode_by_identifier_from_leaf_node(relative_inode_number, &btree_node) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve inode record from leaf node: {}",
                            relative_block_number,
                        ),
                    );
                    Err(error)
                }
            }
        }
    }

    /// Initializes the inode tree.
    pub fn initialize(
        &mut self,
        superblock: &XfsSuperblock,
        has_bigtime: bool,
        has_64bit_number_of_extents: bool,
        root_directory_inode_number: u64,
    ) {
        self.format_version = superblock.format_version;
        self.has_bigtime = has_bigtime;
        self.has_64bit_number_of_extents = has_64bit_number_of_extents;
        self.allocation_group_size = superblock.allocation_group_size;
        self.block_size = superblock.block_size;
        self.inode_size = superblock.inode_size;
        self.absolute_inode_number_bit_shift =
            superblock.number_of_relative_inode_number_bits as u64;
        self.relative_inode_number_bit_mask =
            (1 << (superblock.number_of_relative_inode_number_bits as u64)) - 1;
        self.directory_block_size = superblock.directory_block_size;
        self.number_of_relative_block_number_bits = superblock.number_of_relative_block_number_bits;
        self.root_directory_inode_number = root_directory_inode_number;
    }
}
