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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStream, DataStreamReference, ErrorTrace, FakeDataStream};
use keramics_datetime::DateTime;
use keramics_types::ByteString;

use crate::decmpfs::{
    DecmpfsBlockReader, DecmpfsCompressionMethod, DecmpfsDataStream, DecmpfsHeader,
};
use crate::indexed_hash_map::IndexedHashMap;
use crate::path_component::PathComponent;
use crate::traits::{ExtendedAttributeIterator, FileEntryIterator};

use super::attribute_record::HfsAttributeRecord;
use super::attributes_file::HfsAttributesFile;
use super::block_ranges::HfsBlockRanges;
use super::block_reader::HfsBlockReader;
use super::block_stream::HfsBlockStream;
use super::catalog_file::HfsCatalogFile;
use super::constants::*;
use super::directory_entry::HfsDirectoryEntry;
use super::enums::HfsForkType;
use super::extended_attribute::HfsExtendedAttribute;
use super::extended_attributes::HfsExtendedAttributesIterator;
use super::extents_overflow_file::HfsExtentsOverflowFile;
use super::file_entries::HfsFileEntriesIterator;
use super::fork::HfsFork;
use super::fork_descriptor::HfsForkDescriptor;
use super::string::HfsString;

/// Hierarchical File System (HFS) file entry.
pub struct HfsFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Data area block number.
    data_area_block_number: u16,

    /// Catalog file.
    catalog_file: Arc<HfsCatalogFile>,

    /// Extents overflow file.
    extents_overflow_file: Arc<HfsExtentsOverflowFile>,

    /// Attributes file.
    attributes_file: Arc<HfsAttributesFile>,

    /// Identifier (CNID).
    pub(super) identifier: u32,

    /// Directory entry.
    directory_entry: HfsDirectoryEntry,

    /// Indirect node.
    indirect_node: Option<HfsDirectoryEntry>,

    /// Compressed data header.
    compressed_data_header: Option<DecmpfsHeader>,

    /// Value to indicate to hide the resource fork.
    hide_resource_fork: bool,

    /// Sub directory entries.
    sub_directory_entries: IndexedHashMap<HfsString, HfsDirectoryEntry>,

    /// Value to indicate the sub directory entries were read.
    sub_directory_entries_read: bool,

    /// Symbolic link target.
    symbolic_link_target: Option<ByteString>,

    /// Attributes.
    attributes: IndexedHashMap<HfsString, HfsAttributeRecord>,
}

impl HfsFileEntry {
    /// Creates a new file entry.
    pub fn new(
        data_stream: &DataStreamReference,
        block_size: u32,
        data_area_block_number: u16,
        catalog_file: &Arc<HfsCatalogFile>,
        extents_overflow_file: &Arc<HfsExtentsOverflowFile>,
        attributes_file: &Arc<HfsAttributesFile>,
        directory_entry: HfsDirectoryEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            data_area_block_number,
            catalog_file: catalog_file.clone(),
            extents_overflow_file: extents_overflow_file.clone(),
            attributes_file: attributes_file.clone(),
            identifier: directory_entry.get_identifier(),
            directory_entry,
            indirect_node: None,
            compressed_data_header: None,
            hide_resource_fork: false,
            sub_directory_entries: IndexedHashMap::new(),
            sub_directory_entries_read: false,
            symbolic_link_target: None,
            attributes: IndexedHashMap::new(),
        }
    }

    /// Creates a new file entry.
    pub(super) fn new_with_initialize(
        data_stream: &DataStreamReference,
        block_size: u32,
        data_area_block_number: u16,
        catalog_file: &Arc<HfsCatalogFile>,
        extents_overflow_file: &Arc<HfsExtentsOverflowFile>,
        attributes_file: &Arc<HfsAttributesFile>,
        directory_entry: HfsDirectoryEntry,
    ) -> Result<Self, ErrorTrace> {
        let mut file_entry: Self = Self::new(
            data_stream,
            block_size,
            data_area_block_number,
            catalog_file,
            extents_overflow_file,
            attributes_file,
            directory_entry,
        );
        match file_entry.read_indirect_node() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read indirect node");
                return Err(error);
            }
        }
        match file_entry.read_attributes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        Ok(file_entry)
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_access_time(),
            None => self.directory_entry.get_access_time(),
        }
    }

    /// Retrieves the backup time.
    pub fn get_backup_time(&self) -> &DateTime {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_backup_time(),
            None => self.directory_entry.get_backup_time(),
        }
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_change_time(),
            None => self.directory_entry.get_change_time(),
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> &DateTime {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_creation_time(),
            None => self.directory_entry.get_creation_time(),
        }
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> Option<&u16> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_file_mode(),
            None => self.directory_entry.get_file_mode(),
        }
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> Option<&u32> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_group_identifier(),
            None => self.directory_entry.get_group_identifier(),
        }
    }

    /// Retrieves the identifier (CNID).
    pub fn get_identifier(&self) -> u32 {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_identifier(),
            None => self.directory_entry.get_identifier(),
        }
    }

    /// Retrieves the link reference.
    pub fn get_link_reference(&self) -> Option<&u32> {
        self.directory_entry.get_link_reference()
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> &DateTime {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_modification_time(),
            None => self.directory_entry.get_modification_time(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&HfsString> {
        self.directory_entry.name.as_ref()
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> u32 {
        self.directory_entry.get_number_of_links()
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> Option<&u32> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_owner_identifier(),
            None => self.directory_entry.get_owner_identifier(),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self.compressed_data_header.as_ref() {
            Some(compressed_data_header) => compressed_data_header.uncompressed_data_size,
            None => match self.get_data_fork_descriptor() {
                Some(fork_descriptor) => fork_descriptor.size,
                None => 0,
            },
        }
    }

    /// Retrieves the block stream.
    fn get_block_stream(
        &self,
        fork_descriptor: &HfsForkDescriptor,
    ) -> Result<HfsBlockStream, ErrorTrace> {
        let identifier: u32 = self.get_identifier();
        let mut block_ranges: HfsBlockRanges = HfsBlockRanges::new();

        match block_ranges.read_fork_descriptor(
            self.data_area_block_number,
            identifier,
            fork_descriptor,
            &self.data_stream,
            &self.extents_overflow_file,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine block ranges from fork descriptor"
                );
                return Err(error);
            }
        }
        let mut block_reader: HfsBlockReader =
            HfsBlockReader::new(&self.data_stream, self.block_size, fork_descriptor.size);

        match block_reader.open(block_ranges.ranges) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open block reader");
                return Err(error);
            }
        }
        Ok(HfsBlockStream::new(block_reader))
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> Result<Option<&ByteString>, ErrorTrace> {
        if self.symbolic_link_target.is_none() && self.is_symbolic_link() {
            let fork_descriptor: &HfsForkDescriptor = match self.get_data_fork_descriptor() {
                Some(fork_descriptor) => fork_descriptor,
                None => return Ok(None),
            };
            if fork_descriptor.size > 1024 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid symbolic link target size values out of bounds"
                ));
            }
            let mut block_stream: HfsBlockStream = match self.get_block_stream(&fork_descriptor) {
                Ok(block_stream) => block_stream,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve block stream");
                    return Err(error);
                }
            };
            let mut data: Vec<u8> = vec![0; fork_descriptor.size as usize];

            match block_stream.read(&mut data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read symbolic link target data from block stream"
                    );
                    return Err(error);
                }
            }
            self.symbolic_link_target = Some(ByteString::from(data.as_slice()));
        }
        Ok(self.symbolic_link_target.as_ref())
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if !self.has_data_fork() {
            return Ok(None);
        }
        match self.compressed_data_header.as_ref() {
            Some(compressed_data_header) => {
                let compression_method: DecmpfsCompressionMethod =
                    match compressed_data_header.get_compression_method() {
                        Some(compression_method) => compression_method,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported compression method: {}",
                                compressed_data_header.compression_method
                            )));
                        }
                    };
                let data_stream: DataStreamReference = match compressed_data_header
                    .compression_method
                {
                    4 | 8 | 12 => {
                        let fork_descriptor: &HfsForkDescriptor =
                            match self.get_resource_fork_descriptor() {
                                Some(fork_descriptor) => fork_descriptor,
                                None => return Ok(None),
                            };
                        match self.get_block_stream(&fork_descriptor) {
                            Ok(block_stream) => Arc::new(RwLock::new(block_stream)),
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve block stream"
                                );
                                return Err(error);
                            }
                        }
                    }
                    _ => {
                        let lookup_name: HfsString = HfsString::from("com.apple.decmpfs");

                        match self.attributes.get_value_by_key(&lookup_name) {
                            Some(attribute_record) => match attribute_record {
                                HfsAttributeRecord::InlineData(attribute_inline_data_record) => {
                                    let data_stream: FakeDataStream = FakeDataStream::new(
                                        &attribute_inline_data_record.data,
                                        attribute_inline_data_record.data_size as u64,
                                    );
                                    Arc::new(RwLock::new(data_stream))
                                }
                                _ => {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unsupported decmpfs attribute record type"
                                    ));
                                }
                            },
                            None => {
                                return Err(keramics_core::error_trace_new!(
                                    "Missing com.apple.decmpfs attribute record"
                                ));
                            }
                        }
                    }
                };
                let mut decmpfs_block_reader: DecmpfsBlockReader =
                    DecmpfsBlockReader::new(&data_stream, compression_method);

                match decmpfs_block_reader.open(compressed_data_header.uncompressed_data_size) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open decmpfs data stream"
                        );
                        return Err(error);
                    }
                }
                Ok(Some(Arc::new(RwLock::new(DecmpfsDataStream::new(
                    decmpfs_block_reader,
                )))))
            }
            None => {
                let fork_descriptor: &HfsForkDescriptor = match self.get_data_fork_descriptor() {
                    Some(fork_descriptor) => fork_descriptor,
                    None => return Ok(None),
                };
                match self.get_block_stream(&fork_descriptor) {
                    Ok(block_stream) => Ok(Some(Arc::new(RwLock::new(block_stream)))),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve block stream"
                        );
                        Err(error)
                    }
                }
            }
        }
    }

    /// Retrieves the data fork.
    pub fn get_data_fork(&mut self) -> Result<Option<HfsFork>, ErrorTrace> {
        if !self.has_data_fork() {
            return Ok(None);
        }
        match self.get_data_stream() {
            Ok(Some(data_stream)) => Ok(Some(HfsFork::new(HfsForkType::Data, data_stream))),
            Ok(None) => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                Err(error)
            }
        }
    }

    /// Retrieves the data fork descriptor.
    fn get_data_fork_descriptor(&self) -> Option<&HfsForkDescriptor> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_data_fork_descriptor(),
            None => self.directory_entry.get_data_fork_descriptor(),
        }
    }

    /// Retrieves the resource fork.
    pub fn get_resource_fork(&mut self) -> Result<Option<HfsFork>, ErrorTrace> {
        if self.hide_resource_fork {
            return Ok(None);
        }
        let fork_descriptor: &HfsForkDescriptor = match self.get_resource_fork_descriptor() {
            Some(fork_descriptor) => fork_descriptor,
            None => return Ok(None),
        };
        match self.get_block_stream(&fork_descriptor) {
            Ok(block_stream) => Ok(Some(HfsFork::new(
                HfsForkType::Resource,
                Arc::new(RwLock::new(block_stream)),
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve block stream");
                Err(error)
            }
        }
    }

    /// Retrieves the resource fork descriptor.
    fn get_resource_fork_descriptor(&self) -> Option<&HfsForkDescriptor> {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.get_resource_fork_descriptor(),
            None => self.directory_entry.get_resource_fork_descriptor(),
        }
    }

    /// Retrieves the data stream of an extended attribute.
    fn get_extended_attribute_data_stream(
        &self,
        attribute_record: &HfsAttributeRecord,
    ) -> Result<DataStreamReference, ErrorTrace> {
        match attribute_record {
            HfsAttributeRecord::Extents(_) => {
                // TODO: implement
                todo!();
            }
            HfsAttributeRecord::ForkData(fork_data_attribute_record) => {
                match self.get_block_stream(&fork_data_attribute_record.fork_descriptor) {
                    Ok(block_stream) => Ok(Arc::new(RwLock::new(block_stream))),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve fork data block stream"
                        );
                        Err(error)
                    }
                }
            }
            HfsAttributeRecord::InlineData(attribute_inline_data_record) => {
                let data_stream: FakeDataStream = FakeDataStream::new(
                    &attribute_inline_data_record.data,
                    attribute_inline_data_record.data_size as u64,
                );
                Ok(Arc::new(RwLock::new(data_stream)))
            }
        }
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_name(
        &self,
        extended_attribute_name: &PathComponent,
    ) -> Result<Option<HfsExtendedAttribute>, ErrorTrace> {
        let lookup_name: HfsString = match extended_attribute_name.to_utf16_string() {
            Ok(utf16_string) => HfsString::Utf16String(utf16_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to convert path component to UTF-16 string"
                );
                return Err(error);
            }
        };
        match self.attributes.get_key_value_by_key(&lookup_name) {
            Some((name, attributes_entry)) => {
                let data_stream: DataStreamReference =
                    match self.get_extended_attribute_data_stream(attributes_entry) {
                        Ok(data_stream) => data_stream,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                Ok(Some(HfsExtendedAttribute::new(name, data_stream)))
            }
            None => Ok(None),
        }
    }

    /// Retrieves an extended attributes iterator.
    pub fn extended_attributes(&mut self) -> HfsExtendedAttributesIterator<'_> {
        HfsExtendedAttributesIterator::new(self)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &PathComponent,
    ) -> Result<Option<Self>, ErrorTrace> {
        let directory_entry: HfsDirectoryEntry = match self
            .catalog_file
            .get_directory_entry_by_name(&self.data_stream, self.identifier, sub_file_entry_name)
        {
            Ok(Some(directory_entry)) => directory_entry,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve directory entry from catalog file"
                );
                return Err(error);
            }
        };
        match Self::new_with_initialize(
            &self.data_stream,
            self.block_size,
            self.data_area_block_number,
            &self.catalog_file,
            &self.extents_overflow_file,
            &self.attributes_file,
            directory_entry,
        ) {
            Ok(file_entry) => Ok(Some(file_entry)),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file entry");
                Err(error)
            }
        }
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> HfsFileEntriesIterator<'_> {
        HfsFileEntriesIterator::new(self)
    }

    /// Determines if the file entry has a data fork.
    pub fn has_data_fork(&self) -> bool {
        if self.compressed_data_header.is_some() {
            return true;
        }
        match self.get_file_mode() {
            Some(file_mode) => *file_mode & 0xf000 == HFS_FILE_MODE_TYPE_REGULAR_FILE,
            None => self.directory_entry.is_file(),
        }
    }

    /// Determines if the file entry has a resource fork.
    pub fn has_resource_fork(&self) -> bool {
        if self.hide_resource_fork {
            return false;
        }
        match self.get_resource_fork_descriptor() {
            Some(fork_descriptor) => fork_descriptor.size > 0,
            None => false,
        }
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        match &self.indirect_node {
            Some(indirect_node) => indirect_node.is_directory(),
            None => self.directory_entry.is_directory(),
        }
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        self.identifier == HFS_ROOT_DIRECTORY_IDENTIFIER
    }

    /// Determines if the file entry is a symbolic link.
    fn is_symbolic_link(&self) -> bool {
        match self.get_file_mode() {
            Some(file_mode) => *file_mode & 0xf000 == HFS_FILE_MODE_TYPE_SYMBOLIC_LINK,
            None => false,
        }
    }

    /// Reads the attributes.
    pub(super) fn read_attributes(&mut self) -> Result<(), ErrorTrace> {
        let identifier: u32 = self.get_identifier();

        match self.attributes_file.get_attributes_by_identifier(
            &self.data_stream,
            identifier,
            &mut self.attributes,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve attributes for identifier: {}",
                        identifier
                    )
                );
                return Err(error);
            }
        }
        let lookup_name: HfsString = HfsString::from("com.apple.decmpfs");

        match self.attributes.get_value_by_key(&lookup_name) {
            Some(attribute_record) => match attribute_record {
                HfsAttributeRecord::InlineData(attribute_inline_data_record) => {
                    let mut compressed_data_header: DecmpfsHeader = DecmpfsHeader::new();

                    keramics_core::debug_trace_data_and_structure!(
                        "DecmpfsHeader",
                        0,
                        &attribute_inline_data_record.data,
                        attribute_inline_data_record.data_size,
                        DecmpfsHeader::debug_read_data(&attribute_inline_data_record.data)
                    );
                    match compressed_data_header.read_data(&attribute_inline_data_record.data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read decmpfs header"
                            );
                            return Err(error);
                        }
                    }
                    match self.get_data_fork_descriptor() {
                        Some(fork_descriptor) => {
                            if fork_descriptor.size != 0 {
                                return Err(keramics_core::error_trace_new!(
                                    "Unsupported non-empty data fork"
                                ));
                            }
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Missing data fork descriptor"
                            ));
                        }
                    }
                    self.hide_resource_fork = match compressed_data_header.compression_method {
                        4 | 8 | 12 | 14 => true,
                        _ => false,
                    };
                    self.compressed_data_header = Some(compressed_data_header);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported decmpfs attribute record type"
                    ));
                }
            },
            None => {}
        }
        Ok(())
    }

    /// Reads the indirect node.
    pub(super) fn read_indirect_node(&mut self) -> Result<(), ErrorTrace> {
        if let Some(link_reference) = self.get_link_reference() {
            if *link_reference > 2 {
                match self
                    .catalog_file
                    .get_directory_entry_by_identifier(&self.data_stream, *link_reference)
                {
                    Ok(Some(directory_entry)) => {
                        self.indirect_node = Some(directory_entry);
                    }
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing directory entry: {}",
                            *link_reference
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve directory entry: {} from catalog file",
                                *link_reference
                            )
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Reads the sub directory entries.
    fn read_sub_directory_entries(&mut self) -> Result<(), ErrorTrace> {
        match self.catalog_file.get_directory_entries_by_identifier(
            &self.data_stream,
            self.identifier,
            &mut self.sub_directory_entries,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve sub directory entries from catalog file"
                );
                return Err(error);
            }
        }
        self.sub_directory_entries_read = true;

        Ok(())
    }
}

impl ExtendedAttributeIterator for HfsFileEntry {
    type ExtendedAttributeItem = HfsExtendedAttribute;

    /// Retrieves the number of extended attributes.
    fn get_number_of_extended_attributes(&mut self) -> Result<usize, ErrorTrace> {
        Ok(self.attributes.len())
    }

    /// Retrieves a specific extended attribute.
    fn get_extended_attribute_by_index(
        &mut self,
        extended_attribute_index: usize,
    ) -> Result<HfsExtendedAttribute, ErrorTrace> {
        match self
            .attributes
            .get_key_value_by_index(extended_attribute_index)
        {
            Some((name, attribute_record)) => {
                let data_stream: DataStreamReference =
                    match self.get_extended_attribute_data_stream(attribute_record) {
                        Ok(data_stream) => data_stream,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                Ok(HfsExtendedAttribute::new(name, data_stream))
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing extended attribute: {}",
                extended_attribute_index
            ))),
        }
    }
}

impl FileEntryIterator for HfsFileEntry {
    /// Retrieves the number of sub file entries.
    fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries_read {
            match self.read_sub_directory_entries() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read sub directory entries"
                    );
                    return Err(error);
                }
            }
        }
        Ok(self.sub_directory_entries.len())
    }

    /// Retrieves a specific sub file entry.
    fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<Self, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries_read {
            match self.read_sub_directory_entries() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read sub directory entries"
                    );
                    return Err(error);
                }
            }
        }
        match self
            .sub_directory_entries
            .get_key_value_by_index(sub_file_entry_index)
        {
            Some((name, directory_entry)) => {
                let mut sub_directory_entry: HfsDirectoryEntry = directory_entry.clone();
                sub_directory_entry.name = Some(name.clone());

                match Self::new_with_initialize(
                    &self.data_stream,
                    self.block_size,
                    self.data_area_block_number,
                    &self.catalog_file,
                    &self.extents_overflow_file,
                    &self.attributes_file,
                    sub_directory_entry,
                ) {
                    Ok(file_entry) => Ok(file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to create file entry");
                        Err(error)
                    }
                }
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing directory entry: {}",
                sub_file_entry_index
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::HfsTime;
    use keramics_encodings::CharacterEncoding;
    use keramics_types::Utf16String;

    use crate::hfs::file_system::HfsFileSystem;
    use crate::path::Path;

    use crate::tests::get_test_data_path;

    fn get_file_system(path_string: &str) -> Result<HfsFileSystem, ErrorTrace> {
        let mut file_system: HfsFileSystem = HfsFileSystem::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    // Tests with HFS.

    #[test]
    fn test_get_access_time_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_backup_time_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_backup_time(), &DateTime::NotSet);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_change_time(), None);
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_creation_time(),
            &DateTime::HfsTime(HfsTime {
                timestamp: 3849937114
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_file_mode(), None);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_group_identifier(), None);

        Ok(())
    }

    // TODO: add tests for get_identifier
    // TODO: add tests for get_link_reference

    #[test]
    fn test_get_modification_time_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_modification_time(),
            &DateTime::HfsTime(HfsTime {
                timestamp: 3849937114
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<&HfsString> = hfs_file_entry.get_name();
        let expected_name: HfsString = HfsString::ByteString(ByteString {
            encoding: CharacterEncoding::MacRoman,
            elements: vec![116, 101, 115, 116, 102, 105, 108, 101, 49],
        });
        assert_eq!(name, Some(expected_name).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_number_of_links(), 1);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_owner_identifier(), None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_size(), 9);

        Ok(())
    }

    // TODO: add tests for get_block_stream

    #[test]
    fn test_get_symbolic_link_target_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let symbolic_link_target: Option<&ByteString> =
            hfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(symbolic_link_target, None);

        let path: Path = Path::from("/file_symboliclink1");
        let result: Option<HfsFileEntry> = hfs_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = hfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = hfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_data_fork

    #[test]
    fn test_get_data_fork_descriptor_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_data_fork_descriptor();
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_data_fork_descriptor();
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_resource_fork

    #[test]
    fn test_get_resource_fork_descriptor_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_resource_fork_descriptor();
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_resource_fork_descriptor();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_attributes: usize = hfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Result<HfsExtendedAttribute, ErrorTrace> =
            hfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::from("myxattr1");
        let result: Option<HfsExtendedAttribute> =
            hfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut extended_attributes_iterator: HfsExtendedAttributesIterator =
            hfs_file_entry.extended_attributes();

        let result: Option<Result<HfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = hfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = hfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: HfsFileEntry = hfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<&HfsString> = sub_file_entry.get_name();
        let expected_name: HfsString = HfsString::ByteString(ByteString {
            encoding: CharacterEncoding::MacRoman,
            elements: vec![116, 101, 115, 116, 102, 105, 108, 101, 49],
        });
        assert_eq!(name, Some(expected_name).as_ref());

        let result: Result<HfsFileEntry, ErrorTrace> =
            hfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_name_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::ByteString(ByteString::from("testfile1"));
        let result: Option<HfsFileEntry> = hfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_some());

        let name: PathComponent = PathComponent::ByteString(ByteString::from("bogus"));
        let result: Option<HfsFileEntry> = hfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: HfsFileEntriesIterator =
            hfs_file_entry.sub_file_entries();

        let result: Option<Result<HfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<HfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(12).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_has_data_fork_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_data_fork();
        assert_eq!(result, false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_data_fork();
        assert_eq!(result, true);

        Ok(())
    }

    #[test]
    fn test_has_resource_fork_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_resource_fork();
        assert_eq!(result, false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_resource_fork();
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_directory_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_root_directory_with_hfs() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let path: Path = Path::from("/");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), true);

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), false);

        Ok(())
    }

    // TODO: add tests for read_attributes
    // TODO: add tests for read_indirect_node
    // TODO: add tests for read_sub_directory_entries

    // Tests with HFS+.

    #[test]
    fn test_get_access_time_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_access_time(),
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3868686544
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_backup_time_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_backup_time(), &DateTime::NotSet);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_change_time(),
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3868686545
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_creation_time(),
            &DateTime::HfsTime(HfsTime {
                timestamp: 3868686544
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_file_mode(), Some(&0o100644));

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_group_identifier(), Some(&20));

        Ok(())
    }

    // TODO: add tests for get_identifier
    // TODO: add tests for get_link_reference

    #[test]
    fn test_get_modification_time_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            hfs_file_entry.get_modification_time(),
            &DateTime::HfsTime(HfsTime {
                timestamp: 3868686544
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<&HfsString> = hfs_file_entry.get_name();
        assert_eq!(name, Some(HfsString::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_number_of_links(), 1);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_owner_identifier(), Some(&501));

        Ok(())
    }

    #[test]
    fn test_get_size_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.get_size(), 9);

        Ok(())
    }

    // TODO: add tests for get_block_stream

    #[test]
    fn test_get_symbolic_link_target_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let symbolic_link_target: Option<&ByteString> =
            hfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(symbolic_link_target, None);

        let path: Path = Path::from("/file_symboliclink1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let symbolic_link_target: Option<&ByteString> =
            hfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(
            symbolic_link_target,
            Some(ByteString::from("/Volumes/hfsplus_test/testdir1/testfile1")).as_ref()
        );
        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = hfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = hfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_data_fork

    #[test]
    fn test_get_data_fork_descriptor_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_data_fork_descriptor();
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_data_fork_descriptor();
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_resource_fork

    #[test]
    fn test_get_resource_fork_descriptor_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_resource_fork_descriptor();
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<&HfsForkDescriptor> = hfs_file_entry.get_resource_fork_descriptor();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/xattr1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_attributes: usize = hfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_attributes, 1);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/xattr1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let extended_attribute: HfsExtendedAttribute =
            hfs_file_entry.get_extended_attribute_by_index(0)?;
        let expected_name: HfsString = HfsString::Utf16String(Utf16String {
            elements: vec![109, 121, 120, 97, 116, 116, 114, 49],
        });
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let result: Result<HfsExtendedAttribute, ErrorTrace> =
            hfs_file_entry.get_extended_attribute_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/xattr1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::from("myxattr1");
        let extended_attribute: HfsExtendedAttribute = hfs_file_entry
            .get_extended_attribute_by_name(&name)?
            .unwrap();
        let expected_name: HfsString = HfsString::Utf16String(Utf16String {
            elements: vec![109, 121, 120, 97, 116, 116, 114, 49],
        });
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<HfsExtendedAttribute> =
            hfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1/xattr1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut extended_attributes_iterator: HfsExtendedAttributesIterator =
            hfs_file_entry.extended_attributes();

        let result: Option<Result<HfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<HfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = hfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 13);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = hfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: HfsFileEntry = hfs_file_entry.get_sub_file_entry_by_index(6)?;

        let name: Option<&HfsString> = sub_file_entry.get_name();
        assert_eq!(name, Some(HfsString::from("large_xattr")).as_ref());

        let result: Result<HfsFileEntry, ErrorTrace> =
            hfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_name_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::Utf16String(Utf16String::from("large_xattr"));
        let result: Option<HfsFileEntry> = hfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_some());

        let name: PathComponent = PathComponent::Utf16String(Utf16String::from("bogus"));
        let result: Option<HfsFileEntry> = hfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut hfs_file_entry: HfsFileEntry =
            hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: HfsFileEntriesIterator =
            hfs_file_entry.sub_file_entries();

        let result: Option<Result<HfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<HfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(12).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_has_data_fork_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_data_fork();
        assert_eq!(result, false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_data_fork();
        assert_eq!(result, true);

        Ok(())
    }

    #[test]
    fn test_has_resource_fork_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_resource_fork();
        assert_eq!(result, false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: bool = hfs_file_entry.has_resource_fork();
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_directory_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_root_directory_with_hfsplus() -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let path: Path = Path::from("/");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), true);

        let path: Path = Path::from("/testdir1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), false);

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(hfs_file_entry.is_root_directory(), false);

        Ok(())
    }

    // TODO: add tests for read_attributes
    // TODO: add tests for read_indirect_node
    // TODO: add tests for read_sub_directory_entries
}
