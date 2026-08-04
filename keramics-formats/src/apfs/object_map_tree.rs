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
use keramics_types::bytes_to_u64_le;

use super::btree_node::ApfsBtreeNode;
use super::object_map_key::ApfsObjectMapKey;
use super::object_map_value::ApfsObjectMapValue;

/// Apple File System (APFS) object map B-tree.
pub struct ApfsObjectMapTree {
    /// Block size.
    pub block_size: u64,

    /// Root (node) block number.
    pub root_block_number: u64,
}

impl ApfsObjectMapTree {
    /// Creates a new object map B-tree.
    pub fn new() -> Self {
        Self {
            block_size: 0,
            root_block_number: 0,
        }
    }

    /// Retrieves a specific object map value.
    pub fn get_value_by_object_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_identifier: u64,
        object_transaction_identifier: u64,
    ) -> Result<Option<ApfsObjectMapValue>, ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_value_by_object_identifier_from_node(
            data_stream,
            self.root_block_number,
            object_identifier,
            object_transaction_identifier,
            &mut read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve value from root node: {}",
                        self.root_block_number
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific object map value from a node.
    fn get_value_by_object_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
        object_identifier: u64,
        object_transaction_identifier: u64,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<Option<ApfsObjectMapValue>, ErrorTrace> {
        if read_node_block_numbers.contains(&block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Node: {} already read",
                block_number
            )));
        }
        let node: ApfsBtreeNode = match self.get_node_by_number(data_stream, block_number) {
            Ok(node) => node,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve node: {}", block_number)
                );
                return Err(error);
            }
        };
        let expected_object_type: u32 = if block_number == self.root_block_number {
            0x40000002
        } else {
            0x40000003
        };
        if node.object_header.object_type != expected_object_type {
            return Err(keramics_core::error_trace_new!("Unsupported object type"));
        }
        if node.object_header.object_subtype != 0x0000000b {
            return Err(keramics_core::error_trace_new!(
                "Unsupported object subtype"
            ));
        }
        if node.node_header.flags & 0x0004 == 0 {
            return Err(keramics_core::error_trace_new!("Unsupported flags"));
        }
        if block_number == self.root_block_number {
            match &node.footer {
                Some(footer) => {
                    if footer.node_size != 4096 {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported footer node size"
                        ));
                    }
                    // TODO: compare key and value size
                }
                None => {
                    return Err(keramics_core::error_trace_new!("Missing footer"));
                }
            }
        }
        let mut last_key: ApfsObjectMapKey = ApfsObjectMapKey::new();
        let mut last_entry_index: usize = 0;

        let mut entry_index: usize = 0;
        let number_of_entries: usize = node.entries.len();

        while entry_index < number_of_entries {
            let key_data: &[u8] = match node.get_key_data_by_index(entry_index) {
                Some(key_data) => key_data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve key data of entry: {}",
                        entry_index
                    )));
                }
            };
            keramics_core::debug_trace_structure!(ApfsObjectMapKey::debug_read_data(key_data));

            let mut key: ApfsObjectMapKey = ApfsObjectMapKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if key.object_identifier > object_identifier {
                break;
            }
            if key.object_identifier == object_identifier
                && key.object_transaction_identifier > object_transaction_identifier
            {
                break;
            }
            last_key = key;
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if node.is_branch() {
            let value_data: &[u8] = match node.get_value_data_by_index(last_entry_index) {
                Some(value_data) => value_data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve value data of entry: {}",
                        last_entry_index
                    )));
                }
            };
            if value_data.len() < 16 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported value data size"
                ));
            }
            let sub_node_block_number: u64 = bytes_to_u64_le!(value_data, 0);

            match self.get_value_by_object_identifier_from_node(
                data_stream,
                sub_node_block_number,
                object_identifier,
                object_transaction_identifier,
                read_node_block_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve value from node: {}",
                            sub_node_block_number
                        )
                    );
                    Err(error)
                }
            }
        } else {
            if last_key.object_identifier != object_identifier {
                Ok(None)
            } else {
                let value_data: &[u8] = match node.get_value_data_by_index(last_entry_index) {
                    Some(value_data) => value_data,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve value data of entry: {}",
                            last_entry_index
                        )));
                    }
                };
                keramics_core::debug_trace_structure!(ApfsObjectMapValue::debug_read_data(
                    value_data
                ));
                let mut value: ApfsObjectMapValue = ApfsObjectMapValue::new();

                match value.read_data(&value_data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read value");
                        return Err(error);
                    }
                }
                Ok(Some(value))
            }
        }
    }

    /// Retrieves a specific node.
    fn get_node_by_number(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
    ) -> Result<ApfsBtreeNode, ErrorTrace> {
        let node_offset: u64 = block_number * self.block_size;

        let mut node: ApfsBtreeNode = ApfsBtreeNode::new();

        match node.read_at_position(&data_stream, SeekFrom::Start(node_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read object map B-tree root node at offset: {} (0x{:08x}))",
                        node_offset, node_offset
                    )
                );
                return Err(error);
            }
        }
        Ok(node)
    }

    /// Initializes the object map B-tree.
    pub fn initialize(
        &mut self,
        block_size: u32,
        root_block_number: u64,
    ) -> Result<(), ErrorTrace> {
        self.block_size = block_size as u64;
        self.root_block_number = root_block_number;

        Ok(())
    }
}
