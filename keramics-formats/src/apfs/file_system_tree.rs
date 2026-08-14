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

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{
    ByteString, Utf16CharacterMappings, Utf16String, bytes_to_u16_le, bytes_to_u64_le,
};

use crate::indexed_hash_map::IndexedHashMap;
use crate::path_component::PathComponent;

use super::attribute_record::ApfsAttributeRecord;
use super::btree_node::ApfsBtreeNode;
use super::constants::*;
use super::directory_entry::ApfsDirectoryEntry;
use super::directory_record::ApfsDirectoryRecord;
use super::extent::ApfsExtent;
use super::extent_record::ApfsExtentRecord;
use super::file_system_key::ApfsFileSystemKey;
use super::file_system_key_with_extent::ApfsFileSystemKeyWithExtent;
use super::file_system_key_with_name::ApfsFileSystemKeyWithName;
use super::file_system_key_with_name_and_hash::ApfsFileSystemKeyWithNameAndHash;
use super::inode::ApfsInode;
use super::object_map_tree::ApfsObjectMapTree;
use super::object_map_value::ApfsObjectMapValue;

/// Apple File System (APFS) file system B-tree.
pub struct ApfsFileSystemTree {
    /// Block size.
    pub block_size: u32,

    /// Root (node) block number.
    pub root_block_number: u64,

    /// Case folding mappings.
    case_folding_mappings: Utf16CharacterMappings,

    /// Value to indicate if case folding should be used.
    use_case_folding: bool,
}

impl ApfsFileSystemTree {
    /// Creates a new file system B-tree.
    pub fn new(use_case_folding: bool) -> Self {
        Self {
            block_size: 0,
            root_block_number: 0,
            case_folding_mappings: Utf16CharacterMappings::from(
                APFS_UTF16_CASE_MAPPINGS.as_slice(),
            ),
            use_case_folding,
        }
    }

    /// Checks a node.
    fn check_node(&self, block_number: u64, node: &ApfsBtreeNode) -> Result<(), ErrorTrace> {
        if block_number == self.root_block_number {
            if node.object_header.object_type != 0x00000002
                && node.object_header.object_type != 0x10000002
            {
                return Err(keramics_core::error_trace_new!("Unsupported object type"));
            }
        } else {
            if node.object_header.object_type != 0x00000003
                && node.object_header.object_type != 0x10000003
            {
                return Err(keramics_core::error_trace_new!("Unsupported object type"));
            }
        }
        if node.object_header.object_subtype != 0x0000000e {
            return Err(keramics_core::error_trace_new!(
                "Unsupported object subtype"
            ));
        }
        if node.node_header.flags & 0x0004 != 0 {
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

    /// Retrieves attributes.
    pub fn get_attributes_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        object_identifier: u64,
        object_transaction_identifier: u64,
        attributes: &mut IndexedHashMap<ByteString, ApfsAttributeRecord>,
    ) -> Result<(), ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_attributes_by_identifier_from_node(
            data_stream,
            object_map_tree,
            self.root_block_number,
            object_identifier,
            object_transaction_identifier,
            attributes,
            &mut read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve attributes from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves attributes from a node.
    fn get_attributes_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        block_number: u64,
        object_identifier: u64,
        object_transaction_identifier: u64,
        attributes: &mut IndexedHashMap<ByteString, ApfsAttributeRecord>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        if node.object_header.object_type == 0x00000000 {
            return Ok(());
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
        let is_branch: bool = node.is_branch();

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
            keramics_core::debug_trace_structure!(ApfsFileSystemKey::debug_read_data(key_data));

            let mut key: ApfsFileSystemKey = ApfsFileSystemKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if !is_branch {
                if key.object_identifier == object_identifier
                    && key.data_type == APFS_FILE_SYSTEM_DATA_TYPE_EXTENDED_ATTRIBUTE
                {
                    let (name, _): (ByteString, u32) =
                        match self.read_name(&node, &key_data, entry_index) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read entry: {} name", entry_index)
                                );
                                return Err(error);
                            }
                        };
                    match self.read_attribute(&node, entry_index) {
                        Ok(attribute) => {
                            attributes.insert(name, attribute);
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read attribute entry: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                }
            } else if entry_index > 0 {
                match self.get_attributes_by_identifier_from_sub_node(
                    data_stream,
                    object_map_tree,
                    &node,
                    last_entry_index,
                    object_identifier,
                    object_transaction_identifier,
                    attributes,
                    read_node_block_numbers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve attributes from entry: {}",
                                last_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            if (key.object_identifier > object_identifier)
                || (key.object_identifier == object_identifier
                    && key.data_type > APFS_FILE_SYSTEM_DATA_TYPE_EXTENDED_ATTRIBUTE)
            {
                break;
            }
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if is_branch {
            if entry_index == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid entry index value out of bounds"
                ));
            }
            match self.get_attributes_by_identifier_from_sub_node(
                data_stream,
                object_map_tree,
                &node,
                last_entry_index,
                object_identifier,
                object_transaction_identifier,
                attributes,
                read_node_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve attributes from entry: {}",
                            last_entry_index
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves attributes from a sub node.
    fn get_attributes_by_identifier_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        node: &ApfsBtreeNode,
        entry_index: usize,
        object_identifier: u64,
        object_transaction_identifier: u64,
        attributes: &mut IndexedHashMap<ByteString, ApfsAttributeRecord>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        let sub_node_object_identifier: u64 = bytes_to_u64_le!(value_data, 0);

        let object_map_value: ApfsObjectMapValue = match object_map_tree.get_value_by_identifier(
            data_stream,
            sub_node_object_identifier,
            object_transaction_identifier,
        ) {
            Ok(Some(object_map_value)) => object_map_value,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing object map value of file system sub node object: {}",
                    sub_node_object_identifier
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve object map value of file system sub node object: {}",
                        sub_node_object_identifier
                    )
                );
                return Err(error);
            }
        };
        match self.get_attributes_by_identifier_from_node(
            data_stream,
            object_map_tree,
            object_map_value.physical_address,
            object_identifier,
            object_transaction_identifier,
            attributes,
            read_node_block_numbers,
        ) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve attributes from node: {} block: {}",
                        sub_node_object_identifier, object_map_value.physical_address
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves directory entries.
    pub fn get_directory_entries_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        parent_object_identifier: u64,
        object_transaction_identifier: u64,
        directory_entries: &mut IndexedHashMap<ByteString, ApfsDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_directory_entries_by_identifier_from_node(
            data_stream,
            object_map_tree,
            self.root_block_number,
            parent_object_identifier,
            object_transaction_identifier,
            directory_entries,
            &mut read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve directory entries from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves directory entries from a node.
    fn get_directory_entries_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        block_number: u64,
        parent_object_identifier: u64,
        object_transaction_identifier: u64,
        directory_entries: &mut IndexedHashMap<ByteString, ApfsDirectoryEntry>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        if node.object_header.object_type == 0x00000000 {
            return Ok(());
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
        let is_branch: bool = node.is_branch();

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
            keramics_core::debug_trace_structure!(ApfsFileSystemKey::debug_read_data(key_data));

            let mut key: ApfsFileSystemKey = ApfsFileSystemKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if !is_branch {
                if key.object_identifier == parent_object_identifier
                    && key.data_type == APFS_FILE_SYSTEM_DATA_TYPE_DIRECTORY_RECORD
                {
                    let (name, _): (ByteString, u32) =
                        match self.read_name(&node, &key_data, entry_index) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read entry: {} name", entry_index)
                                );
                                return Err(error);
                            }
                        };
                    match self.read_directory_entry(&node, entry_index) {
                        Ok(directory_entry) => {
                            directory_entries.insert(name, directory_entry);
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read directory entry: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                }
            } else if entry_index > 0 {
                match self.get_directory_entries_by_identifier_from_sub_node(
                    data_stream,
                    object_map_tree,
                    &node,
                    last_entry_index,
                    parent_object_identifier,
                    object_transaction_identifier,
                    directory_entries,
                    read_node_block_numbers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve directory entries from entry: {}",
                                last_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            if (key.object_identifier > parent_object_identifier)
                || (key.object_identifier == parent_object_identifier
                    && key.data_type > APFS_FILE_SYSTEM_DATA_TYPE_DIRECTORY_RECORD)
            {
                break;
            }
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if is_branch {
            if entry_index == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid entry index value out of bounds"
                ));
            }
            match self.get_directory_entries_by_identifier_from_sub_node(
                data_stream,
                object_map_tree,
                &node,
                last_entry_index,
                parent_object_identifier,
                object_transaction_identifier,
                directory_entries,
                read_node_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve directory entries from entry: {}",
                            last_entry_index
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves directory entries from a sub node.
    fn get_directory_entries_by_identifier_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        node: &ApfsBtreeNode,
        entry_index: usize,
        parent_object_identifier: u64,
        object_transaction_identifier: u64,
        directory_entries: &mut IndexedHashMap<ByteString, ApfsDirectoryEntry>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        let sub_node_object_identifier: u64 = bytes_to_u64_le!(value_data, 0);

        let object_map_value: ApfsObjectMapValue = match object_map_tree.get_value_by_identifier(
            data_stream,
            sub_node_object_identifier,
            object_transaction_identifier,
        ) {
            Ok(Some(object_map_value)) => object_map_value,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing object map value of file system sub node object: {}",
                    sub_node_object_identifier
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve object map value of file system sub node object: {}",
                        sub_node_object_identifier
                    )
                );
                return Err(error);
            }
        };
        match self.get_directory_entries_by_identifier_from_node(
            data_stream,
            object_map_tree,
            object_map_value.physical_address,
            parent_object_identifier,
            object_transaction_identifier,
            directory_entries,
            read_node_block_numbers,
        ) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve directory entries from node: {} block: {}",
                        sub_node_object_identifier, object_map_value.physical_address
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_directory_entry_by_name(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        parent_object_identifier: u64,
        name: &PathComponent,
        object_transaction_identifier: u64,
    ) -> Result<Option<ApfsDirectoryEntry>, ErrorTrace> {
        // TODO: convert name to Unicode NFD.
        let name_string: Utf16String = if self.use_case_folding {
            match name.to_utf16_string_with_case_folding(&self.case_folding_mappings) {
                Ok(utf16_string) => utf16_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UTF-16 string with case folding"
                    );
                    return Err(error);
                }
            }
        } else {
            match name.to_utf16_string() {
                Ok(utf16_string) => utf16_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UTF-16 string"
                    );
                    return Err(error);
                }
            }
        };
        // TODO: calculate name_hash

        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_directory_entry_by_name_from_node(
            data_stream,
            object_map_tree,
            self.root_block_number,
            parent_object_identifier,
            &name_string,
            object_transaction_identifier,
            &mut read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve directory entry from root node: {}",
                        self.root_block_number
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves specific directory entry from a node.
    fn get_directory_entry_by_name_from_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        block_number: u64,
        parent_object_identifier: u64,
        name: &Utf16String,
        object_transaction_identifier: u64,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<Option<ApfsDirectoryEntry>, ErrorTrace> {
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
        if node.object_header.object_type == 0x00000000 {
            return Ok(None);
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
        let is_branch: bool = node.is_branch();

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
            keramics_core::debug_trace_structure!(ApfsFileSystemKey::debug_read_data(key_data));

            let mut key: ApfsFileSystemKey = ApfsFileSystemKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if key.object_identifier == parent_object_identifier
                && key.data_type == APFS_FILE_SYSTEM_DATA_TYPE_DIRECTORY_RECORD
            {
                let (key_name, _): (ByteString, u32) =
                    match self.read_name(&node, &key_data, entry_index) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read entry: {} name", entry_index)
                            );
                            return Err(error);
                        }
                    };
                let utf16_string: Utf16String = if self.use_case_folding {
                    match Utf16String::from_byte_string_with_case_folding(
                        &key_name,
                        &self.case_folding_mappings,
                    ) {
                        Ok(utf16_string) => utf16_string,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable determine UTF-16 string with case folding from key: {} name",
                                    entry_index
                                )
                            );
                            return Err(error);
                        }
                    }
                } else {
                    match Utf16String::from_byte_string(&key_name) {
                        Ok(utf16_string) => utf16_string,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable determine UTF-16 string from key: {} name",
                                    entry_index
                                )
                            );
                            return Err(error);
                        }
                    }
                };
                let result: Ordering = utf16_string.cmp(name);

                // Note that the order of the keys is not alphabetical given the name size and hash.
                if result == Ordering::Equal {
                    if is_branch {
                        last_entry_index = entry_index;

                        entry_index += 1;

                        break;
                    } else {
                        match self.read_directory_entry(&node, entry_index) {
                            Ok(mut directory_entry) => {
                                directory_entry.name = Some(key_name);

                                return Ok(Some(directory_entry));
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read directory entry: {}", entry_index)
                                );
                                return Err(error);
                            }
                        }
                    }
                }
            }
            if (key.object_identifier > parent_object_identifier)
                || (key.object_identifier == parent_object_identifier
                    && key.data_type > APFS_FILE_SYSTEM_DATA_TYPE_DIRECTORY_RECORD)
            {
                break;
            }
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if is_branch {
            if entry_index == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid entry index value out of bounds"
                ));
            }
            match self.get_directory_entry_by_name_from_sub_node(
                data_stream,
                object_map_tree,
                &node,
                last_entry_index,
                parent_object_identifier,
                name,
                object_transaction_identifier,
                read_node_block_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve directory entry from entry: {}",
                            last_entry_index,
                        )
                    );
                    Err(error)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Retrieves specific directory entry from a sub node.
    fn get_directory_entry_by_name_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        node: &ApfsBtreeNode,
        entry_index: usize,
        parent_object_identifier: u64,
        name: &Utf16String,
        object_transaction_identifier: u64,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<Option<ApfsDirectoryEntry>, ErrorTrace> {
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
        let sub_node_object_identifier: u64 = bytes_to_u64_le!(value_data, 0);

        let object_map_value: ApfsObjectMapValue = match object_map_tree.get_value_by_identifier(
            data_stream,
            sub_node_object_identifier,
            object_transaction_identifier,
        ) {
            Ok(Some(object_map_value)) => object_map_value,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing object map value of file system sub node object: {}",
                    sub_node_object_identifier
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve object map value of file system sub node object: {}",
                        sub_node_object_identifier
                    )
                );
                return Err(error);
            }
        };
        match self.get_directory_entry_by_name_from_node(
            data_stream,
            object_map_tree,
            object_map_value.physical_address,
            parent_object_identifier,
            name,
            object_transaction_identifier,
            read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve directory entry from node: {} block: {}",
                        sub_node_object_identifier, object_map_value.physical_address
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves extents.
    pub fn get_extents_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        object_identifier: u64,
        object_transaction_identifier: u64,
        extents: &mut Vec<ApfsExtent>,
    ) -> Result<(), ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_extents_by_identifier_from_node(
            data_stream,
            object_map_tree,
            self.root_block_number,
            object_identifier,
            object_transaction_identifier,
            extents,
            &mut read_node_block_numbers,
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
    fn get_extents_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        block_number: u64,
        object_identifier: u64,
        object_transaction_identifier: u64,
        extents: &mut Vec<ApfsExtent>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        if node.object_header.object_type == 0x00000000 {
            return Ok(());
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
        let is_branch: bool = node.is_branch();

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
            keramics_core::debug_trace_structure!(ApfsFileSystemKey::debug_read_data(key_data));

            let mut key: ApfsFileSystemKey = ApfsFileSystemKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if !is_branch {
                if key.object_identifier == object_identifier
                    && key.data_type == APFS_FILE_SYSTEM_DATA_TYPE_FILE_EXTENT
                {
                    match self.read_extent(&node, &key_data, entry_index) {
                        Ok(extent) => {
                            extents.push(extent);
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read extent entry: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                }
            } else if entry_index > 0 {
                match self.get_extents_by_identifier_from_sub_node(
                    data_stream,
                    object_map_tree,
                    &node,
                    last_entry_index,
                    object_identifier,
                    object_transaction_identifier,
                    extents,
                    read_node_block_numbers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve extents from entry: {}",
                                last_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            if (key.object_identifier > object_identifier)
                || (key.object_identifier == object_identifier
                    && key.data_type > APFS_FILE_SYSTEM_DATA_TYPE_FILE_EXTENT)
            {
                break;
            }
            last_entry_index = entry_index;

            entry_index += 1;
        }
        if is_branch {
            if entry_index == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid entry index value out of bounds"
                ));
            }
            match self.get_extents_by_identifier_from_sub_node(
                data_stream,
                object_map_tree,
                &node,
                last_entry_index,
                object_identifier,
                object_transaction_identifier,
                extents,
                read_node_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve extents from entry: {}",
                            last_entry_index
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves extents from a sub node.
    fn get_extents_by_identifier_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        node: &ApfsBtreeNode,
        entry_index: usize,
        object_identifier: u64,
        object_transaction_identifier: u64,
        extents: &mut Vec<ApfsExtent>,
        read_node_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
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
        let sub_node_object_identifier: u64 = bytes_to_u64_le!(value_data, 0);

        let object_map_value: ApfsObjectMapValue = match object_map_tree.get_value_by_identifier(
            data_stream,
            sub_node_object_identifier,
            object_transaction_identifier,
        ) {
            Ok(Some(object_map_value)) => object_map_value,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing object map value of file system sub node object: {}",
                    sub_node_object_identifier
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve object map value of file system sub node object: {}",
                        sub_node_object_identifier
                    )
                );
                return Err(error);
            }
        };
        match self.get_extents_by_identifier_from_node(
            data_stream,
            object_map_tree,
            object_map_value.physical_address,
            object_identifier,
            object_transaction_identifier,
            extents,
            read_node_block_numbers,
        ) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve extents from node: {} block: {}",
                        sub_node_object_identifier, object_map_value.physical_address
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific inode.
    pub fn get_inode_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        object_identifier: u64,
        object_transaction_identifier: u64,
    ) -> Result<Option<ApfsInode>, ErrorTrace> {
        let mut read_node_block_numbers: HashSet<u64> = HashSet::new();

        match self.get_value_data_by_identifier_from_node(
            data_stream,
            object_map_tree,
            self.root_block_number,
            APFS_FILE_SYSTEM_DATA_TYPE_INODE,
            object_identifier,
            object_transaction_identifier,
            &mut read_node_block_numbers,
        ) {
            Ok(Some(value_data)) => {
                keramics_core::debug_trace_structure!(ApfsInode::debug_read_data(&value_data));

                let mut inode: ApfsInode = ApfsInode::new();

                match inode.read_data(&value_data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read inode");
                        return Err(error);
                    }
                }
                Ok(Some(inode))
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

    /// Retrieves a specific node.
    fn get_node_by_number(
        &self,
        data_stream: &DataStreamReference,
        block_number: u64,
    ) -> Result<ApfsBtreeNode, ErrorTrace> {
        let node_offset: u64 = block_number * (self.block_size as u64);

        let mut node: ApfsBtreeNode = ApfsBtreeNode::new();

        match node.read_at_position(&data_stream, SeekFrom::Start(node_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read file system B-tree root node at offset: {} (0x{:08x}))",
                        node_offset, node_offset
                    )
                );
                return Err(error);
            }
        }
        Ok(node)
    }

    /// Retrieves specific value data from a node.
    fn get_value_data_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        block_number: u64,
        data_type: u8,
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
        if node.object_header.object_type == 0x00000000 {
            return Ok(None);
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
        let mut last_key: ApfsFileSystemKey = ApfsFileSystemKey::new();
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
            keramics_core::debug_trace_structure!(ApfsFileSystemKey::debug_read_data(key_data));

            let mut key: ApfsFileSystemKey = ApfsFileSystemKey::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read key");
                    return Err(error);
                }
            }
            if (key.object_identifier > object_identifier)
                || (key.object_identifier == object_identifier && key.data_type > data_type)
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
                object_map_tree,
                &node,
                last_entry_index,
                data_type,
                object_identifier,
                object_transaction_identifier,
                read_node_block_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve value data of type: {} from entry: {}",
                            data_type, last_entry_index,
                        )
                    );
                    Err(error)
                }
            }
        } else if last_key.object_identifier == object_identifier && last_key.data_type == data_type
        {
            match node.get_value_data_by_index(last_entry_index) {
                Some(value_data) => Ok(Some(value_data.to_vec())),
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve entry: {} value data of type: {}",
                        last_entry_index, data_type
                    )));
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Retrieves specific value data from a sub node.
    fn get_value_data_by_identifier_from_sub_node(
        &self,
        data_stream: &DataStreamReference,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        node: &ApfsBtreeNode,
        entry_index: usize,
        data_type: u8,
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
        let sub_node_object_identifier: u64 = bytes_to_u64_le!(value_data, 0);

        let object_map_value: ApfsObjectMapValue = match object_map_tree.get_value_by_identifier(
            data_stream,
            sub_node_object_identifier,
            object_transaction_identifier,
        ) {
            Ok(Some(object_map_value)) => object_map_value,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing object map value of file system sub node object: {}",
                    sub_node_object_identifier
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve object map value of file system sub node object: {}",
                        sub_node_object_identifier
                    )
                );
                return Err(error);
            }
        };
        match self.get_value_data_by_identifier_from_node(
            data_stream,
            object_map_tree,
            object_map_value.physical_address,
            data_type,
            object_identifier,
            object_transaction_identifier,
            read_node_block_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve value data of type: {} from node: {} block: {}",
                        data_type, sub_node_object_identifier, object_map_value.physical_address
                    )
                );
                Err(error)
            }
        }
    }

    /// Initializes the file system B-tree.
    pub fn initialize(&mut self, block_size: u32, root_block_number: u64) {
        self.block_size = block_size;
        self.root_block_number = root_block_number;
    }

    /// Reads an attribute.
    fn read_attribute(
        &self,
        node: &ApfsBtreeNode,
        entry_index: usize,
    ) -> Result<ApfsAttributeRecord, ErrorTrace> {
        let value_data: &[u8] = match node.get_value_data_by_index(entry_index) {
            Some(value_data) => value_data,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve entry: {} value data",
                    entry_index
                )));
            }
        };
        keramics_core::debug_trace_structure!(ApfsAttributeRecord::debug_read_data(&value_data));

        let mut attribute_record: ApfsAttributeRecord = ApfsAttributeRecord::new();

        match attribute_record.read_data(&value_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attribute record");
                return Err(error);
            }
        }
        Ok(attribute_record)
    }

    /// Reads a directory entry.
    fn read_directory_entry(
        &self,
        node: &ApfsBtreeNode,
        entry_index: usize,
    ) -> Result<ApfsDirectoryEntry, ErrorTrace> {
        let value_data: &[u8] = match node.get_value_data_by_index(entry_index) {
            Some(value_data) => value_data,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve entry: {} value data",
                    entry_index
                )));
            }
        };
        keramics_core::debug_trace_structure!(ApfsDirectoryRecord::debug_read_data(&value_data));

        let mut directory_record: ApfsDirectoryRecord = ApfsDirectoryRecord::new();

        match directory_record.read_data(&value_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read directory record");
                return Err(error);
            }
        }
        Ok(ApfsDirectoryEntry::new(directory_record))
    }

    /// Reads an extent.
    fn read_extent(
        &self,
        node: &ApfsBtreeNode,
        key_data: &[u8],
        entry_index: usize,
    ) -> Result<ApfsExtent, ErrorTrace> {
        let key_data_size: usize = key_data.len();

        if key_data_size < 10 {
            return Err(keramics_core::error_trace_new!("Unsupported key data size"));
        }
        keramics_core::debug_trace_structure!(ApfsFileSystemKeyWithExtent::debug_read_data(
            key_data
        ));
        let mut key: ApfsFileSystemKeyWithExtent = ApfsFileSystemKeyWithExtent::new();

        match key.read_data(&key_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read extent key: {}", entry_index)
                );
                return Err(error);
            }
        }
        let value_data: &[u8] = match node.get_value_data_by_index(entry_index) {
            Some(value_data) => value_data,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve entry: {} value data",
                    entry_index
                )));
            }
        };
        keramics_core::debug_trace_structure!(ApfsDirectoryRecord::debug_read_data(&value_data));

        let mut extent_record: ApfsExtentRecord = ApfsExtentRecord::new();

        match extent_record.read_data(&value_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extent record");
                return Err(error);
            }
        }
        Ok(ApfsExtent::new(
            key.extent_offset,
            extent_record.extent_size,
            extent_record.physical_block_number,
            extent_record.encryption_identifier,
        ))
    }

    /// Reads a name.
    fn read_name(
        &self,
        node: &ApfsBtreeNode,
        key_data: &[u8],
        entry_index: usize,
    ) -> Result<(ByteString, u32), ErrorTrace> {
        let key_data_size: usize = key_data.len();

        if key_data_size < 10 {
            return Err(keramics_core::error_trace_new!("Unsupported key data size"));
        }
        let mut name_hash: u32 = 0;

        let name_offset: usize;
        let name_end_offset: usize;

        // Use the name size to determine if the key data contains a name with or without hash.
        let name_size: usize = (bytes_to_u16_le!(key_data, 8) & 0x03ff) as usize;

        if name_size < key_data_size - 10 {
            keramics_core::debug_trace_structure!(
                ApfsFileSystemKeyWithNameAndHash::debug_read_data(key_data)
            );
            let mut key: ApfsFileSystemKeyWithNameAndHash = ApfsFileSystemKeyWithNameAndHash::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read directory key: {}", entry_index)
                    );
                    return Err(error);
                }
            }
            name_offset = 12;
            name_end_offset = name_offset + (key.name_size as usize);
            name_hash = key.name_hash;
        } else {
            keramics_core::debug_trace_structure!(ApfsFileSystemKeyWithName::debug_read_data(
                key_data
            ));

            let mut key: ApfsFileSystemKeyWithName = ApfsFileSystemKeyWithName::new();

            match key.read_data(&key_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read directory key: {}", entry_index)
                    );
                    return Err(error);
                }
            }
            name_offset = 10;
            name_end_offset = name_offset + (key.name_size as usize);
            // TODO: calculate hash
        }
        if name_end_offset > key_data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid directory key - name size value out of bounds"
            ));
        }
        let mut name: ByteString = ByteString::new();
        name.read_data(&key_data[name_offset..name_end_offset]);

        Ok((name, name_hash))
    }
}
