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
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::indexed_hash_map::IndexedHashMap;
use crate::path_component::PathComponent;
use crate::traits::{ExtendedAttributeIterator, FileEntryIterator};

use super::attribute::XfsAttribute;
use super::attributes_table::XfsAttributesTable;
use super::attributes_tree::XfsAttributesTree;
use super::block_reader::XfsBlockReader;
use super::block_stream::XfsBlockStream;
use super::constants::*;
use super::directory_entry::XfsDirectoryEntry;
use super::directory_list::XfsDirectoryList;
use super::directory_table::XfsDirectoryTable;
use super::directory_tree::XfsDirectoryTree;
use super::extended_attribute::XfsExtendedAttribute;
use super::extended_attributes::XfsExtendedAttributesIterator;
use super::extent_list::XfsExtentList;
use super::extent_tree::XfsExtentTree;
use super::file_entries::XfsFileEntriesIterator;
use super::inode::XfsInode;
use super::inode_tree::XfsInodeTree;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) file entry.
pub struct XfsFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Inode tree.
    inode_tree: Arc<XfsInodeTree>,

    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Value to indicate version 2 directories are used.
    has_directory_v2: bool,

    /// Value to indicate directory entries contain a file type value.
    has_file_type: bool,

    /// The (absolute) inode number.
    pub(super) inode_number: u64,

    /// The inode.
    inode: XfsInode,

    /// The name.
    name: Option<ByteString>,

    /// Sub directory entries.
    sub_directory_entries: IndexedHashMap<ByteString, XfsDirectoryEntry>,

    /// Value to indicate the sub directory entries were read.
    sub_directory_entries_read: bool,

    /// Symbolic link target.
    symbolic_link_target: Option<ByteString>,

    /// Attributes.
    attributes: IndexedHashMap<ByteString, XfsAttribute>,

    /// Value to indicate the attributes were read.
    attributes_read: bool,
}

impl XfsFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        inode_tree: &Arc<XfsInodeTree>,
        character_encoding: &CharacterEncoding,
        has_directory_v2: bool,
        has_file_type: bool,
        inode_number: u64,
        inode: XfsInode,
        name: Option<ByteString>,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            inode_tree: inode_tree.clone(),
            character_encoding: character_encoding.clone(),
            has_directory_v2,
            has_file_type,
            inode_number,
            inode,
            name,
            sub_directory_entries: IndexedHashMap::new(),
            sub_directory_entries_read: false,
            symbolic_link_target: None,
            attributes: IndexedHashMap::new(),
            attributes_read: false,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> &DateTime {
        &self.inode.access_time
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> &DateTime {
        &self.inode.change_time
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.inode.creation_time.as_ref()
    }

    /// Retrieves the device identifier.
    pub fn get_device_identifier(&self) -> Option<&u32> {
        self.inode.device_identifier.as_ref()
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> u16 {
        self.inode.file_mode
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> u32 {
        self.inode.group_identifier
    }

    /// Retrieves the inode number.
    pub fn get_inode_number(&self) -> u64 {
        self.inode_number
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> &DateTime {
        &self.inode.modification_time
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&ByteString> {
        self.name.as_ref()
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
        match self.inode.file_mode & 0xf000 {
            XFS_FILE_MODE_TYPE_REGULAR_FILE | XFS_FILE_MODE_TYPE_SYMBOLIC_LINK => {
                self.inode.data_size
            }
            _ => 0,
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> Result<Option<&ByteString>, ErrorTrace> {
        if self.symbolic_link_target.is_none() && self.is_symbolic_link() {
            if self.inode.data_size > 1024 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid symbolic link target data size: {} value out of bounds",
                    self.inode.data_size,
                )));
            }
            let mut byte_string: ByteString =
                ByteString::new_with_encoding(&self.character_encoding);

            match self.inode.fork_type {
                XFS_FORK_TYPE_INLINE_DATA => byte_string.read_data(self.inode.data_fork.as_slice()),
                XFS_FORK_TYPE_EXTENTS | XFS_FORK_TYPE_BTREE => {
                    let mut block_stream: XfsBlockStream =
                        XfsBlockStream::new(XfsBlockReader::new(
                            &self.data_stream,
                            self.inode_tree.block_size,
                            &self.inode.extents,
                            self.inode.data_size,
                        ));
                    let mut data: Vec<u8> = vec![0; self.inode.data_size as usize];

                    match block_stream.read_exact(&mut data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read symbolic link target data from block stream"
                            );
                            return Err(error);
                        }
                    }
                    byte_string.read_data(data.as_slice())
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported data fork type",
                    ));
                }
            }
            self.symbolic_link_target = Some(byte_string);
        }
        Ok(self.symbolic_link_target.as_ref())
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&mut self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.inode.file_mode & 0xf000 != XFS_FILE_MODE_TYPE_REGULAR_FILE {
            return Ok(None);
        }
        match self.inode.fork_type {
            XFS_FORK_TYPE_INLINE_DATA => Ok(Some(Arc::new(RwLock::new(FakeDataStream::new(
                &self.inode.data_fork,
                self.inode.data_size,
            ))))),
            XFS_FORK_TYPE_EXTENTS | XFS_FORK_TYPE_BTREE => Ok(Some(Arc::new(RwLock::new(
                XfsBlockStream::new(XfsBlockReader::new(
                    &self.data_stream,
                    self.inode_tree.block_size,
                    &self.inode.extents,
                    self.inode.data_size,
                )),
            )))),
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported data fork type",
                ));
            }
        }
    }

    /// Retrieves the data stream of an extended attribute.
    fn get_extended_attribute_data_stream(
        &self,
        attribute_record: &XfsAttribute,
    ) -> Result<DataStreamReference, ErrorTrace> {
        match attribute_record {
            XfsAttribute::InlineData(inline_data) => Ok(Arc::new(RwLock::new(
                FakeDataStream::new(&inline_data, inline_data.len() as u64),
            ))),
        }
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_name(
        &mut self,
        extended_attribute_name: &PathComponent,
    ) -> Result<Option<XfsExtendedAttribute>, ErrorTrace> {
        if !self.attributes_read {
            match self.read_attributes() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                    return Err(error);
                }
            }
        }
        let lookup_name: ByteString =
            match extended_attribute_name.to_byte_string(&self.character_encoding) {
                Ok(byte_string) => byte_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to byte string"
                    );
                    return Err(error);
                }
            };
        match self.attributes.get_key_value_by_key(&lookup_name) {
            Some((name, attribute)) => {
                let data_stream: DataStreamReference =
                    match self.get_extended_attribute_data_stream(attribute) {
                        Ok(data_stream) => data_stream,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                Ok(Some(XfsExtendedAttribute::new(name, data_stream)))
            }
            None => Ok(None),
        }
    }

    /// Retrieves an extended attributes iterator.
    pub fn extended_attributes(&mut self) -> XfsExtendedAttributesIterator<'_> {
        XfsExtendedAttributesIterator::new(self)
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> XfsFileEntriesIterator<'_> {
        XfsFileEntriesIterator::new(self)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &PathComponent,
    ) -> Result<Option<Self>, ErrorTrace> {
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
        let lookup_name: ByteString =
            match sub_file_entry_name.to_byte_string(&self.character_encoding) {
                Ok(byte_string) => byte_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to byte string"
                    );
                    return Err(error);
                }
            };
        match self
            .sub_directory_entries
            .get_key_value_by_key(&lookup_name)
        {
            Some((name, directory_entry)) => {
                match self
                    .inode_tree
                    .get_inode_by_identifier(&self.data_stream, directory_entry.inode_number)
                {
                    Ok(Some(inode)) => Ok(Some(Self::new(
                        &self.data_stream,
                        &self.inode_tree,
                        &self.character_encoding,
                        self.has_directory_v2,
                        self.has_file_type,
                        directory_entry.inode_number,
                        inode,
                        Some(name.clone()),
                    ))),
                    Ok(None) => Ok(None),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve inode: {}", directory_entry.inode_number)
                        );
                        Err(error)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        self.inode.file_mode & 0xf000 == XFS_FILE_MODE_TYPE_DIRECTORY
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        self.inode_number == self.inode_tree.root_directory_inode_number
    }

    /// Determines if the file entry is a symbolic link.
    fn is_symbolic_link(&self) -> bool {
        self.inode.file_mode & 0xf000 == XFS_FILE_MODE_TYPE_SYMBOLIC_LINK
    }

    /// Reads the attributes.
    fn read_attributes(&mut self) -> Result<(), ErrorTrace> {
        match self.inode.attributes_fork_type {
            XFS_FORK_TYPE_INLINE_DATA => {
                let mut attributes_table: XfsAttributesTable =
                    XfsAttributesTable::new(&self.character_encoding);

                match attributes_table.read_data(&self.inode.attributes_fork, &mut self.attributes)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attributes table"
                        );
                        return Err(error);
                    }
                }
            }
            XFS_FORK_TYPE_EXTENTS | XFS_FORK_TYPE_BTREE => {
                let mut extents: Vec<XfsPackedExtent> = Vec::new();

                if self.inode.attributes_fork_type == XFS_FORK_TYPE_EXTENTS {
                    let extent_list: XfsExtentList = XfsExtentList::new();

                    match extent_list.read_data(
                        self.inode.number_of_attributes_extents as u64,
                        &self.inode.attributes_fork,
                        &mut extents,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read extent list"
                            );
                            return Err(error);
                        }
                    }
                } else {
                    let extent_tree: XfsExtentTree = XfsExtentTree::new(
                        self.inode_tree.format_version,
                        self.inode_tree.allocation_group_size,
                        self.inode_tree.number_of_relative_block_number_bits,
                        self.inode_tree.block_size,
                    );
                    match extent_tree.read_extents(
                        &self.data_stream,
                        &self.inode.attributes_fork,
                        &mut extents,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read extent tree"
                            );
                            return Err(error);
                        }
                    }
                }
                let attributes_tree: XfsAttributesTree = XfsAttributesTree::new(
                    &self.character_encoding,
                    self.inode_tree.format_version,
                    self.inode_tree.allocation_group_size,
                    self.inode_tree.number_of_relative_block_number_bits,
                    self.inode_tree.block_size,
                );
                match attributes_tree.read_attributes(
                    &self.data_stream,
                    &extents,
                    &mut self.attributes,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attributes tree"
                        );
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported attributes fork type",
                ));
            }
        }
        self.attributes_read = true;

        Ok(())
    }

    /// Reads the sub directory entries.
    fn read_sub_directory_entries(&mut self) -> Result<(), ErrorTrace> {
        match self.inode.fork_type {
            XFS_FORK_TYPE_INLINE_DATA => {
                let mut directory_table: XfsDirectoryTable =
                    XfsDirectoryTable::new(&self.character_encoding);

                match directory_table.read_data(
                    self.has_directory_v2,
                    self.has_file_type,
                    &self.inode.data_fork,
                    &mut self.sub_directory_entries,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read directory table"
                        );
                        return Err(error);
                    }
                }
            }
            XFS_FORK_TYPE_EXTENTS | XFS_FORK_TYPE_BTREE => {
                if self.has_directory_v2 {
                    let directory_list: XfsDirectoryList = XfsDirectoryList::new(
                        &self.character_encoding,
                        self.inode_tree.allocation_group_size,
                        self.inode_tree.number_of_relative_block_number_bits,
                        self.inode_tree.block_size,
                        self.inode_tree.directory_block_size,
                    );
                    match directory_list.read_entries(
                        self.has_file_type,
                        &self.data_stream,
                        self.inode.data_size,
                        &self.inode.extents,
                        &mut self.sub_directory_entries,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read directory list"
                            );
                            return Err(error);
                        }
                    }
                } else {
                    let directory_tree: XfsDirectoryTree = XfsDirectoryTree::new(
                        &self.character_encoding,
                        self.inode_tree.allocation_group_size,
                        self.inode_tree.number_of_relative_block_number_bits,
                        self.inode_tree.block_size,
                    );
                    match directory_tree.read_entries(
                        &self.data_stream,
                        &self.inode.extents,
                        &mut self.sub_directory_entries,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read directory tree"
                            );
                            return Err(error);
                        }
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported data fork type",
                ));
            }
        }
        self.sub_directory_entries_read = true;

        Ok(())
    }
}

impl ExtendedAttributeIterator for XfsFileEntry {
    type ExtendedAttributeItem = XfsExtendedAttribute;

    /// Retrieves the number of extended attributes.
    fn get_number_of_extended_attributes(&mut self) -> Result<usize, ErrorTrace> {
        if !self.attributes_read {
            match self.read_attributes() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                    return Err(error);
                }
            }
        }
        Ok(self.attributes.len())
    }

    /// Retrieves a specific extended attribute.
    fn get_extended_attribute_by_index(
        &mut self,
        extended_attribute_index: usize,
    ) -> Result<XfsExtendedAttribute, ErrorTrace> {
        if !self.attributes_read {
            match self.read_attributes() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                    return Err(error);
                }
            }
        }
        match self
            .attributes
            .get_key_value_by_index(extended_attribute_index)
        {
            Some((name, attribute)) => {
                let data_stream: DataStreamReference =
                    match self.get_extended_attribute_data_stream(attribute) {
                        Ok(data_stream) => data_stream,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                Ok(XfsExtendedAttribute::new(name, data_stream))
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing extended attribute: {}",
                extended_attribute_index
            ))),
        }
    }
}

impl FileEntryIterator for XfsFileEntry {
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
                match self
                    .inode_tree
                    .get_inode_by_identifier(&self.data_stream, directory_entry.inode_number)
                {
                    Ok(Some(inode)) => Ok(Self::new(
                        &self.data_stream,
                        &self.inode_tree,
                        &self.character_encoding,
                        self.has_directory_v2,
                        self.has_file_type,
                        directory_entry.inode_number,
                        inode,
                        Some(name.clone()),
                    )),
                    Ok(None) => Err(keramics_core::error_trace_new!(format!(
                        "Missing inode: {}",
                        directory_entry.inode_number
                    ))),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve inode: {}", directory_entry.inode_number)
                        );
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
    use keramics_datetime::PosixTime64Ns;

    use crate::path::Path;
    use crate::xfs::file_system::XfsFileSystem;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<XfsFileSystem, ErrorTrace> {
        let mut file_system: XfsFileSystem = XfsFileSystem::new();

        let path_string: String = get_test_data_path("xfs/xfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            xfs_file_entry.get_access_time(),
            &DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 180859596
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_change_time() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            xfs_file_entry.get_change_time(),
            &DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 182386739
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            xfs_file_entry.get_creation_time(),
            Some(DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 180859596
            }))
            .as_ref()
        );
        Ok(())
    }

    #[test]
    fn test_get_device_identifier() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let device_identifier: Option<&u32> = xfs_file_entry.get_device_identifier();
        assert_eq!(device_identifier, None);

        let path: Path = Path::from("/testdir1/blockdev1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let device_identifier: Option<&u32> = xfs_file_entry.get_device_identifier();
        assert_eq!(device_identifier, Some(0x39006000).as_ref());

        let path: Path = Path::from("/testdir1/chardev1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let device_identifier: Option<&u32> = xfs_file_entry.get_device_identifier();
        assert_eq!(device_identifier, Some(0x44003400).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_file_mode() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_file_mode(), 0o100644);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_group_identifier(), 1000);

        Ok(())
    }

    #[test]
    fn test_get_inode_number() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_inode_number(), 16133);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            xfs_file_entry.get_modification_time(),
            &DateTime::PosixTime64Ns(PosixTime64Ns {
                timestamp: 1787837648,
                fraction: 181438932
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_links() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_number_of_links(), 2);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_owner_identifier(), 1000);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.get_size(), 9);

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = xfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = xfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_attributes: usize = xfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_attributes, 1);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let extended_attribute: XfsExtendedAttribute =
            xfs_file_entry.get_extended_attribute_by_index(0)?;
        let expected_name: ByteString = ByteString {
            encoding: CharacterEncoding::Utf8,
            elements: b"secure.selinux".to_vec(),
        };
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let result: Result<XfsExtendedAttribute, ErrorTrace> =
            xfs_file_entry.get_extended_attribute_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::from("secure.selinux");
        let extended_attribute: XfsExtendedAttribute = xfs_file_entry
            .get_extended_attribute_by_name(&name)?
            .unwrap();
        let expected_name: ByteString = ByteString {
            encoding: CharacterEncoding::Utf8,
            elements: b"secure.selinux".to_vec(),
        };
        assert_eq!(extended_attribute.get_name(), &expected_name);

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<XfsExtendedAttribute> =
            xfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut extended_attributes_iterator: XfsExtendedAttributesIterator =
            xfs_file_entry.extended_attributes();

        let result: Option<Result<XfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<XfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = xfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 10);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = xfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: XfsFileEntry = xfs_file_entry.get_sub_file_entry_by_index(1)?;

        let name: Option<&ByteString> = sub_file_entry.get_name();
        assert_eq!(name, Some(ByteString::from("TestFile2")).as_ref());

        let result: Result<XfsFileEntry, ErrorTrace> =
            xfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_name() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::ByteString(ByteString::from("TestFile2"));
        let result: Option<XfsFileEntry> = xfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_some());

        let name: PathComponent = PathComponent::ByteString(ByteString::from("bogus"));
        let result: Option<XfsFileEntry> = xfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: XfsFileEntriesIterator =
            xfs_file_entry.sub_file_entries();

        let result: Option<Result<XfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<XfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(9).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_is_directory() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_root_directory() -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_root_directory(), true);

        let path: Path = Path::from("/testdir1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_root_directory(), false);

        let path: Path = Path::from("/testdir1/testfile1");
        let xfs_file_entry: XfsFileEntry = xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(xfs_file_entry.is_root_directory(), false);

        Ok(())
    }

    // TODO: add tests for read_attributes
    // TODO: add tests for read_sub_directory_entries
}
