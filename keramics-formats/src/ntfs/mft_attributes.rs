/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace, FakeDataStream};
use keramics_types::Ucs2String;

use super::block_stream::NtfsBlockStream;
use super::compressed_stream::NtfsCompressedStream;
use super::constants::*;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_attribute_group::NtfsMftAttributeGroup;
use super::reparse_point::NtfsReparsePoint;
use super::standard_information::NtfsStandardInformation;
use super::wof_compressed_stream::NtfsWofCompressedStream;

/// New Technologies File System (NTFS) Master File Table (MFT) attributes.
pub struct NtfsMftAttributes {
    /// Attributes.
    pub attributes: Vec<NtfsMftAttribute>,

    /// Attribute groups per name.
    pub(super) attribute_groups: HashMap<Option<Ucs2String>, NtfsMftAttributeGroup>,

    /// Index of the attribute list attribute in the attributes vector.
    pub attribute_list: Option<usize>,

    /// Indexes of data attributes in the attributes vector.
    data_attributes: Vec<usize>,

    /// Reparse point.
    pub reparse_point: Option<NtfsReparsePoint>,

    /// Standard information.
    pub standard_information: Option<NtfsStandardInformation>,
}

impl NtfsMftAttributes {
    /// Creates new attributes.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            attribute_groups: HashMap::new(),
            attribute_list: None,
            data_attributes: Vec::new(),
            reparse_point: None,
            standard_information: None,
        }
    }

    /// Adds an attribute.
    pub fn add_attribute(&mut self, mut attribute: NtfsMftAttribute) -> Result<(), ErrorTrace> {
        if !self.attribute_groups.contains_key(&attribute.name) {
            let attribute_name: Option<Ucs2String> = match &attribute.name {
                Some(name) => Some(name.clone()),
                None => None,
            };
            let attribute_group: NtfsMftAttributeGroup = NtfsMftAttributeGroup::new();
            self.attribute_groups
                .insert(attribute_name, attribute_group);
        }
        let attribute_group: &mut NtfsMftAttributeGroup =
            match self.attribute_groups.get_mut(&attribute.name) {
                Some(attribute_group) => attribute_group,
                None => {
                    return Err(keramics_core::error_trace_new!("Missing attribute group"));
                }
            };
        match attribute_group.get_attribute_index(attribute.attribute_type) {
            Some(existing_attribute_index) => {
                match self.attributes.get_mut(*existing_attribute_index) {
                    Some(existing_attribute) => match existing_attribute.merge(&mut attribute) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to merge attribute"
                            );
                            return Err(error);
                        }
                    },
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing attribute: {}",
                            existing_attribute_index
                        )));
                    }
                };
            }
            None => {
                let attribute_index: usize = self.attributes.len();
                let attribute_type: u32 = attribute.attribute_type;

                match attribute_type {
                    NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST => {
                        if self.attribute_list.is_some() {
                            return Err(keramics_core::error_trace_new!(
                                "Attribute list already set"
                            ));
                        }
                        self.attribute_list = Some(attribute_index);
                    }
                    NTFS_ATTRIBUTE_TYPE_DATA => {
                        self.data_attributes.push(attribute_index);
                    }
                    NTFS_ATTRIBUTE_TYPE_REPARSE_POINT => {
                        if self.reparse_point.is_some() {
                            return Err(keramics_core::error_trace_new!(
                                "Reparse point already set"
                            ));
                        }
                        let reparse_point: NtfsReparsePoint =
                            match NtfsReparsePoint::from_attribute(&attribute) {
                                Ok(reparse_point) => reparse_point,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to create reparse point from attribute"
                                    );
                                    return Err(error);
                                }
                            };
                        self.reparse_point = Some(reparse_point);
                    }
                    NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION => {
                        if self.standard_information.is_some() {
                            return Err(keramics_core::error_trace_new!(
                                "Standard information already set"
                            ));
                        }
                        let standard_information: NtfsStandardInformation =
                            match NtfsStandardInformation::from_attribute(&attribute) {
                                Ok(standard_information) => standard_information,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to create standard information from attribute"
                                    );
                                    return Err(error);
                                }
                            };
                        self.standard_information = Some(standard_information);
                    }
                    _ => {}
                };
                // A MFT entry can have mutliple $FILE_NAME attributes.
                if attribute_type != NTFS_ATTRIBUTE_TYPE_FILE_NAME {
                    attribute_group.add_attribute_index(attribute_type, attribute_index);
                }
                self.attributes.push(attribute);
            }
        };
        Ok(())
    }

    /// Retrieves the number of attributes.
    pub fn get_number_of_attributes(&self) -> usize {
        self.attributes.len()
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute_by_index(
        &self,
        attribute_index: usize,
    ) -> Result<&NtfsMftAttribute, ErrorTrace> {
        match self.attributes.get(attribute_index) {
            Some(mft_attribute) => Ok(mft_attribute),
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing attribute: {}",
                    attribute_index
                )));
            }
        }
    }

    /// Retrieves a specific attribute of a specific attribute group.
    pub fn get_attribute_for_group(
        &self,
        attribute_group: &NtfsMftAttributeGroup,
        attribute_type: u32,
    ) -> Option<&NtfsMftAttribute> {
        match attribute_group.get_attribute_index(attribute_type) {
            Some(attribute_index) => self.attributes.get(*attribute_index),
            None => None,
        }
    }

    /// Retrieves a specific attribute group.
    pub fn get_attribute_group(&self, name: &Option<Ucs2String>) -> Option<&NtfsMftAttributeGroup> {
        self.attribute_groups.get(name)
    }

    /// Determines if the MFT attributes have a specific attribute group.
    pub fn has_attribute_group(&self, name: &Option<Ucs2String>) -> bool {
        self.attribute_groups.contains_key(name)
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute(
        &self,
        name: &Option<Ucs2String>,
        attribute_type: u32,
    ) -> Option<&NtfsMftAttribute> {
        match self.attribute_groups.get(name) {
            Some(attribute_group) => self.get_attribute_for_group(attribute_group, attribute_type),
            None => None,
        }
    }

    /// Retrieves the number of data attributes.
    pub fn get_number_of_data_attributes(&self) -> usize {
        self.data_attributes.len()
    }

    /// Retrieves a specific data attribute.
    pub fn get_data_attribute_by_index(&self, attribute_index: usize) -> Option<&NtfsMftAttribute> {
        match self.data_attributes.get(attribute_index) {
            Some(attribute_index) => self.attributes.get(*attribute_index),
            None => None,
        }
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: &Option<Ucs2String>,
        data_stream: &DataStreamReference,
        cluster_block_size: u32,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let data_attribute: &NtfsMftAttribute =
            match self.get_attribute(name, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(data_attribute) => data_attribute,
                None => return Ok(None),
            };
        let wof_compression_method: Option<u32> = match &self.reparse_point {
            Some(NtfsReparsePoint::WindowsOverlayFilter { reparse_data }) => match name {
                None => Some(reparse_data.compression_method),
                _ => None,
            },
            _ => None,
        };
        if let Some(compression_method) = wof_compression_method {
            if data_attribute.is_resident() {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported resident $DATA attribute"
                ));
            }
            let attribute_name: Ucs2String = Ucs2String::from("WofCompressedData");
            let wof_data_attribute: &NtfsMftAttribute =
                match self.get_attribute(&Some(attribute_name), NTFS_ATTRIBUTE_TYPE_DATA) {
                    Some(mft_attribute) => mft_attribute,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing WofCompressedData $DATA attribute"
                        ));
                    }
                };
            let mut wof_compressed_stream: NtfsWofCompressedStream =
                NtfsWofCompressedStream::new(cluster_block_size, compression_method);
            match wof_compressed_stream.open(
                data_stream,
                wof_data_attribute,
                data_attribute.valid_data_size,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to open WoF compressed stream"
                    );
                    return Err(error);
                }
            }
            Ok(Some(Arc::new(RwLock::new(wof_compressed_stream))))
        } else if data_attribute.is_resident() {
            // A resident $DATA attribute with a compression type in the data flags is stored uncompressed.
            let data_stream: FakeDataStream =
                FakeDataStream::new(&data_attribute.resident_data, data_attribute.data_size);
            Ok(Some(Arc::new(RwLock::new(data_stream))))
        } else if data_attribute.is_compressed() {
            let mut compressed_stream: NtfsCompressedStream =
                NtfsCompressedStream::new(cluster_block_size);

            match compressed_stream.open(data_stream, data_attribute) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to open compressed stream"
                    );
                    return Err(error);
                }
            }
            Ok(Some(Arc::new(RwLock::new(compressed_stream))))
        } else {
            let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(cluster_block_size);
            match block_stream.open(data_stream, data_attribute) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open block stream");
                    return Err(error);
                }
            }
            Ok(Some(Arc::new(RwLock::new(block_stream))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for add_attribute

    // TODO: add tests for get_number_of_attributes
    // TODO: add tests for get_attribute_by_index

    // TODO: add tests for get_attribute_for_group
    // TODO: add tests for get_attribute_group
    // TODO: add tests for has_attribute_group

    // TODO: add tests for get_attribute

    // TODO: add tests for get_number_of_data_attributes
    // TODO: add tests for get_data_attribute_by_index

    // TODO: add tests for get_data_stream_by_name
}
