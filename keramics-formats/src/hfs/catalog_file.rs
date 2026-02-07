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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::{Utf16CharacterMappings, bytes_to_u16_be, bytes_to_u32_be};

use crate::path_component::PathComponent;
use crate::util::calculate_alignment_padding;

use super::block_range::HfsBlockRange;
use super::btree_file::HfsBtreeFile;
use super::btree_node::HfsBtreeNode;
use super::catalog_file_entry_record::HfsCatalogFileEntryRecord;
use super::catalog_file_record::HfsCatalogFileRecord;
use super::catalog_folder_record::HfsCatalogFolderRecord;
use super::catalog_key::HfsCatalogKey;
use super::catalog_thread_record::HfsCatalogThreadRecord;
use super::constants::*;
use super::directory_entries::HfsDirectoryEntries;
use super::directory_entry::HfsDirectoryEntry;
use super::enums::{HfsBtreeNodeType, HfsFormat, HfsKeyComparisonMethod};
use super::string::HfsString;

/// Hierarchical File System (HFS) catalog file.
pub struct HfsCatalogFile {
    /// B-tree file.
    btree_file: HfsBtreeFile,

    /// Character encoding.
    encoding: CharacterEncoding,

    /// Case folding mappings.
    case_folding_mappings: Utf16CharacterMappings,
}

impl HfsCatalogFile {
    /// Creates a new catalog file.
    pub fn new() -> Self {
        Self {
            btree_file: HfsBtreeFile::new(),
            encoding: CharacterEncoding::MacRoman,
            case_folding_mappings: Utf16CharacterMappings::from(HFS_UTF16_CASE_MAPPINGS.as_slice()),
        }
    }

    /// Retrieves directory entries.
    pub fn get_directory_entries_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        parent_identifier: u32,
        directory_entries: &mut HfsDirectoryEntries,
    ) -> Result<(), ErrorTrace> {
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_directory_entries_by_identifier_from_node(
            data_stream,
            self.btree_file.root_node_number,
            parent_identifier,
            directory_entries,
            &mut read_node_numbers,
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
        node_number: u32,
        parent_identifier: u32,
        directory_entries: &mut HfsDirectoryEntries,
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
        let mut last_key: HfsCatalogKey = HfsCatalogKey::new();
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
                format!("HfsCatalogKey of record: {}", record_index),
                node.get_record_offset_by_index(record_index),
                record_data,
                record_data.len(),
                HfsCatalogKey::debug_read_data(&self.btree_file.format, record_data)
            );
            let mut key: HfsCatalogKey = HfsCatalogKey::new();

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
                if key.parent_identifier == parent_identifier {
                    let mut data_offset: usize = key.size as usize;

                    if self.btree_file.format == HfsFormat::Hfs {
                        let alignment_padding: usize = calculate_alignment_padding(data_offset, 2);

                        if alignment_padding > 0 {
                            // TODO: debug print alignment padding.
                            data_offset += alignment_padding;
                        }
                    }
                    if data_offset + 2 > record_data.len() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid data size of record: {} value out of bounds",
                            record_index
                        )));
                    }
                    let record_type: u16 = bytes_to_u16_be!(record_data, data_offset);

                    let process_record: bool = match record_type {
                        0x0001 | 0x0002 => self.btree_file.format != HfsFormat::Hfs,
                        0x0100 | 0x0200 => self.btree_file.format == HfsFormat::Hfs,
                        _ => false,
                    };
                    if process_record {
                        let name: HfsString = match key.read_name(
                            &self.btree_file.format,
                            &self.encoding,
                            record_data,
                        ) {
                            Ok(name) => name,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read name of key: {}", record_index)
                                );
                                return Err(error);
                            }
                        };
                        match self.read_directory_entry(&key, record_data) {
                            Ok(directory_entry) => {
                                directory_entries.insert_entry(name, directory_entry);
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read directory entry: {}", record_index)
                                );
                                return Err(error);
                            }
                        }
                    }
                }
            } else if record_index > 0 {
                if key.parent_identifier >= parent_identifier {
                    let data_offset: usize = last_key.size as usize;

                    if data_offset + 4 > last_record_data.len() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid data size of record: {} value out of bounds",
                            record_index
                        )));
                    }
                    keramics_core::debug_trace_data!(
                        format!("HfsCatalogBranchNodeValue: {}", record_index),
                        node.get_record_offset_by_index(record_index - 1),
                        &last_record_data[data_offset..data_offset + 4],
                        4
                    );
                    let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

                    match self.get_directory_entries_by_identifier_from_node(
                        data_stream,
                        sub_node_number,
                        parent_identifier,
                        directory_entries,
                        read_node_numbers,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve directory entries from node: {}",
                                    sub_node_number
                                )
                            );
                            return Err(error);
                        }
                    }
                }
                if key.parent_identifier > parent_identifier {
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
                format!("HfsCatalogBranchNodeValue: {}", record_index),
                node.get_record_offset_by_index(record_index - 1),
                &last_record_data[data_offset..data_offset + 4],
                4
            );
            let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

            match self.get_directory_entries_by_identifier_from_node(
                data_stream,
                sub_node_number,
                parent_identifier,
                directory_entries,
                read_node_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve directory entries from node: {}",
                            sub_node_number
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves a specific directory entry.
    pub fn get_directory_entry_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        identifier: u32,
    ) -> Result<Option<HfsDirectoryEntry>, ErrorTrace> {
        let thread_record: HfsCatalogThreadRecord =
            match self.get_thread_record_by_identifier(data_stream, identifier) {
                Ok(Some(thread_record)) => thread_record,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve thread record: {}", identifier)
                    );
                    return Err(error);
                }
            };
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_directory_entry_by_thread_record_from_node(
            data_stream,
            self.btree_file.root_node_number,
            &thread_record,
            &mut read_node_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve directory entry from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_directory_entry_by_name(
        &self,
        data_stream: &DataStreamReference,
        parent_identifier: u32,
        name: &PathComponent,
    ) -> Result<Option<HfsDirectoryEntry>, ErrorTrace> {
        if self.btree_file.root_node_number == 0 {
            return Ok(None);
        }
        let name_string: HfsString = match &self.btree_file.format {
            HfsFormat::Hfs => match name.to_byte_string(&self.encoding) {
                Ok(byte_string) => HfsString::ByteString(byte_string),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to byte string"
                    );
                    return Err(error);
                }
            },
            HfsFormat::HfsPlus => {
                match name.to_utf16_string_with_case_folding(&self.case_folding_mappings) {
                    Ok(utf16_string) => HfsString::Utf16String(utf16_string),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to convert path component to UTF-16 string with case folding"
                        );
                        return Err(error);
                    }
                }
            }
            HfsFormat::HfsX => match name.to_utf16_string() {
                Ok(utf16_string) => HfsString::Utf16String(utf16_string),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UTF-16 string"
                    );
                    return Err(error);
                }
            },
        };
        let mut thread_record: HfsCatalogThreadRecord = HfsCatalogThreadRecord::new();
        thread_record.parent_identifier = parent_identifier;

        // TODO: convert name to Unicode NFD.
        if self.btree_file.key_comparion_method == HfsKeyComparisonMethod::Binary {
            thread_record.name = name_string;
        } else {
            thread_record.name =
                match name_string.new_with_case_folding(&self.case_folding_mappings) {
                    Ok(hfs_string) => hfs_string,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable apply case folding to name"
                        );
                        return Err(error);
                    }
                }
        }
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_directory_entry_by_thread_record_from_node(
            data_stream,
            self.btree_file.root_node_number,
            &thread_record,
            &mut read_node_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve directory entry from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific directory entry from a node.
    fn get_directory_entry_by_thread_record_from_node(
        &self,
        data_stream: &DataStreamReference,
        node_number: u32,
        thread_record: &HfsCatalogThreadRecord,
        read_node_numbers: &mut HashSet<u32>,
    ) -> Result<Option<HfsDirectoryEntry>, ErrorTrace> {
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
        let mut last_key: HfsCatalogKey = HfsCatalogKey::new();
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
                format!("HfsCatalogKey of record: {}", record_index),
                node.get_record_offset_by_index(record_index),
                record_data,
                record_data.len(),
                HfsCatalogKey::debug_read_data(&self.btree_file.format, record_data)
            );
            let mut key: HfsCatalogKey = HfsCatalogKey::new();

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
            if key.parent_identifier > thread_record.parent_identifier {
                break;
            }
            if key.parent_identifier == thread_record.parent_identifier {
                let process_record: bool = if is_branch {
                    true
                } else if key.size == 0 {
                    false
                } else {
                    let mut data_offset: usize = key.size as usize;

                    if self.btree_file.format == HfsFormat::Hfs {
                        let alignment_padding: usize = calculate_alignment_padding(data_offset, 2);

                        if alignment_padding > 0 {
                            // TODO: debug print alignment padding.
                            data_offset += alignment_padding;
                        }
                    }
                    if data_offset + 2 > record_data.len() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid data size of record: {} value out of bounds",
                            record_index
                        )));
                    }
                    let record_type: u16 = bytes_to_u16_be!(record_data, data_offset);

                    match record_type {
                        0x0001 | 0x0002 => self.btree_file.format != HfsFormat::Hfs,
                        0x0100 | 0x0200 => self.btree_file.format == HfsFormat::Hfs,
                        _ => false,
                    }
                };
                if process_record {
                    let name: HfsString =
                        match key.read_name(&self.btree_file.format, &self.encoding, record_data) {
                            Ok(name) => name,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read name of key: {}", record_index)
                                );
                                return Err(error);
                            }
                        };
                    let result: Ordering =
                        if self.btree_file.key_comparion_method == HfsKeyComparisonMethod::Binary {
                            name.cmp(&thread_record.name)
                        } else {
                            let case_folded_name: HfsString =
                                match name.new_with_case_folding(&self.case_folding_mappings) {
                                    Ok(hfs_string) => hfs_string,
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable apply case folding to name of key: {}",
                                                record_index
                                            )
                                        );
                                        return Err(error);
                                    }
                                };
                            case_folded_name.cmp(&thread_record.name)
                        };
                    if result == Ordering::Greater {
                        break;
                    }
                    if result == Ordering::Equal {
                        if is_branch {
                            record_index += 1;

                            last_key = key;
                            last_record_data = record_data;

                            break;
                        } else {
                            match self.read_directory_entry(&key, record_data) {
                                Ok(mut directory_entry) => {
                                    if !name.is_empty() {
                                        directory_entry.name = Some(name);
                                    }
                                    return Ok(Some(directory_entry));
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!("Unable to read directory entry: {}", record_index)
                                    );
                                    return Err(error);
                                }
                            }
                        }
                    }
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
                format!("HfsCatalogBranchNodeValue: {}", record_index),
                node.get_record_offset_by_index(record_index - 1),
                &last_record_data[data_offset..data_offset + 4],
                4
            );
            let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

            match self.get_directory_entry_by_thread_record_from_node(
                data_stream,
                sub_node_number,
                thread_record,
                read_node_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve directory entry from node: {}",
                            sub_node_number
                        )
                    );
                    Err(error)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Retrieves a specific catalog thread record.
    fn get_thread_record_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        identifier: u32,
    ) -> Result<Option<HfsCatalogThreadRecord>, ErrorTrace> {
        if self.btree_file.root_node_number == 0 {
            return Ok(None);
        }
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_thread_record_by_identifier_from_node(
            data_stream,
            self.btree_file.root_node_number,
            identifier,
            &mut read_node_numbers,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve thread record from root node"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific catalog thread record from a node.
    fn get_thread_record_by_identifier_from_node(
        &self,
        data_stream: &DataStreamReference,
        node_number: u32,
        identifier: u32,
        read_node_numbers: &mut HashSet<u32>,
    ) -> Result<Option<HfsCatalogThreadRecord>, ErrorTrace> {
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
        let mut last_key: HfsCatalogKey = HfsCatalogKey::new();
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
                format!("HfsCatalogKey of record: {}", record_index),
                node.get_record_offset_by_index(record_index),
                record_data,
                record_data.len(),
                HfsCatalogKey::debug_read_data(&self.btree_file.format, record_data)
            );
            let mut key: HfsCatalogKey = HfsCatalogKey::new();

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
            if key.size > 0 {
                if key.parent_identifier > identifier {
                    break;
                }
                if key.parent_identifier == identifier {
                    if is_branch {
                        break;
                    }
                    if key.name_size == 0 {
                        match self.read_thread_record(&key, record_data) {
                            Ok(thread_record) => return Ok(Some(thread_record)),
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read thread record: {}", record_index)
                                );
                                return Err(error);
                            }
                        }
                    }
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
                format!("HfsCatalogBranchNodeValue: {}", record_index),
                node.get_record_offset_by_index(record_index - 1),
                &last_record_data[data_offset..data_offset + 4],
                4
            );
            let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

            match self.get_thread_record_by_identifier_from_node(
                data_stream,
                sub_node_number,
                identifier,
                read_node_numbers,
            ) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve thread record from node: {}",
                            sub_node_number
                        )
                    );
                    Err(error)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Initializes the catalog file.
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

    /// Reads a directory entry.
    fn read_directory_entry(
        &self,
        key: &HfsCatalogKey,
        record_data: &[u8],
    ) -> Result<HfsDirectoryEntry, ErrorTrace> {
        let mut data_offset: usize = key.size as usize;

        if self.btree_file.format == HfsFormat::Hfs {
            let alignment_padding: usize = calculate_alignment_padding(data_offset, 2);

            if alignment_padding > 0 {
                // TODO: debug print alignment padding.
                data_offset += alignment_padding;
            }
        }
        if data_offset + 2 > record_data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid record data size value out of bounds"
            )));
        }
        let record_type: u16 = bytes_to_u16_be!(record_data, data_offset);

        match record_type {
            0x0001 | 0x0100 => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsCatalogFolderRecord",
                    0,
                    &record_data[data_offset..],
                    record_data.len() - data_offset,
                    HfsCatalogFolderRecord::debug_read_data(
                        &self.btree_file.format,
                        &record_data[data_offset..]
                    )
                );
                let mut folder_record: HfsCatalogFolderRecord = HfsCatalogFolderRecord::new();

                match folder_record.read_data(&self.btree_file.format, &record_data[data_offset..])
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read folder record"
                        );
                        return Err(error);
                    }
                }
                Ok(HfsDirectoryEntry::new(HfsCatalogFileEntryRecord::Folder(
                    folder_record,
                )))
            }
            0x0002 | 0x0200 => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsCatalogFileRecord",
                    0,
                    &record_data[data_offset..],
                    record_data.len() - data_offset,
                    HfsCatalogFileRecord::debug_read_data(
                        &self.btree_file.format,
                        &record_data[data_offset..]
                    )
                );
                let mut file_record: HfsCatalogFileRecord = HfsCatalogFileRecord::new();

                match file_record.read_data(&self.btree_file.format, &record_data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read file record");
                        return Err(error);
                    }
                }
                Ok(HfsDirectoryEntry::new(HfsCatalogFileEntryRecord::File(
                    file_record,
                )))
            }
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                record_type
            ))),
        }
    }

    /// Reads a thread record.
    fn read_thread_record(
        &self,
        key: &HfsCatalogKey,
        record_data: &[u8],
    ) -> Result<HfsCatalogThreadRecord, ErrorTrace> {
        let mut data_offset: usize = key.size as usize;

        if self.btree_file.format == HfsFormat::Hfs {
            let alignment_padding: usize = calculate_alignment_padding(data_offset, 2);

            if alignment_padding > 0 {
                // TODO: debug print alignment padding.
                data_offset += alignment_padding;
            }
        }
        keramics_core::debug_trace_data_and_structure!(
            "HfsCatalogThreadRecord",
            0,
            &record_data[data_offset..],
            record_data.len() - data_offset,
            HfsCatalogThreadRecord::debug_read_data(
                &self.btree_file.format,
                &record_data[data_offset..]
            )
        );
        let mut thread_record: HfsCatalogThreadRecord = HfsCatalogThreadRecord::new();

        match thread_record.read_data(&self.btree_file.format, &record_data[data_offset..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read thread record");
                return Err(error);
            }
        }
        let name: HfsString = match thread_record.read_name(
            &self.btree_file.format,
            &self.encoding,
            &record_data[data_offset..],
        ) {
            Ok(name) => name,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read name of thread record"
                );
                return Err(error);
            }
        };
        if self.btree_file.key_comparion_method == HfsKeyComparisonMethod::Binary {
            thread_record.name = name;
        } else {
            thread_record.name = match name.new_with_case_folding(&self.case_folding_mappings) {
                Ok(hfs_string) => hfs_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable apply case folding to name of thread record"
                    );
                    return Err(error);
                }
            }
        }
        Ok(thread_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    // Tests with HFS.

    fn get_data_stream_hfs() -> Result<DataStreamReference, ErrorTrace> {
        let path_string: String = get_test_data_path("hfs/hfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        Ok(data_stream)
    }

    fn get_catalog_file_hfs(
        data_stream: &DataStreamReference,
    ) -> Result<HfsCatalogFile, ErrorTrace> {
        let mut catalog_file: HfsCatalogFile = HfsCatalogFile::new();

        catalog_file.initialize(
            &HfsFormat::Hfs,
            512,
            32256,
            vec![HfsBlockRange::new(0, 5 + 63, 63)],
            data_stream,
        )?;
        Ok(catalog_file)
    }

    // TODO: add tests for get_directory_entries_by_identifier
    // TODO: add tests for get_directory_entries_by_identifier_from_node

    #[test]
    fn test_get_directory_entry_by_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfs()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfs(&data_stream)?;

        let result: Option<HfsDirectoryEntry> =
            test_struct.get_directory_entry_by_identifier(&data_stream, 2)?;
        assert!(result.is_some());

        let result: Option<HfsDirectoryEntry> =
            test_struct.get_directory_entry_by_identifier(&data_stream, 999)?;
        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests for get_directory_entry_by_name

    #[test]
    fn test_get_directory_entry_by_thread_record_from_node_with_hfs() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfs()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfs(&data_stream)?;

        let mut thread_record: HfsCatalogThreadRecord = test_struct
            .get_thread_record_by_identifier(&data_stream, 2)?
            .unwrap();

        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        let result: Option<HfsDirectoryEntry> = test_struct
            .get_directory_entry_by_thread_record_from_node(
                &data_stream,
                test_struct.btree_file.root_node_number,
                &thread_record,
                &mut read_node_numbers,
            )?;
        assert!(result.is_some());

        thread_record.parent_identifier = 999;

        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        let result: Option<HfsDirectoryEntry> = test_struct
            .get_directory_entry_by_thread_record_from_node(
                &data_stream,
                test_struct.btree_file.root_node_number,
                &thread_record,
                &mut read_node_numbers,
            )?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_thread_record_by_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfs()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfs(&data_stream)?;

        let result: Option<HfsCatalogThreadRecord> =
            test_struct.get_thread_record_by_identifier(&data_stream, 2)?;
        assert!(result.is_some());

        let result: Option<HfsCatalogThreadRecord> =
            test_struct.get_thread_record_by_identifier(&data_stream, 999)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_initialize_with_hfs() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfs()?;

        let mut test_struct: HfsCatalogFile = HfsCatalogFile::new();
        test_struct.initialize(
            &HfsFormat::Hfs,
            512,
            32256,
            vec![HfsBlockRange::new(0, 5 + 63, 63)],
            &data_stream,
        )
    }

    // TODO: add tests for read_directory_entry
    // TODO: add tests for read_thread_record

    // Tests with HFS+.

    fn get_data_stream_hfsplus() -> Result<DataStreamReference, ErrorTrace> {
        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        Ok(data_stream)
    }

    fn get_catalog_file_hfsplus(
        data_stream: &DataStreamReference,
    ) -> Result<HfsCatalogFile, ErrorTrace> {
        let mut catalog_file: HfsCatalogFile = HfsCatalogFile::new();

        catalog_file.initialize(
            &HfsFormat::HfsPlus,
            4096,
            81920,
            vec![HfsBlockRange::new(0, 242, 20)],
            data_stream,
        )?;
        Ok(catalog_file)
    }

    #[test]
    fn test_get_directory_entries_by_identifier() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfsplus()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfsplus(&data_stream)?;

        let mut directory_entries: HfsDirectoryEntries = HfsDirectoryEntries::new();
        test_struct.get_directory_entries_by_identifier(
            &data_stream,
            29,
            &mut directory_entries,
        )?;
        assert_eq!(directory_entries.get_number_of_entries(), 5);

        Ok(())
    }

    // TODO: add tests for get_directory_entries_by_identifier_from_node

    #[test]
    fn test_get_directory_entry_by_identifier_with_hfsplus() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfsplus()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfsplus(&data_stream)?;

        let result: Option<HfsDirectoryEntry> =
            test_struct.get_directory_entry_by_identifier(&data_stream, 2)?;
        assert!(result.is_some());

        let result: Option<HfsDirectoryEntry> =
            test_struct.get_directory_entry_by_identifier(&data_stream, 999)?;
        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests for get_directory_entry_by_name

    #[test]
    fn test_get_directory_entry_by_thread_record_from_node_with_hfsplus() -> Result<(), ErrorTrace>
    {
        let data_stream: DataStreamReference = get_data_stream_hfsplus()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfsplus(&data_stream)?;

        let mut thread_record: HfsCatalogThreadRecord = test_struct
            .get_thread_record_by_identifier(&data_stream, 2)?
            .unwrap();

        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        let result: Option<HfsDirectoryEntry> = test_struct
            .get_directory_entry_by_thread_record_from_node(
                &data_stream,
                test_struct.btree_file.root_node_number,
                &thread_record,
                &mut read_node_numbers,
            )?;
        assert!(result.is_some());

        thread_record.parent_identifier = 999;

        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        let result: Option<HfsDirectoryEntry> = test_struct
            .get_directory_entry_by_thread_record_from_node(
                &data_stream,
                test_struct.btree_file.root_node_number,
                &thread_record,
                &mut read_node_numbers,
            )?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_thread_record_by_identifier_with_hfsplus() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfsplus()?;
        let test_struct: HfsCatalogFile = get_catalog_file_hfsplus(&data_stream)?;

        let result: Option<HfsCatalogThreadRecord> =
            test_struct.get_thread_record_by_identifier(&data_stream, 2)?;
        assert!(result.is_some());

        let result: Option<HfsCatalogThreadRecord> =
            test_struct.get_thread_record_by_identifier(&data_stream, 999)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_initialize_with_hfsplus() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream_hfsplus()?;

        let mut test_struct: HfsCatalogFile = HfsCatalogFile::new();
        test_struct.initialize(
            &HfsFormat::HfsPlus,
            4096,
            81920,
            vec![HfsBlockRange::new(0, 242, 20)],
            &data_stream,
        )
    }

    // TODO: add tests for read_directory_entry
    // TODO: add tests for read_thread_record
}
