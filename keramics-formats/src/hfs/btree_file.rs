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

use super::block_range::HfsBlockRange;
use super::btree_header_record::HfsBtreeHeaderRecord;
use super::btree_node::HfsBtreeNode;
use super::btree_node_descriptor::HfsBtreeNodeDescriptor;
use super::enums::{HfsBtreeNodeType, HfsFormat, HfsKeyComparisonMethod};

/// Hierarchical File System (HFS) B-tree file.
pub struct HfsBtreeFile {
    /// Format.
    pub format: HfsFormat,

    /// Block size.
    block_size: u32,

    /// Node size.
    node_size: u16,

    /// Root node number.
    pub root_node_number: u32,

    /// Size.
    size: u64,

    /// Block ranges.
    block_ranges: Vec<HfsBlockRange>,

    /// Key comparision method.
    pub key_comparion_method: HfsKeyComparisonMethod,
}

impl HfsBtreeFile {
    /// Creates a new B-tree file.
    pub fn new() -> Self {
        Self {
            format: HfsFormat::HfsPlus,
            block_size: 0,
            node_size: 0,
            root_node_number: 0,
            size: 0,
            block_ranges: Vec::new(),
            key_comparion_method: HfsKeyComparisonMethod::CaseFold,
        }
    }

    /// Retrieves a specific node.
    pub fn get_node_by_number(
        &self,
        data_stream: &DataStreamReference,
        node_number: u32,
    ) -> Result<HfsBtreeNode, ErrorTrace> {
        let node_logical_offset: u64 = (node_number as u64) * (self.node_size as u64);

        // TODO: optimize block range lookup
        let block_range_index: usize = match self.block_ranges.iter().position(|block_range| {
            let range_logical_end_offset: u64 = ((block_range.logical_block_number as u64)
                + (block_range.number_of_blocks as u64))
                * (self.block_size as u64);

            node_logical_offset < range_logical_end_offset
        }) {
            Some(index) => index,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid node number: {} value out of bounds",
                    node_number
                )));
            }
        };
        let node_physical_offset: u64 = match self.block_ranges.get(block_range_index) {
            Some(block_range) => {
                let range_logical_offset: u64 =
                    (block_range.logical_block_number as u64) * (self.block_size as u64);
                let range_physical_offset: u64 =
                    (block_range.physical_block_number as u64) * (self.block_size as u64);

                range_physical_offset + (node_logical_offset - range_logical_offset)
            }
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve block range: {}",
                    block_range_index
                )));
            }
        };
        let mut node: HfsBtreeNode = HfsBtreeNode::new();

        match node.read_data_stream(data_stream, node_physical_offset, self.node_size) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read node: {} at offset: {} (0x{:08x}",
                        node_number, node_physical_offset, node_physical_offset
                    )
                );
                return Err(error);
            }
        }
        Ok(node)
    }

    /// Initializes the B-tree file.
    pub fn initialize(
        &mut self,
        format: &HfsFormat,
        block_size: u32,
        size: u64,
        block_ranges: Vec<HfsBlockRange>,
    ) {
        self.format = format.clone();
        self.block_size = block_size;
        self.size = size;
        self.block_ranges = block_ranges;
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let block_range: &HfsBlockRange = match self.block_ranges.first() {
            Some(block_range) => block_range,
            None => {
                return Err(keramics_core::error_trace_new!("Missing first block range"));
            }
        };
        let header_record_offset: u64 =
            (block_range.physical_block_number as u64) * (self.block_size as u64);

        let mut data: [u8; 512] = [0; 512];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(header_record_offset)
        );
        keramics_core::debug_trace_data_and_structure!(
            "HfsBtreeNodeDescriptor",
            header_record_offset,
            &data[0..14],
            14,
            HfsBtreeNodeDescriptor::debug_read_data(&data)
        );
        let mut node_descriptor: HfsBtreeNodeDescriptor = HfsBtreeNodeDescriptor::new();

        match node_descriptor.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read node descriptor");
                return Err(error);
            }
        }
        if node_descriptor.node_type != HfsBtreeNodeType::HeaderNode {
            return Err(keramics_core::error_trace_new!(
                "Unsupported node type in first node descriptor"
            ));
        }
        keramics_core::debug_trace_data_and_structure!(
            "HfsBtreeHeaderRecord",
            header_record_offset + 14,
            &data[14..120],
            106,
            HfsBtreeHeaderRecord::debug_read_data(&data[14..])
        );
        let mut header_record: HfsBtreeHeaderRecord = HfsBtreeHeaderRecord::new();

        match header_record.read_data(&data[14..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read header record");
                return Err(error);
            }
        }
        self.node_size = header_record.node_size;
        self.root_node_number = header_record.root_node_number;

        if self.format == HfsFormat::HfsX {
            self.key_comparion_method = match header_record.key_comparion_method {
                0x00 | 0xbc => HfsKeyComparisonMethod::Binary,
                0xcf => HfsKeyComparisonMethod::CaseFold,
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported key comparision method"
                    ));
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x13, 0x00, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    // TODO: add tests for get_node_by_number

    #[test]
    fn test_initialize() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let test_data_size: u64 = test_data.len() as u64;
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let block_ranges: Vec<HfsBlockRange> = vec![HfsBlockRange::new(0, 0, 1)];
        let mut test_struct = HfsBtreeFile::new();
        test_struct.initialize(&HfsFormat::HfsPlus, 4096, test_data_size, block_ranges);

        assert_eq!(test_struct.format, HfsFormat::HfsPlus);
        assert_eq!(test_struct.size, test_data_size);
        assert_eq!(test_struct.block_ranges.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let test_data_size: u64 = test_data.len() as u64;
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let block_ranges: Vec<HfsBlockRange> = vec![HfsBlockRange::new(0, 0, 1)];
        let mut test_struct = HfsBtreeFile::new();
        test_struct.initialize(&HfsFormat::HfsPlus, 4096, test_data_size, block_ranges);
        test_struct.read_data_stream(&data_stream)?;

        assert_eq!(test_struct.node_size, 4096);
        assert_eq!(test_struct.root_node_number, 0);

        Ok(())
    }
}
