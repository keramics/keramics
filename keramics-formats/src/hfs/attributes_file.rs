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

use crate::indexed_hash_map::IndexedHashMap;

use super::attribute_extents_record::HfsAttributeExtentsRecord;
use super::attribute_fork_data_record::HfsAttributeForkDataRecord;
use super::attribute_inline_data_record::HfsAttributeInlineDataRecord;
use super::attribute_key::HfsAttributeKey;
use super::attribute_record::HfsAttributeRecord;
use super::block_range::HfsBlockRange;
use super::btree_file::HfsBtreeFile;
use super::btree_node::HfsBtreeNode;
use super::enums::{HfsBtreeNodeType, HfsFormat};
use super::string::HfsString;

/// Hierarchical File System (HFS) attributes file.
pub struct HfsAttributesFile {
    /// B-tree file.
    btree_file: HfsBtreeFile,
}

impl HfsAttributesFile {
    /// Creates a new attributes file.
    pub fn new() -> Self {
        Self {
            btree_file: HfsBtreeFile::new(),
        }
    }

    /// Retrieves attributes.
    pub fn get_attributes_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        identifier: u32,
        attributes: &mut IndexedHashMap<HfsString, HfsAttributeRecord>,
    ) -> Result<(), ErrorTrace> {
        if self.btree_file.root_node_number == 0 {
            return Ok(());
        }
        let mut read_node_numbers: HashSet<u32> = HashSet::new();

        match self.get_attributes_by_identifier_from_node(
            data_stream,
            self.btree_file.root_node_number,
            identifier,
            attributes,
            &mut read_node_numbers,
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
        node_number: u32,
        identifier: u32,
        attributes: &mut IndexedHashMap<HfsString, HfsAttributeRecord>,
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
        let mut last_key: HfsAttributeKey = HfsAttributeKey::new();
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
                format!("HfsAttributeKey of record: {}", record_index),
                node.get_record_offset_by_index(record_index),
                record_data,
                record_data.len(),
                HfsAttributeKey::debug_read_data(record_data)
            );
            let mut key: HfsAttributeKey = HfsAttributeKey::new();

            match key.read_data(record_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read key: {}", record_index)
                    );
                    return Err(error);
                }
            }
            if key.size >= 6 {
                if key.identifier > identifier {
                    break;
                }
                if key.identifier == identifier {
                    if !is_branch {
                        let name: HfsString = match key.read_name(record_data) {
                            Ok(name) => name,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read name of key: {}", record_index)
                                );
                                return Err(error);
                            }
                        };
                        let attribute_record: HfsAttributeRecord = match self
                            .read_attribute_record(&key, record_data)
                        {
                            Ok(attribute_record) => attribute_record,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!("Unable to read attribute record: {}", record_index)
                                );
                                return Err(error);
                            }
                        };
                        match attribute_record {
                            HfsAttributeRecord::Extents(mut extents_attribute_record) => {
                                match attributes.get_value_by_key_mut(&name) {
                                    Some(HfsAttributeRecord::ForkData(fork_attribute_record)) => {
                                        if key.logical_block_number == 0 {
                                            return Err(keramics_core::error_trace_new!(format!(
                                                "Unsupported key: {} with zero logical block number",
                                                record_index
                                            )));
                                        }
                                        extents_attribute_record.logical_block_number =
                                            key.logical_block_number;

                                        fork_attribute_record
                                            .extents_records
                                            .push(extents_attribute_record)
                                    }
                                    _ => {
                                        return Err(keramics_core::error_trace_new!(format!(
                                            "Missing attribute: {} fork data record",
                                            name
                                        )));
                                    }
                                }
                            }
                            _ => {
                                if key.logical_block_number != 0 {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Unsupported key: {} with non-zero logical block number",
                                        record_index
                                    )));
                                }
                                if attributes.insert(name.clone(), attribute_record).is_some() {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Unsupported multiple attributes with the name: {}",
                                        name
                                    )));
                                }
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
                            format!("HfsAttributesBranchNodeValue: {}", record_index),
                            node.get_record_offset_by_index(record_index - 1),
                            &last_record_data[data_offset..data_offset + 4],
                            4
                        );
                        let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

                        match self.get_attributes_by_identifier_from_node(
                            data_stream,
                            sub_node_number,
                            identifier,
                            attributes,
                            read_node_numbers,
                        ) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to retrieve attributes from node: {}",
                                        sub_node_number
                                    )
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
        if is_branch && record_index > 0 {
            let data_offset: usize = last_key.size;

            if data_offset + 4 > last_record_data.len() {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid data size of record: {} value out of bounds",
                    record_index
                )));
            }
            keramics_core::debug_trace_data!(
                format!("HfsAttributesBranchNodeValue: {}", record_index),
                node.get_record_offset_by_index(record_index - 1),
                &last_record_data[data_offset..data_offset + 4],
                4
            );
            let sub_node_number: u32 = bytes_to_u32_be!(last_record_data, data_offset);

            match self.get_attributes_by_identifier_from_node(
                data_stream,
                sub_node_number,
                identifier,
                attributes,
                read_node_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve attributes from node: {}",
                            sub_node_number
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Initializes the attributes file.
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

    /// Reads an attribute record.
    fn read_attribute_record(
        &self,
        key: &HfsAttributeKey,
        record_data: &[u8],
    ) -> Result<HfsAttributeRecord, ErrorTrace> {
        let data_offset: usize = key.size;

        if data_offset + 4 > record_data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid record data size value out of bounds"
            )));
        }
        let record_type: u32 = bytes_to_u32_be!(record_data, data_offset);

        match record_type {
            0x00000010 => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsAttributeInlineDataRecord",
                    0,
                    &record_data[data_offset..],
                    record_data.len() - data_offset,
                    HfsAttributeInlineDataRecord::debug_read_data(&record_data[data_offset..])
                );
                let mut attribute_record: HfsAttributeInlineDataRecord =
                    HfsAttributeInlineDataRecord::new();

                match attribute_record.read_data(&record_data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute inline data record"
                        );
                        return Err(error);
                    }
                }
                Ok(HfsAttributeRecord::InlineData(attribute_record))
            }
            0x00000020 => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsAttributeForkDataRecord",
                    0,
                    &record_data[data_offset..],
                    record_data.len() - data_offset,
                    HfsAttributeForkDataRecord::debug_read_data(&record_data[data_offset..])
                );
                let mut attribute_record: HfsAttributeForkDataRecord =
                    HfsAttributeForkDataRecord::new();

                match attribute_record.read_data(&record_data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute fork data record"
                        );
                        return Err(error);
                    }
                }
                Ok(HfsAttributeRecord::ForkData(attribute_record))
            }
            0x00000030 => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsAttributeExtentsRecord",
                    0,
                    &record_data[data_offset..],
                    record_data.len() - data_offset,
                    HfsAttributeExtentsRecord::debug_read_data(&record_data[data_offset..])
                );
                let mut attribute_record: HfsAttributeExtentsRecord =
                    HfsAttributeExtentsRecord::new();

                match attribute_record.read_data(&record_data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute extents record"
                        );
                        return Err(error);
                    }
                }
                Ok(HfsAttributeRecord::Extents(attribute_record))
            }
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:08x}",
                record_type
            ))),
        }
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

    fn get_attributes_file(
        data_stream: &DataStreamReference,
    ) -> Result<HfsAttributesFile, ErrorTrace> {
        let mut attributes_file: HfsAttributesFile = HfsAttributesFile::new();

        attributes_file.initialize(
            &HfsFormat::HfsPlus,
            4096,
            65536,
            vec![HfsBlockRange::new(0, 10, 16)],
            data_stream,
        )?;
        Ok(attributes_file)
    }

    #[test]
    fn test_get_attributes_by_identifier() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;
        let test_struct: HfsAttributesFile = get_attributes_file(&data_stream)?;

        let mut attributes: IndexedHashMap<HfsString, HfsAttributeRecord> = IndexedHashMap::new();
        test_struct.get_attributes_by_identifier(&data_stream, 34, &mut attributes)?;
        assert_eq!(attributes.len(), 1);

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

    fn build_attribute_key(identifier: u32, logical_block_number: u32, name: &str) -> Vec<u8> {
        let utf16_units: Vec<u16> = name.encode_utf16().collect();
        let key_data_size: u16 = (12 + (utf16_units.len() * 2)) as u16;
        let mut key: Vec<u8> = vec![0; 2 + (key_data_size as usize)];
        key[0..2].copy_from_slice(&key_data_size.to_be_bytes());
        key[4..8].copy_from_slice(&identifier.to_be_bytes());
        key[8..12].copy_from_slice(&logical_block_number.to_be_bytes());
        key[12..14].copy_from_slice(&(utf16_units.len() as u16).to_be_bytes());
        for (index, unit) in utf16_units.iter().enumerate() {
            let offset: usize = 14 + (index * 2);
            key[offset..offset + 2].copy_from_slice(&unit.to_be_bytes());
        }
        key
    }

    #[test]
    fn test_read_attribute_record() -> Result<(), ErrorTrace> {
        let attributes_file: HfsAttributesFile = HfsAttributesFile::new();
        let key_bytes: Vec<u8> = build_attribute_key(10, 0, "attr");
        let mut key: HfsAttributeKey = HfsAttributeKey::new();
        key.read_data(&key_bytes)?;

        // Test inline data record (0x00000010).
        let mut inline_record: Vec<u8> = key_bytes.clone();
        inline_record.extend_from_slice(&0x00000010u32.to_be_bytes());
        inline_record.extend_from_slice(&[0; 8]);
        inline_record.extend_from_slice(&4u32.to_be_bytes());
        inline_record.extend_from_slice(b"test");
        let record: HfsAttributeRecord =
            attributes_file.read_attribute_record(&key, &inline_record)?;
        assert!(matches!(record, HfsAttributeRecord::InlineData(_)));

        // Test fork data record (0x00000020).
        let mut fork_record: Vec<u8> = key_bytes.clone();
        fork_record.extend_from_slice(&0x00000020u32.to_be_bytes());
        fork_record.extend_from_slice(&[0; 84]);
        let record: HfsAttributeRecord =
            attributes_file.read_attribute_record(&key, &fork_record)?;
        assert!(matches!(record, HfsAttributeRecord::ForkData(_)));

        // Test extents record (0x00000030).
        let mut extents_record: Vec<u8> = key_bytes.clone();
        extents_record.extend_from_slice(&0x00000030u32.to_be_bytes());
        extents_record.extend_from_slice(&[0; 68]);
        let record: HfsAttributeRecord =
            attributes_file.read_attribute_record(&key, &extents_record)?;
        assert!(matches!(record, HfsAttributeRecord::Extents(_)));

        // Test unsupported record type (0x00000040).
        let mut unknown_record: Vec<u8> = key_bytes.clone();
        unknown_record.extend_from_slice(&0x00000040u32.to_be_bytes());
        let result: Result<HfsAttributeRecord, ErrorTrace> =
            attributes_file.read_attribute_record(&key, &unknown_record);
        assert!(result.is_err());

        // Test truncated record data size (< key.size + 4).
        let result: Result<HfsAttributeRecord, ErrorTrace> =
            attributes_file.read_attribute_record(&key, &key_bytes);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_attributes_by_identifier_branch_fork_and_errors() -> Result<(), ErrorTrace> {
        let mut image: Vec<u8> = vec![0; 6 * 512];
        // Node 0: Header node pointing to root node 1.
        image[8] = 0x01;
        image[16..20].copy_from_slice(&1u32.to_be_bytes());
        image[32..34].copy_from_slice(&512u16.to_be_bytes());

        // Node 1: Index node pointing to empty leaf 5 (id 10), leaf 2 (id 20), and map node 3 (id 50).
        let mut index_record_1: Vec<u8> = build_attribute_key(10, 0, "a");
        index_record_1.extend_from_slice(&5u32.to_be_bytes());
        let mut index_record_2: Vec<u8> = build_attribute_key(20, 0, "a");
        index_record_2.extend_from_slice(&2u32.to_be_bytes());
        let mut index_record_3: Vec<u8> = build_attribute_key(50, 0, "a");
        index_record_3.extend_from_slice(&3u32.to_be_bytes());
        image[512..1024].copy_from_slice(&build_hfs_btree_node(
            0x00,
            &[&index_record_1, &index_record_2, &index_record_3],
        ));

        // Node 2: Leaf node with ForkData + Extents for id 20 ("big"), and InlineData for id 30 ("end").
        let mut fork_record: Vec<u8> = build_attribute_key(20, 0, "big");
        fork_record.extend_from_slice(&0x00000020u32.to_be_bytes());
        fork_record.extend_from_slice(&[0; 84]);

        let mut extents_record: Vec<u8> = build_attribute_key(20, 8, "big");
        extents_record.extend_from_slice(&0x00000030u32.to_be_bytes());
        extents_record.extend_from_slice(&[0; 68]);

        let mut next_record: Vec<u8> = build_attribute_key(30, 0, "end");
        next_record.extend_from_slice(&0x00000010u32.to_be_bytes());
        next_record.extend_from_slice(&[0; 12]);

        image[1024..1536].copy_from_slice(&build_hfs_btree_node(
            0xff,
            &[&fork_record, &extents_record, &next_record],
        ));

        // Node 3: MapNode (unsupported node type).
        image[1536..2048].copy_from_slice(&build_hfs_btree_node(0x02, &[]));

        // Node 4: Leaf node with orphan Extents record (missing ForkData) for id 40,
        // and duplicate inline record for id 41.
        let mut orphan_extents: Vec<u8> = build_attribute_key(40, 8, "orphan");
        orphan_extents.extend_from_slice(&0x00000030u32.to_be_bytes());
        orphan_extents.extend_from_slice(&[0; 68]);

        let mut duplicate_inline: Vec<u8> = build_attribute_key(41, 0, "dup");
        duplicate_inline.extend_from_slice(&0x00000010u32.to_be_bytes());
        duplicate_inline.extend_from_slice(&[0; 12]);

        let mut nonzero_inline: Vec<u8> = build_attribute_key(42, 4, "bad");
        nonzero_inline.extend_from_slice(&0x00000010u32.to_be_bytes());
        nonzero_inline.extend_from_slice(&[0; 12]);

        image[2048..2560].copy_from_slice(&build_hfs_btree_node(
            0xff,
            &[
                &orphan_extents,
                &duplicate_inline,
                &duplicate_inline,
                &nonzero_inline,
            ],
        ));

        // Node 5: Empty LeafNode.
        image[2560..3072].copy_from_slice(&build_hfs_btree_node(0xff, &[]));

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut attributes_file: HfsAttributesFile = HfsAttributesFile::new();
        attributes_file.initialize(
            &HfsFormat::HfsPlus,
            512,
            image.len() as u64,
            vec![HfsBlockRange::new(0, 0, 6)],
            &data_stream,
        )?;

        let mut attributes: IndexedHashMap<HfsString, HfsAttributeRecord> = IndexedHashMap::new();
        attributes_file.get_attributes_by_identifier(&data_stream, 20, &mut attributes)?;
        assert_eq!(attributes.len(), 1);

        // Unsupported node type error via index lookup for id 50.
        let mut error_attributes: IndexedHashMap<HfsString, HfsAttributeRecord> =
            IndexedHashMap::new();
        let result: Result<(), ErrorTrace> =
            attributes_file.get_attributes_by_identifier(&data_stream, 50, &mut error_attributes);
        assert!(result.is_err());

        // Orphan extents, duplicate attribute name, and non-zero logical block number on inline data.
        for identifier in [40u32, 41, 42] {
            let mut read_node_numbers: HashSet<u32> = HashSet::new();
            let mut test_attributes: IndexedHashMap<HfsString, HfsAttributeRecord> =
                IndexedHashMap::new();
            let result: Result<(), ErrorTrace> = attributes_file
                .get_attributes_by_identifier_from_node(
                    &data_stream,
                    4,
                    identifier,
                    &mut test_attributes,
                    &mut read_node_numbers,
                );
            assert!(result.is_err());
        }

        // Cycle detection error.
        let mut read_node_numbers: HashSet<u32> = HashSet::new();
        read_node_numbers.insert(2);
        let result: Result<(), ErrorTrace> = attributes_file
            .get_attributes_by_identifier_from_node(
                &data_stream,
                2,
                20,
                &mut error_attributes,
                &mut read_node_numbers,
            );
        assert!(result.is_err());

        // Root node 0 returns Ok(()).
        let empty_file: HfsAttributesFile = HfsAttributesFile::new();
        empty_file.get_attributes_by_identifier(&data_stream, 20, &mut error_attributes)?;

        Ok(())
    }

    #[test]
    fn test_initialize() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;

        let mut test_struct: HfsAttributesFile = HfsAttributesFile::new();
        test_struct.initialize(
            &HfsFormat::HfsPlus,
            4096,
            65536,
            vec![HfsBlockRange::new(0, 10, 16)],
            &data_stream,
        )
    }
}
