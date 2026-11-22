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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u32_be;

use super::block_range::HfsBlockRange;
use super::btree_file::HfsBtreeFile;
use super::btree_node::HfsBtreeNode;
use super::enums::{HfsBtreeNodeType, HfsFormat};
use super::extent_descriptor::HfsExtentDescriptor;
use super::extent_descriptor_extended::HfsExtendedExtentDescriptor;
use super::extent_descriptor_standard::HfsStandardExtentDescriptor;
use super::extents_overflow_key::HfsExtentsOverflowKey;

/// Hierarchical File System (HFS) extents overflow file.
pub struct HfsExtentsOverflowFile {
    /// B-tree file.
    btree_file: HfsBtreeFile,
}

impl HfsExtentsOverflowFile {
    /// Creates a new extents overflow file.
    pub fn new() -> Self {
        Self {
            btree_file: HfsBtreeFile::new(),
        }
    }

    /// Retrieves extents.
    pub fn get_extents_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        identifier: u32,
        extents: &mut Vec<HfsExtentDescriptor>,
    ) -> Result<(), ErrorTrace> {
        if self.btree_file.root_node_number == 0 {
            return Ok(());
        }
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_extents_by_identifier_from_node(
            data_stream,
            self.btree_file.root_node_number,
            identifier,
            extents,
            &mut read_node_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve extents from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves extents from a node.
    pub fn get_extents_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        node_number: u32,
        identifier: u32,
        extents: &mut Vec<HfsExtentDescriptor>,
        read_node_numbers: &mut HashSet<u32>,
    ) -> Result<(), ErrorTrace> {
        if read_node_numbers.contains(&node_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Node: {} already read",
                node_number
            )));
        }
        let node: HfsBtreeNode = match self.btree_file.get_node_by_number(data_stream, node_number)
        {
            Ok(node) => node,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve node: {}", node_number)
                );
                return Err(error);
            }
        };
        read_node_numbers.insert(node_number);

        let is_branch: bool = match &node.node_type {
            HfsBtreeNodeType::HeaderNode | HfsBtreeNodeType::IndexNode => true,
            HfsBtreeNodeType::LeafNode => false,
            _ => {
                return Err(keramics_core::error_trace_new!("Unsupported node type"));
            }
        };
        let mut last_key: HfsExtentsOverflowKey = HfsExtentsOverflowKey::new();
        let mut last_record_data: &[u8] = &[];

        let mut record_index: usize = 0;
        let number_of_records: usize = node.records.len();

        while record_index < number_of_records {
            let record_data: &[u8] = match node.get_record_data_by_index(record_index) {
                Some(record_data) => record_data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve data of record: {}",
                        record_index
                    )));
                }
            };
            keramics_core::debug_trace_data_and_structure!(
                format!("HfsExtentsOverflowKey of record: {}", record_index),
                node.get_record_offset_by_index(record_index),
                record_data,
                record_data.len(),
                HfsExtentsOverflowKey::debug_read_data(&self.btree_file.format, record_data)
            );
            let mut key: HfsExtentsOverflowKey = HfsExtentsOverflowKey::new();

            match key.read_data(&self.btree_file.format, record_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read key: {}", record_index)
                    );
                    return Err(error);
                }
            }
            if key.identifier > identifier {
                break;
            }
            if key.identifier == identifier {
                if !is_branch {
                    match self.read_extents_overflow_record(record_data, extents) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read extents overflow record: {}", record_index)
                            );
                            return Err(error);
                        }
                    }
                } else if record_index > 0 {
                    let data_offset: usize = last_key.size;

                    if data_offset + 4 > last_record_data.len() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid data size of record: {} value out of bounds",
                            record_index
                        )));
                    }
                    keramics_core::debug_trace_data!(
                        format!("HfsExtentsOverflowBranchNodeValue: {}", record_index),
                        node.get_record_offset_by_index(record_index - 1),
                        &last_record_data[data_offset..data_offset + 4],
                        4
                    );
                    let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

                    match self.get_extents_by_identifier_from_node(
                        data_stream,
                        sub_node_number,
                        identifier,
                        extents,
                        read_node_numbers,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve extents from node: {}",
                                    sub_node_number
                                )
                            );
                            return Err(error);
                        }
                    }
                }
            }
            record_index += 1;

            last_key = key;
            last_record_data = record_data;
        }
        if is_branch && record_index > 0 {
            let data_offset: usize = last_key.size;

            if data_offset + 4 > last_record_data.len() {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid data size of record: {} value out of bounds",
                    record_index
                )));
            }
            keramics_core::debug_trace_data!(
                format!("HfsExtentsOverflowBranchNodeValue: {}", record_index),
                node.get_record_offset_by_index(record_index - 1),
                &last_record_data[data_offset..data_offset + 4],
                4
            );
            let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

            match self.get_extents_by_identifier_from_node(
                data_stream,
                sub_node_number,
                identifier,
                extents,
                read_node_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve extents from node: {}", sub_node_number)
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Initializes the extents overflow file.
    pub fn initialize(
        &mut self,
        format: &HfsFormat,
        block_size: u32,
        size: u64,
        block_ranges: Vec<HfsBlockRange>,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        self.btree_file
            .initialize(format, block_size, size, block_ranges);

        match self.btree_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read B-tree file");
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads an extents overflow record.
    pub fn read_extents_overflow_record(
        &self,
        record_data: &[u8],
        extents: &mut Vec<HfsExtentDescriptor>,
    ) -> Result<(), ErrorTrace> {
        match &self.btree_file.format {
            HfsFormat::Hfs => {
                if record_data.len() < 20 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid record data size value out of bounds"
                    )));
                }
                for data_offset in (8..20).step_by(4) {
                    let data_end_offset: usize = data_offset + 4;

                    if record_data[data_offset..data_end_offset] == [0; 4] {
                        break;
                    }
                    let mut extent_descriptor: HfsExtentDescriptor = HfsExtentDescriptor::new();

                    match HfsStandardExtentDescriptor::read_data(
                        &mut extent_descriptor,
                        &record_data[data_offset..data_end_offset],
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read extent descriptor at offset: {} (0x{:08x})",
                                    data_offset, data_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                    extents.push(extent_descriptor);
                }
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                if record_data.len() < 76 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid record data size value out of bounds"
                    )));
                }
                for data_offset in (12..76).step_by(8) {
                    let data_end_offset: usize = data_offset + 8;

                    if record_data[data_offset..data_end_offset] == [0; 8] {
                        break;
                    }
                    let mut extent_descriptor: HfsExtentDescriptor = HfsExtentDescriptor::new();

                    match HfsExtendedExtentDescriptor::read_data(
                        &mut extent_descriptor,
                        &record_data[data_offset..data_end_offset],
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read extent descriptor at offset: {} (0x{:08x})",
                                    data_offset, data_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                    extents.push(extent_descriptor);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{open_fake_data_stream, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_data_stream() -> Result<DataStreamReference, ErrorTrace> {
        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        Ok(data_stream)
    }

    fn get_extents_overflow_file(
        data_stream: &DataStreamReference,
    ) -> Result<HfsExtentsOverflowFile, ErrorTrace> {
        let mut extents_overflow_file: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();

        extents_overflow_file.initialize(
            &HfsFormat::HfsPlus,
            4096,
            81920,
            vec![HfsBlockRange::new(0, 2, 20)],
            data_stream,
        )?;
        Ok(extents_overflow_file)
    }

    #[test]
    fn test_get_extents_by_identifier() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;
        let test_struct: HfsExtentsOverflowFile = get_extents_overflow_file(&data_stream)?;

        let mut extents: Vec<HfsExtentDescriptor> = Vec::new();
        test_struct.get_extents_by_identifier(&data_stream, 34, &mut extents)?;
        assert_eq!(extents.len(), 0);

        Ok(())
    }

    fn build_hfs_btree_node(node_type: u8, records: &[&[u8]]) -> Vec<u8> {
        let mut node: Vec<u8> = vec![0; 512];
        node[8] = node_type;
        node[10..12].copy_from_slice(&(records.len() as u16).to_be_bytes());

        let mut data_offset: usize = 14;
        for (index, record) in records.iter().enumerate() {
            node[data_offset..data_offset + record.len()].copy_from_slice(record);
            let slot_offset: usize = 512 - ((index + 1) * 2);
            node[slot_offset..slot_offset + 2].copy_from_slice(&(data_offset as u16).to_be_bytes());
            data_offset += record.len();
        }
        let free_slot_offset: usize = 512 - ((records.len() + 1) * 2);
        node[free_slot_offset..free_slot_offset + 2]
            .copy_from_slice(&(data_offset as u16).to_be_bytes());
        node
    }

    fn build_header_node(root_node_number: u32) -> Vec<u8> {
        let mut node: Vec<u8> = vec![0; 512];
        node[8] = 0x01;
        node[16..20].copy_from_slice(&root_node_number.to_be_bytes());
        node[32..34].copy_from_slice(&512u16.to_be_bytes());
        node
    }

    #[test]
    fn test_get_extents_by_identifier_branch_and_leaf() -> Result<(), ErrorTrace> {
        let mut image: Vec<u8> = vec![0; 5 * 512];
        image[0..512].copy_from_slice(&build_header_node(1));

        // Build index records (12-byte HFS+ key + 4-byte sub-node number).
        // Entry 0 (id 0x10) points to empty leaf node 4; entry 1 (id 0x20) points to leaf node 2.
        let mut index_record_1: Vec<u8> = vec![
            0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
        ];
        index_record_1.extend_from_slice(&4u32.to_be_bytes());

        let mut index_record_2: Vec<u8> = vec![
            0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
        ];
        index_record_2.extend_from_slice(&2u32.to_be_bytes());

        let mut index_record_3: Vec<u8> = vec![
            0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00,
        ];
        index_record_3.extend_from_slice(&3u32.to_be_bytes());

        let index_node: Vec<u8> =
            build_hfs_btree_node(0x00, &[&index_record_1, &index_record_2, &index_record_3]);
        image[512..1024].copy_from_slice(&index_node);

        // Build leaf node 2 with two records: identifier 0x20 and identifier 0x30.
        let mut leaf_record_1: Vec<u8> = vec![0; 76];
        leaf_record_1[0..12].copy_from_slice(&[
            0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
        ]);
        leaf_record_1[12..16].copy_from_slice(&100u32.to_be_bytes());
        leaf_record_1[16..20].copy_from_slice(&4u32.to_be_bytes());
        leaf_record_1[20..24].copy_from_slice(&200u32.to_be_bytes());
        leaf_record_1[24..28].copy_from_slice(&2u32.to_be_bytes());

        let mut leaf_record_2: Vec<u8> = vec![0; 76];
        leaf_record_2[0..12].copy_from_slice(&[
            0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00,
        ]);
        let leaf_node: Vec<u8> = build_hfs_btree_node(0xff, &[&leaf_record_1, &leaf_record_2]);
        image[1024..1536].copy_from_slice(&leaf_node);

        // Node 3: MapNode (unsupported node type).
        image[1536..2048].copy_from_slice(&build_hfs_btree_node(0x02, &[]));

        // Node 4: Empty LeafNode.
        image[2048..2560].copy_from_slice(&build_hfs_btree_node(0xff, &[]));

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut extents_overflow_file: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();
        extents_overflow_file.initialize(
            &HfsFormat::HfsPlus,
            512,
            image.len() as u64,
            vec![HfsBlockRange::new(0, 0, 5)],
            &data_stream,
        )?;

        // Lookup identifier 0x20 traverses index node 1 -> leaf node 2 (stops at 0x30).
        let mut extents: Vec<HfsExtentDescriptor> = Vec::new();
        extents_overflow_file.get_extents_by_identifier(&data_stream, 0x20, &mut extents)?;
        assert_eq!(extents.len(), 2);
        assert_eq!(extents[0].block_number, 100);
        assert_eq!(extents[0].number_of_blocks, 4);
        assert_eq!(extents[1].block_number, 200);
        assert_eq!(extents[1].number_of_blocks, 2);

        // Lookup identifier 0x40 reaches MapNode 3 and returns an error.
        let mut error_extents: Vec<HfsExtentDescriptor> = Vec::new();
        let result: Result<(), ErrorTrace> =
            extents_overflow_file.get_extents_by_identifier(&data_stream, 0x40, &mut error_extents);
        assert!(result.is_err());

        // Test cycle detection when node was already read.
        let mut read_node_numbers: HashSet<u32> = HashSet::new();
        read_node_numbers.insert(2);
        let result: Result<(), ErrorTrace> = extents_overflow_file
            .get_extents_by_identifier_from_node(
                &data_stream,
                2,
                0x20,
                &mut error_extents,
                &mut read_node_numbers,
            );
        assert!(result.is_err());

        // Test root_node_number == 0 returns Ok(()).
        let empty_file: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();
        empty_file.get_extents_by_identifier(&data_stream, 0x20, &mut error_extents)?;

        Ok(())
    }

    #[test]
    fn test_read_extents_overflow_record() -> Result<(), ErrorTrace> {
        // Test standard HFS record (20 bytes: 8-byte key + three 4-byte descriptors).
        let mut hfs_file: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();
        hfs_file.btree_file.format = HfsFormat::Hfs;
        let mut hfs_record: Vec<u8> = vec![0; 20];
        hfs_record[8..10].copy_from_slice(&12u16.to_be_bytes());
        hfs_record[10..12].copy_from_slice(&3u16.to_be_bytes());
        let mut extents: Vec<HfsExtentDescriptor> = Vec::new();
        hfs_file.read_extents_overflow_record(&hfs_record, &mut extents)?;
        assert_eq!(extents.len(), 1);
        assert_eq!(extents[0].block_number, 12);
        assert_eq!(extents[0].number_of_blocks, 3);

        // Test truncated HFS record error.
        let result: Result<(), ErrorTrace> =
            hfs_file.read_extents_overflow_record(&[0; 10], &mut extents);
        assert!(result.is_err());

        // Test truncated HFS+ record error.
        let mut hfsplus_file: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();
        hfsplus_file.btree_file.format = HfsFormat::HfsPlus;
        let result: Result<(), ErrorTrace> =
            hfsplus_file.read_extents_overflow_record(&[0; 20], &mut extents);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_initialize() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;

        let mut test_struct: HfsExtentsOverflowFile = HfsExtentsOverflowFile::new();
        test_struct.initialize(
            &HfsFormat::HfsPlus,
            4096,
            81920,
            vec![HfsBlockRange::new(0, 2, 20)],
            &data_stream,
        )
    }
}
