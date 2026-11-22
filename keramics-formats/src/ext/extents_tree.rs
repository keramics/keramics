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

use super::block_range::{ExtBlockRange, ExtBlockRangeType};
use super::extent_descriptor::ExtExtentDescriptor;
use super::extent_index::ExtExtentIndex;
use super::extents_footer::ExtExtentsFooter;
use super::extents_header::ExtExtentsHeader;

/// Extended File System (ext) extents tree.
pub struct ExtExtentsTree {
    /// Block size.
    block_size: u32,

    /// Number of blocks.
    number_of_blocks: u64,
}

impl ExtExtentsTree {
    /// Creates new extents tree.
    pub fn new(block_size: u32, number_of_blocks: u64) -> Self {
        Self {
            block_size,
            number_of_blocks,
        }
    }

    /// Reads the block numbers tree from an inode data reference.
    pub fn read_data_reference(
        &mut self,
        data_reference: &[u8],
        data_stream: &DataStreamReference,
        block_ranges: &mut Vec<ExtBlockRange>,
    ) -> Result<(), ErrorTrace> {
        let mut logical_block_number: u64 = 0;

        match self.read_node_data(
            data_reference,
            data_stream,
            &mut logical_block_number,
            block_ranges,
            6,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read node");
                return Err(error);
            }
        }
        if logical_block_number < self.number_of_blocks {
            let block_range: ExtBlockRange = ExtBlockRange::new(
                logical_block_number,
                0,
                self.number_of_blocks - logical_block_number,
                ExtBlockRangeType::Sparse,
            );
            block_ranges.push(block_range);
        }
        Ok(())
    }

    /// Reads the extents tree node from a buffer.
    fn read_node_data(
        &mut self,
        data: &[u8],
        data_stream: &DataStreamReference,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> Result<(), ErrorTrace> {
        keramics_core::debug_trace_structure!(ExtExtentsHeader::debug_read_data(data));

        let mut extents_header: ExtExtentsHeader = ExtExtentsHeader::new();

        match extents_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extents header");
                return Err(error);
            }
        }
        if extents_header.depth >= parent_depth {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid depth: {} value out of bounds",
                extents_header.depth,
            )));
        }
        let mut data_offset: usize = 12;
        let data_size: usize = data.len();

        if extents_header.depth > 0 {
            for entry_index in 0..extents_header.number_of_entries {
                let data_end_offset: usize = data_offset + 12;

                if data_end_offset > data_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported extent index entry: {} data size",
                        entry_index
                    )));
                }
                keramics_core::debug_trace_structure!(ExtExtentIndex::debug_read_data(
                    &data[data_offset..]
                ));
                let mut entry: ExtExtentIndex = ExtExtentIndex::new();

                match entry.read_data(&data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read extent index entry: {}", entry_index)
                        );
                        return Err(error);
                    }
                }
                data_offset = data_end_offset;

                let sub_node_offset: u64 = entry.physical_block_number * (self.block_size as u64);

                match self.read_node_at_position(
                    data_stream,
                    SeekFrom::Start(sub_node_offset),
                    logical_block_number,
                    block_ranges,
                    extents_header.depth,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read sub node at offset {} (0x{:08x})",
                                sub_node_offset, sub_node_offset
                            )
                        );
                        return Err(error);
                    }
                }
            }
        } else {
            for entry_index in 0..extents_header.number_of_entries {
                let data_end_offset: usize = data_offset + 12;

                if data_end_offset > data_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported extent descriptor entry: {} data size",
                        entry_index
                    )));
                }
                keramics_core::debug_trace_structure!(ExtExtentDescriptor::debug_read_data(
                    &data[data_offset..]
                ));
                let mut entry: ExtExtentDescriptor = ExtExtentDescriptor::new();

                match entry.read_data(&data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read extent descriptor entry: {}", entry_index)
                        );
                        return Err(error);
                    }
                }
                data_offset = data_end_offset;

                if entry.logical_block_number as u64 > *logical_block_number {
                    let number_of_blocks: u64 =
                        (entry.logical_block_number as u64) - *logical_block_number;

                    let block_range: ExtBlockRange = ExtBlockRange::new(
                        *logical_block_number,
                        0,
                        number_of_blocks,
                        ExtBlockRangeType::Sparse,
                    );
                    block_ranges.push(block_range);

                    *logical_block_number = entry.logical_block_number as u64;
                }
                let mut number_of_blocks: u64 = entry.number_of_blocks as u64;
                let mut range_type: ExtBlockRangeType = if entry.physical_block_number == 0 {
                    ExtBlockRangeType::Sparse
                } else {
                    ExtBlockRangeType::InFile
                };
                if number_of_blocks > 32768 {
                    number_of_blocks -= 32768;
                    range_type = ExtBlockRangeType::Sparse;
                }
                let block_range: ExtBlockRange = ExtBlockRange::new(
                    entry.logical_block_number as u64,
                    entry.physical_block_number,
                    number_of_blocks,
                    range_type,
                );
                block_ranges.push(block_range);

                *logical_block_number = (entry.logical_block_number as u64) + number_of_blocks;
            }
        }
        if data_size - data_offset >= 4 {
            keramics_core::debug_trace_structure!(ExtExtentsFooter::debug_read_data(
                &data[data_offset..]
            ));
            let mut extents_footer: ExtExtentsFooter = ExtExtentsFooter::new();

            match extents_footer.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read extents footer");
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads the extents tree node from a specific position in a data stream.
    fn read_node_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
        logical_block_number: &mut u64,
        block_ranges: &mut Vec<ExtBlockRange>,
        parent_depth: u16,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; self.block_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("ExtExtentsTreeNode", offset, &data, self.block_size);

        match self.read_node_data(
            &data,
            data_stream,
            logical_block_number,
            block_ranges,
            parent_depth,
        ) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read node at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        vec![]
    }

    #[test]
    fn test_read_data_reference() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtExtentsTree::new(1024, 16);

        let test_data: Vec<u8> = get_test_data();
        let test_data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();

        let test_data: Vec<u8> = vec![
            0x0a, 0xf3, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x53, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        test_struct.read_data_reference(&test_data, &test_data_stream, &mut block_ranges)?;

        assert_eq!(block_ranges.len(), 2);

        let block_range: &ExtBlockRange = &block_ranges[0];
        assert_eq!(block_range.logical_block_number, 0);
        assert_eq!(block_range.physical_block_number, 1363);
        assert_eq!(block_range.number_of_blocks, 12);
        assert!(block_range.range_type == ExtBlockRangeType::InFile);

        let block_range: &ExtBlockRange = &block_ranges[1];
        assert_eq!(block_range.logical_block_number, 12);
        assert_eq!(block_range.physical_block_number, 0);
        assert_eq!(block_range.number_of_blocks, 4);
        assert!(block_range.range_type == ExtBlockRangeType::Sparse);

        Ok(())
    }

    #[test]
    fn test_read_node_data() -> Result<(), ErrorTrace> {
        let mut test_struct = ExtExtentsTree::new(1024, 16);

        let test_data: Vec<u8> = get_test_data();
        let test_data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut logical_block_number: u64 = 0;
        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();

        let test_data: Vec<u8> = vec![
            0x0a, 0xf3, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x53, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        test_struct.read_node_data(
            &test_data,
            &test_data_stream,
            &mut logical_block_number,
            &mut block_ranges,
            6,
        )?;
        assert_eq!(block_ranges.len(), 1);

        let block_range: &ExtBlockRange = &block_ranges[0];
        assert_eq!(block_range.logical_block_number, 0);
        assert_eq!(block_range.physical_block_number, 1363);
        assert_eq!(block_range.number_of_blocks, 12);
        assert!(block_range.range_type == ExtBlockRangeType::InFile);

        Ok(())
    }

    #[test]
    fn test_read_data_reference_with_index_node_and_sparse_extents() -> Result<(), ErrorTrace> {
        let block_size: usize = 1024;
        let mut image: Vec<u8> = vec![0; 2 * block_size];

        // Leaf node at block 1 (depth=0, 3 extent descriptors + 4-byte footer):
        // Descriptor 0: logical=2 (creates sparse gap 0..2), number_of_blocks=4, physical=10.
        // Descriptor 1: logical=6, number_of_blocks=32768 + 2 (uninitialized -> sparse), physical=20.
        // Descriptor 2: logical=8, number_of_blocks=2, physical=0 (sparse).
        let leaf: &mut [u8] = &mut image[block_size..2 * block_size];
        leaf[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        leaf[2..4].copy_from_slice(&3u16.to_le_bytes());
        leaf[4..6].copy_from_slice(&4u16.to_le_bytes());
        leaf[6..8].copy_from_slice(&0u16.to_le_bytes());

        // Descriptor 0 at offset 12.
        leaf[12..16].copy_from_slice(&2u32.to_le_bytes());
        leaf[16..18].copy_from_slice(&4u16.to_le_bytes());
        leaf[20..24].copy_from_slice(&10u32.to_le_bytes());

        // Descriptor 1 at offset 24 (uninitialized extent > 32768).
        leaf[24..28].copy_from_slice(&6u32.to_le_bytes());
        leaf[28..30].copy_from_slice(&(32768u16 + 2).to_le_bytes());
        leaf[32..36].copy_from_slice(&20u32.to_le_bytes());

        // Descriptor 2 at offset 36 (physical_block_number == 0).
        leaf[36..40].copy_from_slice(&8u32.to_le_bytes());
        leaf[40..42].copy_from_slice(&2u16.to_le_bytes());

        // Root index node (depth=1, 1 extent index pointing to block 1).
        let mut root_data: Vec<u8> = vec![0; 28];
        root_data[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        root_data[2..4].copy_from_slice(&1u16.to_le_bytes());
        root_data[4..6].copy_from_slice(&4u16.to_le_bytes());
        root_data[6..8].copy_from_slice(&1u16.to_le_bytes());
        // Extent index at offset 12: logical=0, leaf_low=1.
        root_data[16..20].copy_from_slice(&1u32.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut tree: ExtExtentsTree = ExtExtentsTree::new(block_size as u32, 12);
        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();
        tree.read_data_reference(&root_data, &data_stream, &mut block_ranges)?;

        assert_eq!(block_ranges.len(), 5);
        assert!(block_ranges[0].range_type == ExtBlockRangeType::Sparse);
        assert_eq!(block_ranges[0].number_of_blocks, 2);
        assert!(block_ranges[1].range_type == ExtBlockRangeType::InFile);
        assert_eq!(block_ranges[1].physical_block_number, 10);
        assert!(block_ranges[2].range_type == ExtBlockRangeType::Sparse);
        assert_eq!(block_ranges[2].number_of_blocks, 2);
        assert!(block_ranges[3].range_type == ExtBlockRangeType::Sparse);
        assert!(block_ranges[4].range_type == ExtBlockRangeType::Sparse);
        assert_eq!(block_ranges[4].logical_block_number, 10);

        Ok(())
    }

    #[test]
    fn test_read_data_reference_with_errors() {
        let empty_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut tree: ExtExtentsTree = ExtExtentsTree::new(1024, 16);
        let mut block_ranges: Vec<ExtBlockRange> = Vec::new();

        // Test invalid header signature.
        let result: Result<(), ErrorTrace> =
            tree.read_data_reference(&[0; 12], &empty_stream, &mut block_ranges);
        assert!(result.is_err());

        // Test depth >= parent_depth (6).
        let mut invalid_depth: Vec<u8> = vec![0; 12];
        invalid_depth[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        invalid_depth[6..8].copy_from_slice(&6u16.to_le_bytes());
        let result: Result<(), ErrorTrace> =
            tree.read_data_reference(&invalid_depth, &empty_stream, &mut block_ranges);
        assert!(result.is_err());

        // Test truncated extent index entry (depth=1, number_of_entries=1, only 12 bytes).
        let mut truncated_index: Vec<u8> = vec![0; 12];
        truncated_index[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        truncated_index[2..4].copy_from_slice(&1u16.to_le_bytes());
        truncated_index[6..8].copy_from_slice(&1u16.to_le_bytes());
        let result: Result<(), ErrorTrace> =
            tree.read_data_reference(&truncated_index, &empty_stream, &mut block_ranges);
        assert!(result.is_err());

        // Test truncated extent descriptor entry (depth=0, number_of_entries=1, only 12 bytes).
        let mut truncated_descriptor: Vec<u8> = vec![0; 12];
        truncated_descriptor[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        truncated_descriptor[2..4].copy_from_slice(&1u16.to_le_bytes());
        let result: Result<(), ErrorTrace> =
            tree.read_data_reference(&truncated_descriptor, &empty_stream, &mut block_ranges);
        assert!(result.is_err());

        // Test out-of-bounds sub-node read from empty stream.
        let mut missing_sub_node: Vec<u8> = vec![0; 24];
        missing_sub_node[0..2].copy_from_slice(&0xf30au16.to_le_bytes());
        missing_sub_node[2..4].copy_from_slice(&1u16.to_le_bytes());
        missing_sub_node[6..8].copy_from_slice(&1u16.to_le_bytes());
        missing_sub_node[16..20].copy_from_slice(&5u32.to_le_bytes());
        let result: Result<(), ErrorTrace> =
            tree.read_data_reference(&missing_sub_node, &empty_stream, &mut block_ranges);
        assert!(result.is_err());
    }
}
