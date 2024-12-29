/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::rc::Rc;

use crate::checksums::ReversedCrc32Context;
use crate::datetime::DateTime;
use crate::types::{ByteString, SharedValue};
use crate::vfs::{
    VfsDataStreamReference, VfsFileEntryReference, VfsFileSystem, VfsFileSystemReference, VfsPath,
    VfsPathType,
};

use super::constants::*;
use super::features::ExtFeatures;
use super::file_entry::ExtFileEntry;
use super::group_descriptor::ExtGroupDescriptor;
use super::group_descriptor_table::ExtGroupDescriptorTable;
use super::inode::ExtInode;
use super::inode_table::ExtInodeTable;
use super::path::ExtPath;
use super::superblock::ExtSuperblock;

/// Extended File System (ext).
pub struct ExtFileSystem {
    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Features.
    features: ExtFeatures,

    /// Number of inodes.
    pub number_of_inodes: u32,

    /// Block size.
    block_size: u32,

    /// Inode size.
    inode_size: u16,

    /// Inode table.
    inode_table: Rc<ExtInodeTable>,

    /// Metadata checksum seed.
    metadata_checksum_seed: u32,

    /// Volume label.
    pub volume_label: ByteString,

    /// Last mount path.
    pub last_mount_path: ByteString,

    /// Last mount time.
    pub last_mount_time: DateTime,

    /// Last written time.
    pub last_written_time: DateTime,
}

impl ExtFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: SharedValue::none(),
            volume_label: ByteString::new(),
            features: ExtFeatures::new(),
            number_of_inodes: 0,
            block_size: 0,
            inode_size: 0,
            inode_table: Rc::new(ExtInodeTable::new()),
            metadata_checksum_seed: 0,
            last_mount_path: ByteString::new(),
            last_mount_time: DateTime::NotSet,
            last_written_time: DateTime::NotSet,
        }
    }

    /// Reads the block groups.
    fn read_block_groups(&mut self) -> io::Result<()> {
        let mut block_group_number: u32 = 0;
        let mut block_group_offset: u64 = 0;
        let mut block_group_size: u64 = 0;
        let mut exponent3: u32 = 3;
        let mut exponent5: u32 = 5;
        let mut exponent7: u32 = 7;
        let mut meta_group_number: u32 = 0;
        let mut meta_group_start_block_number: u32 = 0;
        let mut number_of_block_groups: u64 = 0;
        let mut number_of_block_groups_per_meta_group: u32 = 0;
        let mut number_of_inodes_per_block_group: u32 = 0;
        let mut group_descriptors: Vec<ExtGroupDescriptor> = Vec::new();

        loop {
            if exponent3 < block_group_number {
                exponent3 *= 3;
            }
            if exponent5 < block_group_number {
                exponent5 *= 5;
            }
            if exponent7 < block_group_number {
                exponent7 *= 7;
            }
            // The first block group will always contain a superblock.
            let block_group_has_superblock: bool = if block_group_number == 0 {
                true
            // If sparse superblock feature is disabled all block groups contain a superblock.
            } else if !self.features.has_sparse_superblock() {
                true
            // TODO: add support for sparse superblock version 2
            } else if self.features.has_sparse_superblock2() {
                false
            // Only block group numbers that are a power of 3, 5 or 7 contain a superblock.
            } else if block_group_number == 1
                || block_group_number == exponent3
                || block_group_number == exponent5
                || block_group_number == exponent7
            {
                true
            } else {
                false
            };
            if block_group_has_superblock {
                let mut superblock_offset: u64 = block_group_offset;

                if block_group_offset == 0 || self.block_size == 1024 {
                    superblock_offset += 1024;
                }
                let mut superblock: ExtSuperblock = ExtSuperblock::new();
                superblock
                    .read_at_position(&self.data_stream, io::SeekFrom::Start(superblock_offset))?;

                if block_group_number == 0 {
                    self.features.initialize(&superblock);

                    number_of_block_groups = superblock.get_number_of_block_groups();
                    block_group_size = superblock.get_block_group_size();

                    self.number_of_inodes = superblock.number_of_inodes;
                    self.block_size = superblock.block_size;
                    self.inode_size = superblock.inode_size;

                    self.volume_label = superblock.volume_label;

                    // TODO: change to ExtPath
                    self.last_mount_path = superblock.last_mount_path;

                    if superblock.last_mount_time.timestamp != 0 {
                        self.last_mount_time = DateTime::PosixTime32(superblock.last_mount_time);
                    }
                    if superblock.last_written_time.timestamp != 0 {
                        self.last_written_time =
                            DateTime::PosixTime32(superblock.last_written_time);
                    }
                    number_of_inodes_per_block_group = superblock.number_of_inodes_per_block_group;

                    if self.features.has_meta_block_groups() {
                        let group_descriptor_size: u32 = self.features.get_group_descriptor_size();
                        number_of_block_groups_per_meta_group =
                            superblock.block_size / group_descriptor_size;
                        meta_group_start_block_number = superblock.first_meta_block_group
                            * number_of_block_groups_per_meta_group;
                    }
                    self.metadata_checksum_seed = match superblock.metadata_checksum_seed {
                        Some(metadata_checksum_seed) => metadata_checksum_seed,
                        None => {
                            let mut crc32_context: ReversedCrc32Context =
                                ReversedCrc32Context::new(0x82f63b78, 0);
                            crc32_context.update(&superblock.file_system_identifier);
                            crc32_context.finalize()
                        }
                    };
                }
                // TODO: compare superblock
            }
            // When the has meta block groups feature is enabled group descriptors are stored at the
            // beginning of the first, second, and last block groups in a meta block group,
            // independent of a superblock. Otherwise group descriptors are stored in the first
            // block after a superblock.
            let mut block_group_has_group_descriptors: bool = false;
            let mut meta_group_block_group_number: u32 = 0;

            if !self.features.has_meta_block_groups()
                || block_group_number < meta_group_start_block_number
            {
                block_group_has_group_descriptors = block_group_has_superblock;
            } else {
                meta_group_block_group_number =
                    block_group_number % number_of_block_groups_per_meta_group;

                if meta_group_block_group_number == 0
                    || meta_group_block_group_number == 1
                    || meta_group_block_group_number == number_of_block_groups_per_meta_group - 1
                {
                    block_group_has_group_descriptors = true;
                }
            }
            if block_group_has_group_descriptors {
                let mut group_descriptor_offset: u64 = block_group_offset;

                if self.block_size == 1024 {
                    group_descriptor_offset += 1024;
                }
                if block_group_has_superblock {
                    group_descriptor_offset += self.block_size as u64;
                }
                let first_group_number: u32 = if !self.features.has_meta_block_groups()
                    || block_group_number < meta_group_start_block_number
                {
                    0
                } else {
                    meta_group_start_block_number
                        + (meta_group_number * number_of_block_groups_per_meta_group)
                };
                let number_of_group_descriptors: u32 = if !self.features.has_meta_block_groups() {
                    number_of_block_groups as u32
                } else if block_group_number < meta_group_start_block_number {
                    meta_group_start_block_number
                } else {
                    number_of_block_groups_per_meta_group
                };
                let mut group_descriptor_table: ExtGroupDescriptorTable =
                    ExtGroupDescriptorTable::new();
                group_descriptor_table.initialize(
                    &self.features,
                    first_group_number,
                    number_of_group_descriptors,
                );
                group_descriptor_table.read_at_position(
                    &self.data_stream,
                    io::SeekFrom::Start(group_descriptor_offset),
                )?;
                if !self.features.has_meta_block_groups()
                    || block_group_number < meta_group_start_block_number
                {
                    if block_group_number == 0 {
                        for group_descriptor in group_descriptor_table.entries.drain(0..) {
                            group_descriptors.push(group_descriptor);
                        }
                    }
                } else if meta_group_block_group_number == 0 {
                    for group_descriptor in group_descriptor_table.entries.drain(0..) {
                        group_descriptors.push(group_descriptor);
                    }
                } else if meta_group_block_group_number == number_of_block_groups_per_meta_group - 1
                {
                    meta_group_number += 1;
                };
            }
            // TODO: read block bitmap for debugging purposes
            // TODO: read inode bitmap for debugging purposes

            block_group_number += 1;
            block_group_offset += block_group_size;

            if block_group_number as u64 >= number_of_block_groups {
                break;
            }
        }
        match Rc::get_mut(&mut self.inode_table) {
            Some(inode_table) => inode_table.initialize(
                &self.features,
                self.block_size,
                self.inode_size,
                number_of_inodes_per_block_group,
                &mut group_descriptors,
            )?,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unable to initialize inode table"),
                ));
            }
        };
        Ok(())
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u8 {
        self.features.get_format_version()
    }

    /// Retrieves the compatible feature flags.
    pub fn get_compatible_feature_flags(&self) -> u32 {
        self.features.compatible_feature_flags
    }

    /// Retrieves the incompatible feature flags.
    pub fn get_incompatible_feature_flags(&self) -> u32 {
        self.features.incompatible_feature_flags
    }

    /// Retrieves the read-only compatible feature flags.
    pub fn get_read_only_compatible_feature_flags(&self) -> u32 {
        self.features.read_only_compatible_feature_flags
    }

    /// Retrieves the file entry for a specific inode number.
    pub fn get_file_entry_by_inode_number(&self, inode_number: u32) -> io::Result<ExtFileEntry> {
        if self.features.is_unsupported() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Ext file system has unsupported features"),
            ));
        }
        if inode_number == 0 || inode_number > self.number_of_inodes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid inode number: {} value out of bounds", inode_number),
            ));
        }
        let inode: ExtInode = self
            .inode_table
            .get_inode(&self.data_stream, inode_number)?;

        let name: ByteString = ByteString::new();
        let file_entry: ExtFileEntry = ExtFileEntry::new(
            &self.data_stream,
            &self.inode_table,
            inode_number,
            inode,
            name,
        );
        Ok(file_entry)
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &ExtPath) -> io::Result<Option<ExtFileEntry>> {
        if path.is_empty() || path.components[0].len() != 0 {
            return Ok(None);
        }
        let mut file_entry: ExtFileEntry = self.get_root_directory()?;

        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            file_entry = match file_entry.get_sub_file_entry_by_name(path_component)? {
                Some(file_entry) => file_entry,
                None => return Ok(None),
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> io::Result<ExtFileEntry> {
        self.get_file_entry_by_inode_number(EXT_INODE_NUMBER_ROOT_DIRECTORY)
    }
}

impl VfsFileSystem for ExtFileSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        if path.path_type != VfsPathType::Ext {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let ext_path: ExtPath = ExtPath::from(&path.location);

        let result: bool = match self.get_file_entry_by_path(&ext_path)? {
            Some(_) => true,
            None => false,
        };
        Ok(result)
    }

    /// Retrieves the path type.
    fn get_vfs_path_type(&self) -> VfsPathType {
        VfsPathType::Ext
    }

    /// Opens a file system.
    fn open(
        &mut self,
        parent_file_system: &VfsFileSystemReference,
        path: &VfsPath,
    ) -> io::Result<()> {
        let result: Option<VfsDataStreamReference> = match parent_file_system.with_write_lock() {
            Ok(file_system) => file_system.open_data_stream(path, None)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.data_stream = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.location),
                ))
            }
        };
        self.read_block_groups()
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntryReference>> {
        if path.path_type != VfsPathType::Ext {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let ext_path: ExtPath = ExtPath::from(&path.location);

        let file_entry: ExtFileEntry = match self.get_file_entry_by_path(&ext_path)? {
            Some(file_entry) => file_entry,
            None => return Ok(None),
        };
        Ok(Some(Box::new(file_entry)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType, VfsPathType};

    fn get_file_system() -> io::Result<ExtFileSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        file_system.open(&parent_file_system, &vfs_path)?;

        Ok(file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        assert_eq!(file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "./bogus2", None);
        assert_eq!(file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_inode_number() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let file_entry: ExtFileEntry = file_system.get_file_entry_by_inode_number(14)?;

        assert_eq!(file_entry.inode_number, 14);

        let name: &ByteString = file_entry.get_name();
        assert!(name.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/passwords.txt");
        let file_entry: ExtFileEntry = file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(file_entry.inode_number, 14);

        let ext_path: ExtPath = ExtPath::from("/a_directory/a_file");
        let file_entry: ExtFileEntry = file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(file_entry.inode_number, 13);

        let name: &ByteString = file_entry.get_name();
        assert!(!name.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let file_entry: ExtFileEntry = file_system.get_root_directory()?;

        assert_eq!(file_entry.inode_number, 2);

        Ok(())
    }

    #[test]
    fn test_get_vfs_path_type() -> io::Result<()> {
        let file_system: ExtFileSystem = ExtFileSystem::new();

        let vfs_path_type: VfsPathType = file_system.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Ext);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        file_system.open(&parent_file_system, &vfs_path)?;

        let format_version: u8 = file_system.get_format_version();
        assert_eq!(format_version, 2);

        let feature_flags: u32 = file_system.get_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000038);

        let feature_flags: u32 = file_system.get_incompatible_feature_flags();
        assert_eq!(feature_flags, 0x00000002);

        let feature_flags: u32 = file_system.get_read_only_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000003);

        assert_eq!(file_system.block_size, 1024);
        assert_eq!(file_system.inode_size, 128);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "/", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_file() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPath =
            VfsPath::new(VfsPathType::Ext, "/passwords.txt", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_test_open_file_entry_non_existing() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "/bogus2", Some(os_vfs_path));
        let result: Option<VfsFileEntryReference> = file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let file_system: ExtFileSystem = get_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::NotSet, "/", None);

        let result = file_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
