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

use std::collections::{BTreeMap, HashSet};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u32_be;

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

    /// Retieves attributes.
    pub fn get_attributes_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        identifier: u32,
        attributes: &mut BTreeMap<HfsString, HfsAttributeRecord>,
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
        attributes: &mut BTreeMap<HfsString, HfsAttributeRecord>,
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
            if !is_branch {
                if key.identifier == identifier {
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
                    match self.read_attribute_record(&key, record_data) {
                        Ok(attribute_record) => {
                            attributes.insert(name, attribute_record);
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read attribute record: {}", record_index)
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
        let data_offset: usize = key.size as usize;

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

    use keramics_core::open_os_data_stream;

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
            81920,
            vec![HfsBlockRange::new(0, 22, 20)],
            data_stream,
        )?;
        Ok(attributes_file)
    }

    #[test]
    fn test_get_attributes_by_identifier() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;
        let test_struct: HfsAttributesFile = get_attributes_file(&data_stream)?;

        let mut attributes: BTreeMap<HfsString, HfsAttributeRecord> = BTreeMap::new();
        test_struct.get_attributes_by_identifier(&data_stream, 34, &mut attributes)?;
        assert_eq!(attributes.len(), 1);

        Ok(())
    }

    // TODO: add tests for get_attributes_by_identifier_from_node

    #[test]
    fn test_initialize() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_data_stream()?;

        let mut test_struct: HfsAttributesFile = HfsAttributesFile::new();
        test_struct.initialize(
            &HfsFormat::HfsPlus,
            4096,
            81920,
            vec![HfsBlockRange::new(0, 22, 20)],
            &data_stream,
        )
    }

    // TODO: add tests for read_attribute_record
}
