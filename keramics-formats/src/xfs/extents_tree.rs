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
use super::extents_list::XfsExtentsList;
use super::extents_tree_branch_header::XfsExtentsTreeBranchHeader;
use super::extents_tree_branch_key::XfsExtentsTreeBranchKey;
use super::extents_tree_branch_value::XfsExtentsTreeBranchValue;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) extents tree.
pub struct XfsExtentsTree {
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

impl XfsExtentsTree {
    /// Creates a new extents tree.
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
        read_block_numbers: &mut HashSet<u64>,
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
            keramics_core::debug_trace_structure!(XfsExtentsTreeBranchKey::debug_read_data(
                &btree_node.data[key_data_offset..]
            ));
            let mut branch_key: XfsExtentsTreeBranchKey = XfsExtentsTreeBranchKey::new();

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

            keramics_core::debug_trace_structure!(XfsExtentsTreeBranchValue::debug_read_data(
                &btree_node.data[value_data_offset..]
            ));
            let mut branch_value: XfsExtentsTreeBranchValue = XfsExtentsTreeBranchValue::new();

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
                read_block_numbers,
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
        read_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
        if read_block_numbers.contains(&block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Extent tree node: {} already read",
                block_number
            )));
        }
        let allocation_group_index: u64 = block_number >> self.block_number_bit_shift;
        let allocation_group_block_number: u64 =
            allocation_group_index * (self.allocation_group_size as u64);
        let relative_block_number: u64 = block_number & self.relative_block_number_bit_mask;

        let btree_node_offset: u64 =
            (allocation_group_block_number + relative_block_number) * (self.block_size as u64);

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
        read_block_numbers.insert(block_number);

        if btree_node.signature != XFS_EXTENT_TREE_SIGNATURE
            && btree_node.signature != XFS_EXTENT_TREE_V5_SIGNATURE
        {
            return Err(keramics_core::error_trace_new!(
                "Unsupported extent B-tree node signature"
            ));
        }
        if btree_node.is_branch() {
            match self.get_extents_from_branch_node(
                data_stream,
                &btree_node,
                extents,
                read_block_numbers,
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
            let extents_list: XfsExtentsList = XfsExtentsList::new();

            match extents_list.read_data(
                btree_node.number_of_records as u64,
                &btree_node.data[btree_node.records_offset..],
                extents,
            ) {
                Ok(_) => Ok(()),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read extents list");
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

        keramics_core::debug_trace_data!("XfsExtentsTreeRootNode", 0, &root_node_data, data_size);

        let mut read_block_numbers: HashSet<u64> = HashSet::new();

        keramics_core::debug_trace_structure!(XfsExtentsTreeBranchHeader::debug_read_data(
            &root_node_data
        ));
        let mut branch_header: XfsExtentsTreeBranchHeader = XfsExtentsTreeBranchHeader::new();

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
            keramics_core::debug_trace_structure!(XfsExtentsTreeBranchKey::debug_read_data(
                &root_node_data[key_data_offset..]
            ));
            let mut branch_key: XfsExtentsTreeBranchKey = XfsExtentsTreeBranchKey::new();

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

            keramics_core::debug_trace_structure!(XfsExtentsTreeBranchValue::debug_read_data(
                &root_node_data[value_data_offset..]
            ));
            let mut branch_value: XfsExtentsTreeBranchValue = XfsExtentsTreeBranchValue::new();

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
                &mut read_block_numbers,
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

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    /// Builds a synthetic 16-byte packed XFS extent for testing.
    fn build_packed_extent_bytes(
        logical_block_number: u64,
        physical_block_number: u64,
        number_of_blocks: u32,
    ) -> [u8; 16] {
        let upper: u64 = (logical_block_number << 9) | ((physical_block_number >> 43) & 0x1ff);
        let lower: u64 = ((physical_block_number & 0x7ffffffffff) << 21)
            | ((number_of_blocks as u64) & 0x1fffff);
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0..8].copy_from_slice(&upper.to_be_bytes());
        bytes[8..16].copy_from_slice(&lower.to_be_bytes());
        bytes
    }

    #[test]
    fn test_read_extents_v4_branch_and_leaf() -> Result<(), ErrorTrace> {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 4 * block_size];

        // Block 1: v4 branch node ("BMAP", level=1, number_of_records=1, pointing to block 2).
        let branch_end_offset: usize = 2 * block_size;
        let branch: &mut [u8] = &mut image[block_size..branch_end_offset];
        branch[0..4].copy_from_slice(XFS_EXTENT_TREE_SIGNATURE);
        branch[4..6].copy_from_slice(&1_u16.to_be_bytes());
        branch[6..8].copy_from_slice(&1_u16.to_be_bytes());
        let number_of_pairs: usize = (block_size - 24) / 16;
        let values_offset: usize = 24 + (number_of_pairs * 8);
        let values_end_offset: usize = values_offset + 8;
        branch[24..32].copy_from_slice(&0_u64.to_be_bytes());
        branch[values_offset..values_end_offset].copy_from_slice(&2_u64.to_be_bytes());

        // Block 2: v4 leaf node ("BMAP", level=0, number_of_records=1).
        let leaf_offset: usize = 2 * block_size;
        let leaf_end_offset: usize = 3 * block_size;
        let leaf: &mut [u8] = &mut image[leaf_offset..leaf_end_offset];
        leaf[0..4].copy_from_slice(XFS_EXTENT_TREE_SIGNATURE);
        leaf[4..6].copy_from_slice(&0_u16.to_be_bytes());
        leaf[6..8].copy_from_slice(&1_u16.to_be_bytes());
        let extent_bytes: [u8; 16] = build_packed_extent_bytes(0, 10, 4);
        leaf[24..40].copy_from_slice(&extent_bytes);

        // Root node inline data: header (level=2, number_of_records=1) + key (8) + value (block 1).
        let mut root_node_data: Vec<u8> = vec![0; 20];
        root_node_data[0..2].copy_from_slice(&2_u16.to_be_bytes());
        root_node_data[2..4].copy_from_slice(&1_u16.to_be_bytes());
        root_node_data[4..12].copy_from_slice(&0_u64.to_be_bytes());
        root_node_data[12..20].copy_from_slice(&1_u64.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let extents_tree: XfsExtentsTree = XfsExtentsTree::new(4, 16, 4, block_size as u32);
        let mut extents: Vec<XfsPackedExtent> = Vec::new();
        extents_tree.read_extents(&data_stream, &root_node_data, &mut extents)?;

        assert_eq!(extents.len(), 1);
        assert_eq!(extents[0].logical_block_number, 0);
        assert_eq!(extents[0].physical_block_number, 10);
        assert_eq!(extents[0].number_of_blocks, 4);

        Ok(())
    }

    #[test]
    fn test_read_extents_v5_leaf() -> Result<(), ErrorTrace> {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 3 * block_size];

        // Block 1: v5 leaf node ("BMA3", 72-byte header, level=0, number_of_records=1).
        let leaf_end_offset: usize = 2 * block_size;
        let leaf: &mut [u8] = &mut image[block_size..leaf_end_offset];
        leaf[0..4].copy_from_slice(XFS_EXTENT_TREE_V5_SIGNATURE);
        leaf[4..6].copy_from_slice(&0_u16.to_be_bytes());
        leaf[6..8].copy_from_slice(&1_u16.to_be_bytes());
        let extent_bytes: [u8; 16] = build_packed_extent_bytes(2, 20, 3);
        leaf[72..88].copy_from_slice(&extent_bytes);

        let mut root_node_data: Vec<u8> = vec![0; 20];
        root_node_data[0..2].copy_from_slice(&1_u16.to_be_bytes());
        root_node_data[2..4].copy_from_slice(&1_u16.to_be_bytes());
        root_node_data[12..20].copy_from_slice(&1_u64.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let extents_tree: XfsExtentsTree = XfsExtentsTree::new(5, 16, 4, block_size as u32);
        let mut extents: Vec<XfsPackedExtent> = Vec::new();
        extents_tree.read_extents(&data_stream, &root_node_data, &mut extents)?;

        // XfsExtentsList inserts a sparse extent for logical blocks 0..2 before block 2..5.
        assert_eq!(extents.len(), 2);
        assert_eq!(extents[0].logical_block_number, 0);
        assert_eq!(extents[0].number_of_blocks, 2);
        assert_eq!(extents[1].logical_block_number, 2);
        assert_eq!(extents[1].physical_block_number, 20);
        assert_eq!(extents[1].number_of_blocks, 3);

        Ok(())
    }

    #[test]
    fn test_read_extents_with_errors() {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 4 * block_size];

        // Block 1: unsupported signature.
        let block1_signature_end_offset: usize = block_size + 4;
        image[block_size..block1_signature_end_offset].copy_from_slice(b"XXXX");

        // Block 2: branch node with record count out of bounds.
        let block2_offset: usize = 2 * block_size;
        let block2_end_offset: usize = 3 * block_size;
        let invalid_count_branch: &mut [u8] = &mut image[block2_offset..block2_end_offset];
        invalid_count_branch[0..4].copy_from_slice(XFS_EXTENT_TREE_SIGNATURE);
        invalid_count_branch[4..6].copy_from_slice(&1_u16.to_be_bytes());
        invalid_count_branch[6..8].copy_from_slice(&100_u16.to_be_bytes());

        // Block 3: branch node with sub-block number 0.
        let block3_offset: usize = 3 * block_size;
        let block3_end_offset: usize = 4 * block_size;
        let zero_child_branch: &mut [u8] = &mut image[block3_offset..block3_end_offset];
        zero_child_branch[0..4].copy_from_slice(XFS_EXTENT_TREE_SIGNATURE);
        zero_child_branch[4..6].copy_from_slice(&1_u16.to_be_bytes());
        zero_child_branch[6..8].copy_from_slice(&1_u16.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let extents_tree: XfsExtentsTree = XfsExtentsTree::new(4, 16, 4, block_size as u32);
        let mut extents: Vec<XfsPackedExtent> = Vec::new();

        // Test truncated root header.
        let result: Result<(), ErrorTrace> =
            extents_tree.read_extents(&data_stream, &[0; 2], &mut extents);
        assert!(result.is_err());

        // Test root record count out of bounds.
        let mut invalid_root_count: Vec<u8> = vec![0; 20];
        invalid_root_count[2..4].copy_from_slice(&5_u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            extents_tree.read_extents(&data_stream, &invalid_root_count, &mut extents);
        assert!(result.is_err());

        // Test root pointing to block number 0.
        let mut zero_root: Vec<u8> = vec![0; 20];
        zero_root[2..4].copy_from_slice(&1_u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            extents_tree.read_extents(&data_stream, &zero_root, &mut extents);
        assert!(result.is_err());

        // Test unsupported node signature (block 1), out-of-bounds branch count (block 2),
        // zero branch child (block 3), and out-of-bounds stream offset (block 10).
        let block_numbers: [u64; 4] = [1, 2, 3, 10];
        for block_number in block_numbers {
            let mut root_node_data: Vec<u8> = vec![0; 20];
            root_node_data[2..4].copy_from_slice(&1_u16.to_be_bytes());
            root_node_data[12..20].copy_from_slice(&block_number.to_be_bytes());
            let result: Result<(), ErrorTrace> =
                extents_tree.read_extents(&data_stream, &root_node_data, &mut extents);
            assert!(result.is_err());
        }

        // Test duplicate block number (cycle detection).
        let mut read_block_numbers: HashSet<u64> = HashSet::new();
        read_block_numbers.insert(1);
        let result: Result<(), ErrorTrace> = extents_tree.get_extents_from_node(
            &data_stream,
            1,
            &mut extents,
            &mut read_block_numbers,
        );
        assert!(result.is_err());
    }
}
