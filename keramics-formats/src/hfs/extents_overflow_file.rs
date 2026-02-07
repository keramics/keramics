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

    /// Retieves extents.
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
            if !is_branch {
                if key.identifier == identifier {
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
                }
            } else if record_index > 0 {
                if key.identifier >= identifier {
                    let data_offset: usize = last_key.size as usize;

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
                if key.identifier > identifier {
                    break;
                }
            }
            record_index += 1;

            last_key = key;
            last_record_data = record_data;
        }
        if is_branch {
            if record_index == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid record index value out of bounds"
                ));
            }
            let data_offset: usize = last_key.size as usize;

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
                    let data_end_offset = data_offset + 4;

                    if &record_data[data_offset..data_end_offset] == [0; 4] {
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
                    let data_end_offset = data_offset + 8;

                    if &record_data[data_offset..data_end_offset] == [0; 8] {
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

    use keramics_core::open_os_data_stream;

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

    // TODO: add tests for get_extents_by_identifier_from_node

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

    // TODO: add tests for read_attribute_record
}
