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

use crate::lru_cache::SharedLruCache;

use super::btree_node::ApfsBtreeNode;
use super::object_map_key::ApfsObjectMapKey;
use super::object_map_value::ApfsObjectMapValue;

/// Apple File System (APFS) object map B-tree.
pub struct ApfsObjectMapTree {
    /// Block size.
    pub block_size: u32,

    /// Root (node) block number.
    pub root_block_number: u64,

    /// Node cache.
    node_cache: SharedLruCache<u64, ApfsBtreeNode>,
}

impl ApfsObjectMapTree {
    /// Creates a new object map B-tree.
    pub fn new() -> Self {
        Self {
            block_size: 0,
            root_block_number: 0,
            node_cache: SharedLruCache::new(64),
        }
    }

    /// Checks a node.
    fn check_node(&self, block_number: u64, node: &ApfsBtreeNode) -> Result<(), ErrorTrace> {
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
        Ok(())
    }

    /// Retrieves a specific node.
    fn get_node_by_number(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
    ) -> Result<ApfsBtreeNode, ErrorTrace> {
        let result: bool = match self.node_cache.contains(&block_number) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to determine if node: {} is cached", block_number)
                );
                return Err(error);
            }
        };
        if !result {
            let node_offset: u64 = block_number * (self.block_size as u64);

            let mut node: ApfsBtreeNode = ApfsBtreeNode::new();

            match node.read_at_position(None, data_stream, SeekFrom::Start(node_offset)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read node: {} at offset: {} (0x{:08x}))",
                            block_number, node_offset, node_offset
                        )
                    );
                    return Err(error);
                }
            }
            match self.check_node(block_number, &node) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Check of node: {} failed", block_number)
                    );
                    return Err(error);
                }
            }
            match self.node_cache.insert(block_number, node) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to insert node: {} in cache", block_number)
                    );
                    return Err(error);
                }
            }
        }
        match self.node_cache.get(&block_number) {
            Ok(Some(node)) => Ok(node),
            Ok(None) => Err(keramics_core::error_trace_new!(format!(
                "Missing node: {}",
                block_number
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve node: {} from cache", block_number)
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific object map value.
    pub fn get_value_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_identifier: u64,
        object_transaction_identifier: u64,
    ) -> Result<Option<ApfsObjectMapValue>, ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_value_data_by_identifier_from_node(
            data_stream,
            self.root_block_number,
            object_identifier,
            object_transaction_identifier,
            &mut read_node_block_numbers,
        ) {
            Ok(Some(value_data)) => {
                keramics_core::debug_trace_structure!(ApfsObjectMapValue::debug_read_data(
                    &value_data
                ));
                let mut value: ApfsObjectMapValue = ApfsObjectMapValue::new();

                match value.read_data(&value_data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read object map value"
                        );
                        return Err(error);
                    }
                }
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve value data from root node: {}",
                        self.root_block_number
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves specific value data from a node.
    fn get_value_data_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
        object_identifier: u64,
        object_transaction_identifier: u64,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<Option<Vec<u8>>, ErrorTrace> {
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
        read_node_block_numbers.insert(block_number);

        let mut last_key: ApfsObjectMapKey = ApfsObjectMapKey::new();
        let mut last_entry_index: usize = 0;

        let mut entry_index: usize = 0;
        let number_of_entries: usize = node.entries.len();

        while entry_index < number_of_entries {
            let key_data: &[u8] = match node.get_key_data_by_index(entry_index) {
                Some(key_data) => key_data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve entry: {} key data",
                        entry_index
                    )));
                }
            };
            keramics_core::debug_trace_structure!(ApfsObjectMapKey::debug_read_data(key_data));

            let mut key: ApfsObjectMapKey = ApfsObjectMapKey::new();

            match key.read_data(key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if (key.object_identifier > object_identifier)
                || (key.object_identifier == object_identifier
                    && key.object_transaction_identifier > object_transaction_identifier)
            {
                break;
            }
            last_key = key;
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if node.is_branch() {
            match self.get_value_data_by_identifier_from_sub_node(
                data_stream,
                &node,
                last_entry_index,
                object_identifier,
                object_transaction_identifier,
                read_node_block_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve value from entry: {}", last_entry_index)
                    );
                    Err(error)
                }
            }
        } else if last_key.object_identifier == object_identifier {
            match node.get_value_data_by_index(last_entry_index) {
                Some(value_data) => Ok(Some(value_data.to_vec())),
                None => Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve entry: {} value data",
                    last_entry_index
                ))),
            }
        } else {
            Ok(None)
        }
    }

    /// Retrieves specific value data from a sub node.
    fn get_value_data_by_identifier_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        node: &ApfsBtreeNode,
        entry_index: usize,
        object_identifier: u64,
        object_transaction_identifier: u64,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<Option<Vec<u8>>, ErrorTrace> {
        let value_data: &[u8] = match node.get_value_data_by_index(entry_index) {
            Some(value_data) => value_data,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve entry: {} value data",
                    entry_index
                )));
            }
        };
        if value_data.len() < 8 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported value data size"
            ));
        }
        let sub_node_block_number: u64 = bytes_to_u64_le!(value_data, 0);

        match self.get_value_data_by_identifier_from_node(
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
    }

    /// Initializes the object map B-tree.
    pub fn initialize(&mut self, block_size: u32, root_block_number: u64) {
        self.block_size = block_size;
        self.root_block_number = root_block_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn build_object_map_node(
        is_root: bool,
        is_leaf: bool,
        entries: &[(u64, u64, Vec<u8>)],
    ) -> Vec<u8> {
        let mut node: Vec<u8> = vec![0; 4096];
        let object_type: u32 = if is_root { 0x40000002 } else { 0x40000003 };
        node[24..28].copy_from_slice(&object_type.to_le_bytes());
        node[28..32].copy_from_slice(&0x0000000bu32.to_le_bytes());

        let mut flags: u16 = 0x0004;
        if is_root {
            flags |= 0x0001;
        }
        if is_leaf {
            flags |= 0x0002;
        }
        node[32..34].copy_from_slice(&flags.to_le_bytes());
        node[36..40].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        let entries_data_size: u16 = (entries.len() * 4) as u16;
        node[40..42].copy_from_slice(&0u16.to_le_bytes());
        node[42..44].copy_from_slice(&entries_data_size.to_le_bytes());

        let data_end_offset: usize = if is_root {
            node[4060..4064].copy_from_slice(&4096u32.to_le_bytes());
            4056
        } else {
            4096
        };
        let keys_data_offset: usize = 56 + (entries_data_size as usize);
        let mut key_relative_offset: usize = 0;
        let mut value_back_offset: usize = 0;

        for (index, (object_identifier, transaction_identifier, value_bytes)) in
            entries.iter().enumerate()
        {
            value_back_offset += value_bytes.len();
            let entry_offset: usize = 56 + (index * 4);
            node[entry_offset..entry_offset + 2]
                .copy_from_slice(&(key_relative_offset as u16).to_le_bytes());
            node[entry_offset + 2..entry_offset + 4]
                .copy_from_slice(&(value_back_offset as u16).to_le_bytes());

            let key_offset: usize = keys_data_offset + key_relative_offset;
            node[key_offset..key_offset + 8].copy_from_slice(&object_identifier.to_le_bytes());
            node[key_offset + 8..key_offset + 16]
                .copy_from_slice(&transaction_identifier.to_le_bytes());
            key_relative_offset += 16;

            let value_offset: usize = data_end_offset - value_back_offset;
            node[value_offset..value_offset + value_bytes.len()].copy_from_slice(value_bytes);
        }
        node
    }

    #[test]
    fn test_get_value_by_identifier_branch_and_leaf() -> Result<(), ErrorTrace> {
        // Block 1: root branch node pointing to leaf block 2 (for id >= 10) and leaf block 3 (for id >= 100).
        let branch_node: Vec<u8> = build_object_map_node(
            true,
            false,
            &[
                (10, 1, 2u64.to_le_bytes().to_vec()),
                (100, 1, 3u64.to_le_bytes().to_vec()),
            ],
        );
        // Block 2: leaf node with two entries for object 10 (tx 1 and tx 5) and one for object 20.
        let mut value_tx1: Vec<u8> = vec![0; 16];
        value_tx1[4..8].copy_from_slice(&4096u32.to_le_bytes());
        value_tx1[8..16].copy_from_slice(&500u64.to_le_bytes());

        let mut value_tx5: Vec<u8> = vec![0; 16];
        value_tx5[4..8].copy_from_slice(&4096u32.to_le_bytes());
        value_tx5[8..16].copy_from_slice(&600u64.to_le_bytes());

        let mut value_obj20: Vec<u8> = vec![0; 16];
        value_obj20[4..8].copy_from_slice(&4096u32.to_le_bytes());
        value_obj20[8..16].copy_from_slice(&700u64.to_le_bytes());

        let leaf_node: Vec<u8> = build_object_map_node(
            false,
            true,
            &[(10, 1, value_tx1), (10, 5, value_tx5), (20, 1, value_obj20)],
        );
        let mut image: Vec<u8> = vec![0; 4 * 4096];
        image[4096..8192].copy_from_slice(&branch_node);
        image[8192..12288].copy_from_slice(&leaf_node);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut tree: ApfsObjectMapTree = ApfsObjectMapTree::new();
        tree.initialize(4096, 1);

        // Lookup object 10 at transaction 3 should select tx 1 (physical block 500).
        let value: ApfsObjectMapValue = tree.get_value_by_identifier(&data_stream, 10, 3)?.unwrap();
        assert_eq!(value.physical_address, 500);

        // Lookup object 10 at transaction 5 (hitting node cache) should select tx 5 (600).
        let value: ApfsObjectMapValue = tree.get_value_by_identifier(&data_stream, 10, 5)?.unwrap();
        assert_eq!(value.physical_address, 600);

        // Lookup missing object 15 returns None.
        let missing: Option<ApfsObjectMapValue> =
            tree.get_value_by_identifier(&data_stream, 15, 5)?;
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_check_node_and_errors() {
        let mut image: Vec<u8> = vec![0; 8 * 4096];

        // Block 1: wrong object type for root.
        let mut wrong_type_node: Vec<u8> = build_object_map_node(true, true, &[]);
        wrong_type_node[24..28].copy_from_slice(&0x40000003u32.to_le_bytes());
        image[4096..8192].copy_from_slice(&wrong_type_node);

        // Block 2: wrong object subtype.
        let mut wrong_subtype_node: Vec<u8> = build_object_map_node(true, true, &[]);
        wrong_subtype_node[28..32].copy_from_slice(&0x0000000eu32.to_le_bytes());
        image[2 * 4096..3 * 4096].copy_from_slice(&wrong_subtype_node);

        // Block 3: missing fixed-size flag (0x0004).
        let mut wrong_flags_node: Vec<u8> = build_object_map_node(true, true, &[]);
        wrong_flags_node[32..34].copy_from_slice(&0x0003u16.to_le_bytes());
        image[3 * 4096..4 * 4096].copy_from_slice(&wrong_flags_node);

        // Block 4: unsupported footer node size.
        let mut wrong_footer_size_node: Vec<u8> = build_object_map_node(true, true, &[]);
        wrong_footer_size_node[4060..4064].copy_from_slice(&8192u32.to_le_bytes());
        image[4 * 4096..5 * 4096].copy_from_slice(&wrong_footer_size_node);

        // Block 5: missing footer (root object type without root flag 0x0001).
        let mut missing_footer_node: Vec<u8> = build_object_map_node(false, true, &[]);
        missing_footer_node[24..28].copy_from_slice(&0x40000002u32.to_le_bytes());
        image[5 * 4096..6 * 4096].copy_from_slice(&missing_footer_node);

        // Block 6: branch node referencing itself (cycle detection).
        let cycle_node: Vec<u8> =
            build_object_map_node(true, false, &[(1, 1, 6u64.to_le_bytes().to_vec())]);
        image[6 * 4096..7 * 4096].copy_from_slice(&cycle_node);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);

        for root_block in [1u64, 2, 3, 4, 5, 6, 99] {
            let mut tree: ApfsObjectMapTree = ApfsObjectMapTree::new();
            tree.initialize(4096, root_block);
            let result: Result<Option<ApfsObjectMapValue>, ErrorTrace> =
                tree.get_value_by_identifier(&data_stream, 1, 1);
            assert!(result.is_err());
        }
    }
}
