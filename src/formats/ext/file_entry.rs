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

use std::cmp::max;
use std::collections::BTreeMap;
use std::io;
use std::io::Read;
use std::rc::Rc;

use crate::bytes_to_u16_le;
use crate::datetime::DateTime;
use crate::types::{ByteString, SharedValue};
use crate::vfs::VfsDataStreamReference;

use super::block_stream::ExtBlockStream;
use super::constants::*;
use super::directory_entry::ExtDirectoryEntry;
use super::directory_tree::ExtDirectoryTree;
use super::inline_stream::ExtInlineDataStream;
use super::inode::ExtInode;
use super::inode_table::ExtInodeTable;

/// Extended File System (ext) file entry.
pub struct ExtFileEntry {
    /// The data stream.
    data_stream: VfsDataStreamReference,

    /// Inode table.
    inode_table: Rc<ExtInodeTable>,

    /// The inode number.
    pub inode_number: u32,

    /// The inode.
    inode: ExtInode,

    /// The name.
    name: Option<ByteString>,

    /// Directory tree.
    directory_tree: BTreeMap<ByteString, ExtDirectoryEntry>,

    /// Value to indicate the directory was read.
    read_directory_tree: bool,

    /// Symbolic link target.
    symbolic_link_target: Option<ByteString>,
}

impl ExtFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &VfsDataStreamReference,
        inode_table: &Rc<ExtInodeTable>,
        inode_number: u32,
        inode: ExtInode,
        name: Option<ByteString>,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            inode_table: inode_table.clone(),
            inode_number: inode_number,
            inode: inode,
            name: name,
            directory_tree: BTreeMap::new(),
            read_directory_tree: false,
            symbolic_link_target: None,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        Some(&self.inode.access_time)
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        Some(&self.inode.change_time)
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.inode.creation_time.as_ref()
    }

    /// Retrieves the deletion time.
    pub fn get_deletion_time(&self) -> &DateTime {
        &self.inode.deletion_time
    }

    /// Retrieves the device identifier.
    pub fn get_device_identifier(&mut self) -> io::Result<Option<u16>> {
        if self.inode.file_mode & 0xf000 == EXT_FILE_MODE_TYPE_CHARACTER_DEVICE
            || self.inode.file_mode & 0xf000 == EXT_FILE_MODE_TYPE_BLOCK_DEVICE
        {
            if self.inode.data_size > 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid device identifier data size: {} value out of bounds",
                        self.inode.data_size,
                    ),
                ));
            }
            let device_identifier: u16 = bytes_to_u16_le!(&self.inode.data_reference, 0);
            return Ok(Some(device_identifier));
        }
        Ok(None)
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> u16 {
        self.inode.file_mode
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> u32 {
        self.inode.group_identifier
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        Some(&self.inode.modification_time)
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&ByteString> {
        self.name.as_ref()
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> u16 {
        self.inode.number_of_links
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> u32 {
        self.inode.owner_identifier
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self.inode.file_mode & 0xf000 {
            EXT_FILE_MODE_TYPE_REGULAR_FILE | EXT_FILE_MODE_TYPE_SYMBOLIC_LINK => {
                self.inode.data_size
            }
            _ => 0,
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> io::Result<Option<&ByteString>> {
        if self.symbolic_link_target.is_none()
            && self.inode.file_mode & 0xf000 == EXT_FILE_MODE_TYPE_SYMBOLIC_LINK
        {
            if self.inode.data_size > (self.inode_table.block_size as u64) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid symbolic link target data size: {} value out of bounds",
                        self.inode.data_size,
                    ),
                ));
            }
            // TODO: move read from data_reference into block_stream?
            let byte_string: ByteString = if self.inode.data_size < 60 {
                ByteString::from_bytes(&self.inode.data_reference)
            } else {
                let number_of_blocks: u64 = max(
                    self.inode
                        .data_size
                        .div_ceil(self.inode_table.block_size as u64),
                    self.inode.number_of_blocks,
                );
                let mut block_stream: ExtBlockStream =
                    ExtBlockStream::new(self.inode_table.block_size, self.inode.data_size);
                block_stream.open(
                    &self.data_stream,
                    number_of_blocks,
                    &self.inode.block_ranges,
                )?;

                let mut data: Vec<u8> = vec![0; self.inode.data_size as usize];
                block_stream.read_exact(&mut data)?;

                ByteString::from_bytes(&data)
            };
            self.symbolic_link_target = Some(byte_string);
        }
        Ok(self.symbolic_link_target.as_ref())
    }

    /// Retrieves the number of attributes.
    pub fn get_number_of_attributes(&mut self) -> io::Result<usize> {
        Ok(self.inode.attributes.len())
    }

    // TODO: add get extended_attributes iterator

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        if !self.read_directory_tree {
            self.read_directory_tree()?;
        }
        Ok(self.directory_tree.len())
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<ExtFileEntry> {
        if !self.read_directory_tree {
            self.read_directory_tree()?;
        }
        let (name, directory_entry): (&ByteString, &ExtDirectoryEntry) =
            match self.directory_tree.iter().nth(sub_file_entry_index) {
                Some(key_and_value) => key_and_value,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Missing directory entry: {}", sub_file_entry_index),
                    ));
                }
            };
        let inode: ExtInode = self
            .inode_table
            .get_inode(&self.data_stream, directory_entry.inode_number)?;

        let file_entry: ExtFileEntry = ExtFileEntry::new(
            &self.data_stream,
            &self.inode_table,
            directory_entry.inode_number,
            inode,
            Some(name.clone()),
        );
        Ok(file_entry)
    }

    // TODO: add get_sub_file_entries iterator

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &ByteString,
    ) -> io::Result<Option<ExtFileEntry>> {
        if !self.read_directory_tree {
            self.read_directory_tree()?;
        }
        let (name, directory_entry): (&ByteString, &ExtDirectoryEntry) =
            match self.directory_tree.get_key_value(sub_file_entry_name) {
                Some(key_and_value) => key_and_value,
                None => return Ok(None),
            };
        let inode: ExtInode = self
            .inode_table
            .get_inode(&self.data_stream, directory_entry.inode_number)?;

        let file_entry: ExtFileEntry = ExtFileEntry::new(
            &self.data_stream,
            &self.inode_table,
            directory_entry.inode_number,
            inode,
            Some(name.clone()),
        );
        Ok(Some(file_entry))
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        if self.inode.file_mode & 0xf000 != EXT_FILE_MODE_TYPE_REGULAR_FILE || name.is_some() {
            return Ok(None);
        }
        if self.inode.flags & EXT_INODE_FLAG_INLINE_DATA != 0 {
            let mut inline_stream: ExtInlineDataStream =
                ExtInlineDataStream::new(self.inode.data_size);
            inline_stream.open(&self.inode.data_reference)?;

            return Ok(Some(SharedValue::new(Box::new(inline_stream))));
        }
        let number_of_blocks: u64 = max(
            self.inode
                .data_size
                .div_ceil(self.inode_table.block_size as u64),
            self.inode.number_of_blocks,
        );
        let mut block_stream: ExtBlockStream =
            ExtBlockStream::new(self.inode_table.block_size, self.inode.data_size);
        block_stream.open(
            &self.data_stream,
            number_of_blocks,
            &self.inode.block_ranges,
        )?;

        Ok(Some(SharedValue::new(Box::new(block_stream))))
    }

    /// Reads the directory tree.
    fn read_directory_tree(&mut self) -> io::Result<()> {
        if self.inode.file_mode & 0xf000 == EXT_FILE_MODE_TYPE_DIRECTORY {
            let mut directory_tree: ExtDirectoryTree =
                ExtDirectoryTree::new(self.inode_table.block_size);

            if self.inode.flags & EXT_INODE_FLAG_INLINE_DATA != 0 {
                directory_tree
                    .read_inline_data(&self.inode.data_reference, &mut self.directory_tree)?;
            } else {
                directory_tree.read_block_data(
                    &self.data_stream,
                    &self.inode.block_ranges,
                    &mut self.directory_tree,
                )?;
            }
        }
        self.read_directory_tree = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use crate::datetime::PosixTime32;
    use crate::formats::ext::file_system::ExtFileSystem;
    use crate::formats::ext::path::ExtPath;
    use crate::vfs::{VfsContext, VfsFileSystem, VfsPath};

    fn get_file_system() -> io::Result<ExtFileSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/ext/ext2.raw".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        file_system.open(&vfs_file_system, &vfs_path)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_access_time() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(
            ext_file_entry.get_access_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977482
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_change_time() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(
            ext_file_entry.get_change_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        let device_identifier: Option<u16> = ext_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        // TODO: test with block or character device file entry

        Ok(())
    }

    #[test]
    fn test_get_file_mode() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_file_mode(), 0o100644);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_group_identifier(), 1000);

        Ok(())
    }

    #[test]
    fn test_get_deletion_time() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_deletion_time(), &DateTime::NotSet);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(
            ext_file_entry.get_modification_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_name() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        let name: &ByteString = ext_file_entry.get_name().unwrap();
        assert_eq!(name.to_string(), "testfile1");

        Ok(())
    }

    #[test]
    fn test_get_number_of_links() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_number_of_links(), 2);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_owner_identifier(), 1000);

        Ok(())
    }

    #[test]
    fn test_get_size() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(ext_file_entry.get_size(), 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        let symbolic_link_target: Option<&ByteString> =
            ext_file_entry.get_symbolic_link_target()?;
        assert_eq!(symbolic_link_target, None);

        // TODO: test with symbolic link file entry

        Ok(())
    }

    #[test]
    fn test_get_number_of_attributes() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        let number_of_attributes: usize = ext_file_entry.get_number_of_attributes()?;
        assert_eq!(number_of_attributes, 0);

        // TODO: test with file entry with attributes

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> io::Result<()> {
        let ext_file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        let number_of_sub_file_entries: usize = ext_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        // TODO: test with directory file entry

        Ok(())
    }

    // TODO: add tests for get_sub_file_entry_by_index
    // TODO: add tests for get_sub_file_entry_by_name
    // TODO: add tests for get_data_stream_by_name
    // TODO: add tests for read_directory_tree
}
