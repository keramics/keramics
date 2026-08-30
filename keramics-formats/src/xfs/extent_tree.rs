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
use super::extent_list::XfsExtentList;
use super::extent_tree_branch_header::XfsExtentTreeBranchHeader;
use super::extent_tree_branch_key::XfsExtentTreeBranchKey;
use super::extent_tree_branch_value::XfsExtentTreeBranchValue;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) extent tree
pub struct XfsExtentTree {
    /// Format version.
    format_version: u16,

    /// Allocation group size.
    allocation_group_size: u32,

    /// Block size.
    block_size: u32,

    /// Block number bit shift.
    block_number_bit_shift: u64,

    /// Relative block number bit mask.
    relative_block_number_bit_mask: u64,
}

impl XfsExtentTree {
    /// Creates a new extent tree.
    pub fn new(
        format_version: u16,
        allocation_group_size: u32,
        number_of_relative_block_number_bits: u32,
        block_size: u32,
    ) -> Self {
        Self {
            format_version,
            allocation_group_size,
            block_size,
            block_number_bit_shift: number_of_relative_block_number_bits as u64,
            relative_block_number_bit_mask: (1 << (number_of_relative_block_number_bits as u64))
                - 1,
        }
    }

    /// Retrieves the extents from a branch node.
    pub fn get_extents_from_branch_node(
        &self,
        data_stream: &DataStreamReference,
        btree_node: &XfsBtreeNode,
        extents: &mut Vec<XfsPackedExtent>,
        read_node_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = btree_node.data.len();

        let records_data_end_offset: usize =
            btree_node.records_offset + ((btree_node.number_of_records as usize) * 16);

        if records_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of records value out of bounds"
            ));
        }
        let number_of_key_value_pairs: usize = (data_size - btree_node.records_offset) / 16;
        let values_data_offset: usize = btree_node.records_offset + (number_of_key_value_pairs * 8);

        let mut key_data_offset: usize = btree_node.records_offset;
        let mut value_data_offset: usize = values_data_offset;

        for record_index in 0..btree_node.number_of_records {
            keramics_core::debug_trace_structure!(XfsExtentTreeBranchKey::debug_read_data(
                &btree_node.data[key_data_offset..]
            ));
            let mut branch_key: XfsExtentTreeBranchKey = XfsExtentTreeBranchKey::new();

            match branch_key.read_data(&btree_node.data[key_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read extent B-tree branch record: {} key",
                            record_index
                        ),
                    );
                    return Err(error);
                }
            }
            key_data_offset += 8;

            keramics_core::debug_trace_structure!(XfsExtentTreeBranchValue::debug_read_data(
                &btree_node.data[value_data_offset..]
            ));
            let mut branch_value: XfsExtentTreeBranchValue = XfsExtentTreeBranchValue::new();

            match branch_value.read_data(&btree_node.data[value_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read extent B-tree branch record: {} value",
                            record_index
                        ),
                    );
                    return Err(error);
                }
            }
            value_data_offset += 8;

            if branch_value.block_number == 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid extent B-tree branch record: {} value - sub block number value out of bounds",
                    record_index
                )));
            }
            match self.get_extents_from_node(
                data_stream,
                branch_value.block_number,
                extents,
                read_node_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve extent record from node: {}",
                            branch_value.block_number,
                        ),
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves the extents from a node.
    pub fn get_extents_from_node(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
        extents: &mut Vec<XfsPackedExtent>,
        read_node_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
        if read_node_numbers.contains(&block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Extent tree node: {} already read",
                block_number
            )));
        }
        let allocation_group_index: u64 = block_number >> self.block_number_bit_shift;
        let allocation_group_block_number: u64 =
            allocation_group_index * (self.allocation_group_size as u64);
        let relative_block_number: u64 = block_number & self.relative_block_number_bit_mask;

        let btree_node_offset: u64 = ((allocation_group_block_number as u64)
            + (relative_block_number as u64))
            * (self.block_size as u64);

        let mut btree_node: XfsBtreeNode = XfsBtreeNode::new();

        match btree_node.read_at_position(
            self.format_version,
            64,
            data_stream,
            self.block_size,
            SeekFrom::Start(btree_node_offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read B-tree node at offset: {} (0x{:08x})",
                        btree_node_offset, btree_node_offset
                    )
                );
                return Err(error);
            }
        }
        read_node_numbers.insert(block_number);

        if &btree_node.signature != &XFS_EXTENT_TREE_SIGNATURE
            && &btree_node.signature != &XFS_EXTENT_TREE_V5_SIGNATURE
        {
            return Err(keramics_core::error_trace_new!(
                "Unsupported extent B-Tree node signature"
            ));
        }
        if btree_node.is_branch() {
            match self.get_extents_from_branch_node(
                data_stream,
                &btree_node,
                extents,
                read_node_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve extents from branch node: {}",
                            relative_block_number,
                        ),
                    );
                    Err(error)
                }
            }
        } else {
            let extent_list: XfsExtentList = XfsExtentList::new();

            match extent_list.read_data(
                btree_node.number_of_records as u64,
                &btree_node.data[btree_node.records_offset..],
                extents,
            ) {
                Ok(_) => Ok(()),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read extent list");
                    Err(error)
                }
            }
        }
    }

    /// Reads the extents.
    pub fn read_extents(
        &self,
        data_stream: &DataStreamReference,
        root_node_data: &[u8],
        extents: &mut Vec<XfsPackedExtent>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = root_node_data.len();

        keramics_core::debug_trace_data!("XfsExtentTreeRootNode", 0, &root_node_data, data_size);

        let mut read_node_numbers: HashSet<u64> = HashSet::new();

        keramics_core::debug_trace_structure!(XfsExtentTreeBranchHeader::debug_read_data(
            &root_node_data
        ));
        let mut branch_header: XfsExtentTreeBranchHeader = XfsExtentTreeBranchHeader::new();

        match branch_header.read_data(root_node_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read extent B-tree branch header",
                );
                return Err(error);
            }
        }
        let records_data_end_offset: usize = 4 + ((branch_header.number_of_records as usize) * 16);

        if records_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of records value out of bounds"
            ));
        }
        let number_of_key_value_pairs: usize = (data_size - 4) / 16;
        let values_data_offset: usize = 4 + (number_of_key_value_pairs * 8);

        let mut key_data_offset: usize = 4;
        let mut value_data_offset: usize = values_data_offset;

        for record_index in 0..branch_header.number_of_records {
            keramics_core::debug_trace_structure!(XfsExtentTreeBranchKey::debug_read_data(
                &root_node_data[key_data_offset..]
            ));
            let mut branch_key: XfsExtentTreeBranchKey = XfsExtentTreeBranchKey::new();

            match branch_key.read_data(&root_node_data[key_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read extent B-tree branch record: {} key",
                            record_index
                        ),
                    );
                    return Err(error);
                }
            }
            key_data_offset += 8;

            keramics_core::debug_trace_structure!(XfsExtentTreeBranchValue::debug_read_data(
                &root_node_data[value_data_offset..]
            ));
            let mut branch_value: XfsExtentTreeBranchValue = XfsExtentTreeBranchValue::new();

            match branch_value.read_data(&root_node_data[value_data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read extent B-tree branch record: {} value",
                            record_index
                        ),
                    );
                    return Err(error);
                }
            }
            value_data_offset += 8;

            if branch_value.block_number == 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid extent B-tree branch record: {} value - sub block number value out of bounds",
                    record_index
                )));
            }
            match self.get_extents_from_node(
                data_stream,
                branch_value.block_number,
                extents,
                &mut read_node_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve extent record from node: {}",
                            branch_value.block_number,
                        ),
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}
