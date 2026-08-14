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

use keramics_core::{DataStreamReference, ErrorTrace, FakeDataStream};
use keramics_datetime::DateTime;
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::decmpfs::{
    DecmpfsBlockReader, DecmpfsCompressionMethod, DecmpfsDataStream, DecmpfsHeader,
};
use crate::indexed_hash_map::IndexedHashMap;
use crate::path_component::PathComponent;
use crate::traits::FileEntryIterator;

use super::attribute_record::ApfsAttributeRecord;
use super::block_reader::ApfsBlockReader;
use super::block_stream::ApfsBlockStream;
use super::constants::*;
use super::directory_entry::ApfsDirectoryEntry;
use super::enums::ApfsForkType;
use super::extended_attribute::ApfsExtendedAttribute;
use super::extended_attributes::ApfsExtendedAttributesIterator;
use super::extent::ApfsExtent;
use super::file_entries::ApfsFileEntriesIterator;
use super::file_system_tree::ApfsFileSystemTree;
use super::fork::ApfsFork;
use super::inode::ApfsInode;
use super::object_map_tree::ApfsObjectMapTree;

/// Apple File System (APFS) file entry.
pub struct ApfsFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Object map B-tree.
    object_map_tree: Arc<ApfsObjectMapTree>,

    /// File system B-tree.
    file_system_tree: Arc<ApfsFileSystemTree>,

    /// Identifier.
    pub(super) identifier: u64,

    /// Transaction identifier.
    transaction_identifier: u64,

    /// Inode.
    inode: ApfsInode,

    /// Directory entry.
    directory_entry: Option<ApfsDirectoryEntry>,

    /// Compressed data header.
    compressed_data_header: Option<DecmpfsHeader>,

    /// Extents.
    extents: Vec<ApfsExtent>,

    /// Sub directory entries.
    sub_directory_entries: IndexedHashMap<ByteString, ApfsDirectoryEntry>,

    /// Value to indicate the sub directory entries were read.
    read_sub_directory_entries: bool,

    /// Symbolic link target.
    symbolic_link_target: Option<ByteString>,

    /// Attributes.
    attributes: IndexedHashMap<ByteString, ApfsAttributeRecord>,
}

impl ApfsFileEntry {
    /// Creates a file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        block_size: u32,
        object_map_tree: &Arc<ApfsObjectMapTree>,
        file_system_tree: &Arc<ApfsFileSystemTree>,
        identifier: u64,
        transaction_identifier: u64,
        inode: ApfsInode,
        directory_entry: Option<ApfsDirectoryEntry>,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            object_map_tree: object_map_tree.clone(),
            file_system_tree: file_system_tree.clone(),
            identifier,
            transaction_identifier,
            inode,
            directory_entry,
            compressed_data_header: None,
            extents: Vec::new(),
            sub_directory_entries: IndexedHashMap::new(),
            read_sub_directory_entries: false,
            symbolic_link_target: None,
            attributes: IndexedHashMap::new(),
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> &DateTime {
        &self.inode.access_time
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> &DateTime {
        &self.inode.creation_time
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> &DateTime {
        &self.inode.change_time
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> u16 {
        self.inode.file_mode
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> u32 {
        self.inode.group_identifier
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> u64 {
        self.identifier
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> &DateTime {
        &self.inode.modification_time
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&ByteString> {
        match &self.directory_entry {
            Some(directory_entry) => directory_entry.name.as_ref(),
            None => self.inode.name.as_ref(),
        }
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> u32 {
        self.inode.number_of_links
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> u32 {
        self.inode.owner_identifier
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self.compressed_data_header.as_ref() {
            Some(compressed_data_header) => compressed_data_header.uncompressed_data_size,
            None => match self.inode.data_stream_descriptor.as_ref() {
                Some(data_stream_descriptor) => data_stream_descriptor.size,
                None => 0,
            },
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> Result<Option<&ByteString>, ErrorTrace> {
        if self.symbolic_link_target.is_none() && self.is_symbolic_link() {
            let lookup_name: ByteString = ByteString::from("com.apple.fs.symlink");

            match self.attributes.get_value_by_key(&lookup_name) {
                Some(attribute_record) => {
                    if attribute_record.flags & 0x0002 != 0 {
                        self.symbolic_link_target =
                            Some(ByteString::from(attribute_record.inline_data.as_slice()));
                    } else {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported symlink attribute record type"
                        ));
                    }
                }
                None => {}
            }
        }
        Ok(self.symbolic_link_target.as_ref())
    }

    /// Retrieves the block stream.
    fn get_block_stream(&self) -> Result<ApfsBlockStream, ErrorTrace> {
        let size: u64 = match self.inode.data_stream_descriptor.as_ref() {
            Some(data_stream_descriptor) => data_stream_descriptor.size,
            None => 0,
        };
        let mut block_reader: ApfsBlockReader =
            ApfsBlockReader::new(&self.data_stream, self.block_size, size);

        match block_reader.open(&self.extents) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open block reader");
                return Err(error);
            }
        }
        Ok(ApfsBlockStream::new(block_reader))
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
                let data_stream: DataStreamReference =
                    match compressed_data_header.compression_method {
                        4 | 8 | 12 => match self.get_block_stream() {
                            Ok(block_stream) => Arc::new(RwLock::new(block_stream)),
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve block stream"
                                );
                                return Err(error);
                            }
                        },
                        _ => {
                            let lookup_name: ByteString = ByteString::from("com.apple.decmpfs");

                            match self.attributes.get_value_by_key(&lookup_name) {
                                Some(attribute_record) => {
                                    if attribute_record.flags & 0x0002 != 0 {
                                        let data_stream: FakeDataStream = FakeDataStream::new(
                                            &attribute_record.inline_data,
                                            attribute_record.data_size as u64,
                                        );
                                        Arc::new(RwLock::new(data_stream))
                                    } else {
                                        return Err(keramics_core::error_trace_new!(
                                            "Unsupported decmpfs attribute record flags"
                                        ));
                                    }
                                }
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
            None => match self.get_block_stream() {
                Ok(block_stream) => Ok(Some(Arc::new(RwLock::new(block_stream)))),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve block stream");
                    Err(error)
                }
            },
        }
    }

    /// Retrieves the data fork.
    pub fn get_data_fork(&mut self) -> Result<Option<ApfsFork>, ErrorTrace> {
        if !self.has_data_fork() {
            return Ok(None);
        }
        match self.get_block_stream() {
            Ok(block_stream) => Ok(Some(ApfsFork::new(
                ApfsForkType::Data,
                Arc::new(RwLock::new(block_stream)),
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve block stream");
                Err(error)
            }
        }
    }

    /// Retrieves the resource fork.
    pub fn get_resource_fork(&mut self) -> Result<Option<ApfsFork>, ErrorTrace> {
        todo!();
    }

    /// Determines if the file entry has a data fork.
    pub fn has_data_fork(&self) -> bool {
        self.inode.file_mode & 0xf000 == APFS_FILE_MODE_TYPE_REGULAR_FILE
    }

    /// Determines if the file entry has a resource fork.
    pub fn has_resource_fork(&self) -> bool {
        let lookup_name: ByteString = ByteString::from("com.apple.ResourceFork");

        self.attributes.contains_key(&lookup_name)
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        self.inode.file_mode & 0xf000 == APFS_FILE_MODE_TYPE_DIRECTORY
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        self.identifier == APFS_ROOT_DIRECTORY_IDENTIFIER
    }

    /// Determines if the file entry is a symbolic link.
    fn is_symbolic_link(&self) -> bool {
        self.inode.file_mode & 0xf000 == APFS_FILE_MODE_TYPE_SYMBOLIC_LINK
    }

    /// Retrieves the data stream of an extended attribute.
    fn get_extended_attribute_data_stream(
        &self,
        attribute_record: &ApfsAttributeRecord,
    ) -> Result<DataStreamReference, ErrorTrace> {
        if attribute_record.flags & 0x0001 != 0 {
            // TODO: implement
            todo!();
        } else if attribute_record.flags & 0x0002 != 0 {
            let data_stream: FakeDataStream = FakeDataStream::new(
                &attribute_record.inline_data,
                attribute_record.data_size as u64,
            );
            Ok(Arc::new(RwLock::new(data_stream)))
        } else {
            Err(keramics_core::error_trace_new!(
                "Unsupported attribute record flags"
            ))
        }
    }

    /// Retrieves the number of extended attributes.
    pub fn get_number_of_extended_attributes(&self) -> Result<usize, ErrorTrace> {
        Ok(self.attributes.len())
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_index(
        &self,
        extended_attribute_index: usize,
    ) -> Result<ApfsExtendedAttribute, ErrorTrace> {
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
                Ok(ApfsExtendedAttribute::new(name, data_stream))
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing extended attribute: {}",
                extended_attribute_index
            ))),
        }
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_name(
        &self,
        extended_attribute_name: &PathComponent,
    ) -> Result<Option<ApfsExtendedAttribute>, ErrorTrace> {
        let lookup_name: ByteString =
            match extended_attribute_name.to_byte_string(&CharacterEncoding::Utf8) {
                Ok(byte_string) => byte_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UTF-8 string"
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
                Ok(Some(ApfsExtendedAttribute::new(name, data_stream)))
            }
            None => Ok(None),
        }
    }

    /// Retrieves an extended attributes iterator.
    pub fn extended_attributes(&mut self) -> ApfsExtendedAttributesIterator<'_> {
        ApfsExtendedAttributesIterator::new(self)
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> ApfsFileEntriesIterator<'_> {
        ApfsFileEntriesIterator::new(self)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &PathComponent,
    ) -> Result<Option<ApfsFileEntry>, ErrorTrace> {
        let directory_entry: ApfsDirectoryEntry =
            match self.file_system_tree.get_directory_entry_by_name(
                &self.data_stream,
                &self.object_map_tree,
                self.identifier,
                sub_file_entry_name,
                self.transaction_identifier,
            ) {
                Ok(Some(directory_entry)) => directory_entry,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve directory entry from file system tree"
                    );
                    return Err(error);
                }
            };
        let identifier: u64 = directory_entry.get_identifier();

        let inode: ApfsInode = match self.file_system_tree.get_inode_by_identifier(
            &self.data_stream,
            &self.object_map_tree,
            identifier,
            self.transaction_identifier,
        ) {
            Ok(Some(inode)) => inode,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve inode: {}", identifier)
                );
                return Err(error);
            }
        };
        let mut file_entry: ApfsFileEntry = ApfsFileEntry::new(
            &self.data_stream,
            self.block_size,
            &self.object_map_tree,
            &self.file_system_tree,
            identifier,
            self.transaction_identifier,
            inode,
            Some(directory_entry),
        );
        match file_entry.read_attributes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        match file_entry.read_extents() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extents");
                return Err(error);
            }
        }
        Ok(Some(file_entry))
    }

    /// Reads the attributes.
    pub(super) fn read_attributes(&mut self) -> Result<(), ErrorTrace> {
        match self.file_system_tree.get_attributes_by_identifier(
            &self.data_stream,
            &self.object_map_tree,
            self.identifier,
            self.transaction_identifier,
            &mut self.attributes,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve attributes from file system tree"
                );
                return Err(error);
            }
        }
        let lookup_name: ByteString = ByteString::from("com.apple.decmpfs");

        match self.attributes.get_value_by_key(&lookup_name) {
            Some(attribute_record) => {
                if attribute_record.flags & 0x0002 != 0 {
                    let mut compressed_data_header: DecmpfsHeader = DecmpfsHeader::new();

                    keramics_core::debug_trace_data_and_structure!(
                        "DecmpfsHeader",
                        0,
                        &attribute_record.inline_data,
                        attribute_record.data_size as usize,
                        DecmpfsHeader::debug_read_data(&attribute_record.inline_data)
                    );
                    match compressed_data_header.read_data(&attribute_record.inline_data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read decmpfs header"
                            );
                            return Err(error);
                        }
                    }
                    self.compressed_data_header = Some(compressed_data_header);
                } else {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported decmpfs attribute record flags"
                    ));
                }
            }
            None => {}
        }
        Ok(())
    }

    /// Reads the extents.
    pub(super) fn read_extents(&mut self) -> Result<(), ErrorTrace> {
        match self.file_system_tree.get_extents_by_identifier(
            &self.data_stream,
            &self.object_map_tree,
            self.inode.data_stream_identifier,
            self.transaction_identifier,
            &mut self.extents,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve data stream: {} extents from file system tree",
                        self.inode.data_stream_identifier
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads the sub directory entries.
    fn read_sub_directory_entries(&mut self) -> Result<(), ErrorTrace> {
        match self.file_system_tree.get_directory_entries_by_identifier(
            &self.data_stream,
            &self.object_map_tree,
            self.identifier,
            self.transaction_identifier,
            &mut self.sub_directory_entries,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve sub directory entries from file system tree"
                );
                return Err(error);
            }
        }
        self.read_sub_directory_entries = true;

        Ok(())
    }
}

impl FileEntryIterator for ApfsFileEntry {
    /// Retrieves the number of sub file entries.
    fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        if self.is_directory() && !self.read_sub_directory_entries {
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
    ) -> Result<ApfsFileEntry, ErrorTrace> {
        if self.is_directory() && !self.read_sub_directory_entries {
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
                let identifier: u64 = directory_entry.get_identifier();

                let inode: ApfsInode = match self.file_system_tree.get_inode_by_identifier(
                    &self.data_stream,
                    &self.object_map_tree,
                    identifier,
                    self.transaction_identifier,
                ) {
                    Ok(Some(inode)) => inode,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing inode: {} for directory entry: {}",
                            identifier, sub_file_entry_index
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve inode: {} of directory entry: {}",
                                identifier, sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                };
                let mut sub_directory_entry: ApfsDirectoryEntry = directory_entry.clone();
                sub_directory_entry.name = Some(name.clone());

                let mut file_entry: ApfsFileEntry = ApfsFileEntry::new(
                    &self.data_stream,
                    self.block_size,
                    &self.object_map_tree,
                    &self.file_system_tree,
                    identifier,
                    self.transaction_identifier,
                    inode,
                    Some(sub_directory_entry),
                );
                match file_entry.read_attributes() {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                        return Err(error);
                    }
                }
                match file_entry.read_extents() {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read extents");
                        return Err(error);
                    }
                }
                Ok(file_entry)
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
    use keramics_datetime::ApfsTime;

    use crate::apfs::{ApfsContainer, ApfsFileSystem, ApfsVolume};
    use crate::path::Path;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<ApfsFileSystem, ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        let volume: ApfsVolume = container.get_volume_by_index(0)?;
        volume.get_file_system()
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            apfs_file_entry.get_access_time(),
            &DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254251713
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            apfs_file_entry.get_creation_time(),
            &DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254516511
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_change_time() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            apfs_file_entry.get_change_time(),
            &DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765262832871
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_file_mode() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(apfs_file_entry.get_file_mode(), 0o100644);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(apfs_file_entry.get_group_identifier(), 99);

        Ok(())
    }

    // TODO add tests for get_identifier

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            apfs_file_entry.get_modification_time(),
            &DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254251713
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<&ByteString> = apfs_file_entry.get_name();
        assert_eq!(name, Some(ByteString::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_number_of_links() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(apfs_file_entry.get_number_of_links(), 2);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(apfs_file_entry.get_owner_identifier(), 99);

        Ok(())
    }

    // TODO: add tests for get_size
    // TODO: add tests for get_block_stream
    // TODO: add tests for get_symbolic_link_target
    // TODO: add tests for get_data_stream
    // TODO: add tests for get_data_fork
    // TODO: add tests for get_resource_fork
    // TODO: add tests for has_data_fork
    // TODO: add tests for has_resource_fork
    // TODO: add tests for is_directory
    // TODO: add tests for is_root_directory
    // TODO: add tests for is_symbolic_link

    #[test]
    fn test_get_number_of_extended_attributes() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/xattr1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_attributes: usize = apfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_attributes, 1);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/xattr1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let extended_attribute: ApfsExtendedAttribute =
            apfs_file_entry.get_extended_attribute_by_index(0)?;
        let expected_name: ByteString = ByteString::from("myxattr1");
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let result: Result<ApfsExtendedAttribute, ErrorTrace> =
            apfs_file_entry.get_extended_attribute_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/xattr1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::from("myxattr1");
        let extended_attribute: ApfsExtendedAttribute = apfs_file_entry
            .get_extended_attribute_by_name(&name)?
            .unwrap();
        let expected_name: ByteString = ByteString::from("myxattr1");
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<ApfsExtendedAttribute> =
            apfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/xattr1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut extended_attributes_iterator: ApfsExtendedAttributesIterator =
            apfs_file_entry.extended_attributes();

        let result: Option<Result<ApfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<ApfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = apfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 13);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = apfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: ApfsFileEntry = apfs_file_entry.get_sub_file_entry_by_index(7)?;

        let name: Option<&ByteString> = sub_file_entry.get_name();
        assert_eq!(name, Some(ByteString::from("large_xattr")).as_ref());

        let result: Result<ApfsFileEntry, ErrorTrace> =
            apfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_name() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::ByteString(ByteString::from("large_xattr"));
        let result: Option<ApfsFileEntry> = apfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_some());

        let name: PathComponent = PathComponent::ByteString(ByteString::from("bogus"));
        let result: Option<ApfsFileEntry> = apfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries() -> Result<(), ErrorTrace> {
        let apfs_file_system: ApfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: ApfsFileEntriesIterator =
            apfs_file_entry.sub_file_entries();

        let result: Option<Result<ApfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<ApfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(12).next();
        assert!(result.is_none());

        Ok(())
    }
}
