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
use std::iter::Peekable;
use std::slice::IterMut;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u16_be;

use super::btree_node_descriptor::HfsBtreeNodeDescriptor;
use super::btree_node_record::HfsBtreeNodeRecord;
use super::enums::HfsBtreeNodeType;

/// Hierarchical File System (HFS) B-tree node.
pub struct HfsBtreeNode {
    /// Offset.
    pub offset: u64,

    /// Node type.
    pub node_type: HfsBtreeNodeType,

    /// Data.
    data: Vec<u8>,

    /// Records.
    pub records: Vec<HfsBtreeNodeRecord>,
}

impl HfsBtreeNode {
    /// Creates a new B-tree node.
    pub fn new() -> Self {
        Self {
            offset: 0,
            node_type: HfsBtreeNodeType::LeafNode,
            data: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Retrieves the data of a specific record.
    pub fn get_record_data_by_index(&self, record_index: usize) -> Option<&[u8]> {
        match self.records.get(record_index) {
            Some(node_record) => {
                let data_end_offset: usize = node_record.offset + node_record.size;

                Some(&self.data[node_record.offset..data_end_offset])
            }
            None => None,
        }
    }

    /// Retrieves the offset of a specific record.
    pub fn get_record_offset_by_index(&self, record_index: usize) -> u64 {
        match self.records.get(record_index) {
            Some(node_record) => self.offset + (node_record.offset as u64),
            None => self.offset,
        }
    }

    /// Reads the B-tree node descriptor from a buffer.
    fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        keramics_core::debug_trace_data_and_structure!(
            "HfsBtreeNodeDescriptor",
            self.offset,
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
        self.node_type = node_descriptor.node_type;

        let data_size: usize = data.len();
        let record_offsets_data_size: usize =
            ((node_descriptor.number_of_records as usize) + 1) * 2;

        if record_offsets_data_size > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid node - number of records value out of bounds"
            ));
        }
        let record_offsets_data_offset: usize = data_size - record_offsets_data_size;

        keramics_core::debug_trace_data!(
            "HfsBtreeNodeRecordOffsets",
            self.offset + (record_offsets_data_offset as u64),
            &data[record_offsets_data_offset..],
            record_offsets_data_size
        );
        // TODO: debug print record offsets

        // Note the the records offsets are stored back-to-front and not necessarily in order.
        let mut data_offset: usize = data_size - 2;

        for record_index in 0..node_descriptor.number_of_records {
            let record_offset: u16 = bytes_to_u16_be!(data, data_offset);
            data_offset -= 2;

            if record_offset < 14 || (record_offset as usize) >= record_offsets_data_offset {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid record: {} offset value out of bounds",
                    record_index
                )));
            }
            let node_record: HfsBtreeNodeRecord = HfsBtreeNodeRecord::new(record_offset);
            self.records.push(node_record);
        }
        // TODO: debug print unused (free) space offset.

        self.records.sort_by_key(|node_record| node_record.offset);

        let mut records_iterator: Peekable<IterMut<HfsBtreeNodeRecord>> =
            self.records.iter_mut().peekable();

        while let Some(node_record) = records_iterator.next() {
            let next_record_offset: usize = match records_iterator.peek() {
                Some(node_record) => node_record.offset as usize,
                None => record_offsets_data_offset,
            };
            node_record.size = next_record_offset - node_record.offset;
        }
        Ok(())
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
        offset: u64,
        size: u16,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; size as usize];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(offset)
        );
        self.offset = offset;

        match self.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read B-tree node data");
                return Err(error);
            }
        }
        self.data = data;

        Ok(())
    }
}
