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

use std::io::SeekFrom;
use std::sync::Arc;

use keramics_checksums::ReversedCrc32Context;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_types::ByteString;

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
    data_stream: Option<DataStreamReference>,

    /// Features.
    features: ExtFeatures,

    /// Number of inodes.
    pub number_of_inodes: u32,

    /// Block size.
    block_size: u32,

    /// Inode size.
    inode_size: u16,

    /// Inode table.
    inode_table: Arc<ExtInodeTable>,

    /// Metadata checksum seed.
    metadata_checksum_seed: u32,

    /// Volume label.
    volume_label: ByteString,

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
            data_stream: None,
            volume_label: ByteString::new(),
            features: ExtFeatures::new(),
            number_of_inodes: 0,
            block_size: 0,
            inode_size: 0,
            inode_table: Arc::new(ExtInodeTable::new()),
            metadata_checksum_seed: 0,
            last_mount_path: ByteString::new(),
            last_mount_time: DateTime::NotSet,
            last_written_time: DateTime::NotSet,
        }
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

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> &ByteString {
        &self.volume_label
    }

    /// Retrieves the file entry for a specific identifier (inode number).
    pub fn get_file_entry_by_identifier(
        &self,
        inode_number: u32,
    ) -> Result<ExtFileEntry, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        if self.features.is_unsupported() {
            return Err(keramics_core::error_trace_new!(
                "Ext file system has unsupported features"
            ));
        }
        if inode_number == 0 || inode_number > self.number_of_inodes {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid inode number: {} value out of bounds",
                inode_number
            )));
        }
        let inode: ExtInode = match self.inode_table.get_inode(data_stream, inode_number) {
            Ok(inode) => inode,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve inode: {}", inode_number)
                );
                return Err(error);
            }
        };
        Ok(ExtFileEntry::new(
            data_stream,
            &self.inode_table,
            inode_number,
            inode,
            None,
        ))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        path: &ExtPath,
    ) -> Result<Option<ExtFileEntry>, ErrorTrace> {
        if path.is_empty() || path.components[0].len() != 0 {
            return Ok(None);
        }
        let result: Option<ExtFileEntry> = match self.get_root_directory() {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        let mut file_entry: ExtFileEntry = match result {
            Some(file_entry) => file_entry,
            None => return Ok(None),
        };
        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            let result: Option<ExtFileEntry> =
                match file_entry.get_sub_file_entry_by_name(path_component) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve sub file entry: {}",
                                path_component.to_string()
                            )
                        );
                        return Err(error);
                    }
                };
            file_entry = match result {
                Some(file_entry) => file_entry,
                None => return Ok(None),
            };
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<Option<ExtFileEntry>, ErrorTrace> {
        if self.number_of_inodes == 0 {
            return Ok(None);
        }
        match self.get_file_entry_by_identifier(EXT_ROOT_DIRECTORY_IDENTIFIER) {
            Ok(file_entry) => Ok(Some(file_entry)),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file entry: {}",
                        EXT_ROOT_DIRECTORY_IDENTIFIER
                    )
                );
                Err(error)
            }
        }
    }

    /// Reads a file system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_block_groups(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read block groups");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the block groups.
    fn read_block_groups(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
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
                if block_group_number == 0 {
                    let mut superblock: ExtSuperblock = ExtSuperblock::new();

                    match superblock
                        .read_at_position(data_stream, SeekFrom::Start(superblock_offset))
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read superblock at offset: {} (0x{:08x})",
                                    superblock_offset, superblock_offset
                                )
                            );
                            return Err(error);
                        }
                    }
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
                } else {
                    let mut superblock: ExtSuperblock = ExtSuperblock::new();

                    match superblock
                        .read_at_position(data_stream, SeekFrom::Start(superblock_offset))
                    {
                        Ok(_) => {
                            // TODO: compare superblock
                        }
                        // Ignore backup superblocks without a correct signature.
                        Err(_) => {}
                    }
                }
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
                match group_descriptor_table
                    .read_at_position(data_stream, SeekFrom::Start(group_descriptor_offset))
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read group descriptor table at offset: {} (0x{:08x})",
                                group_descriptor_offset, group_descriptor_offset
                            )
                        );
                        return Err(error);
                    }
                }
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
        if number_of_inodes_per_block_group > 0 {
            match Arc::get_mut(&mut self.inode_table) {
                Some(inode_table) => match inode_table.initialize(
                    &self.features,
                    self.block_size,
                    self.inode_size,
                    number_of_inodes_per_block_group,
                    &mut group_descriptors,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize inode table"
                        );
                        return Err(error);
                    }
                },
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unable to obtain mutable reference to inode table"
                    ));
                }
            };
        }
        // TODO: sanity check self.number_of_inodes and size of inode table.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    fn get_file_system() -> Result<ExtFileSystem, ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/ext/ext2.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file_system: ExtFileSystem = get_file_system()?;

        let format_version: u8 = file_system.get_format_version();
        assert_eq!(format_version, 2);

        Ok(())
    }

    #[test]
    fn test_get_feature_flags() -> Result<(), ErrorTrace> {
        let file_system: ExtFileSystem = get_file_system()?;

        let feature_flags: u32 = file_system.get_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000038);

        let feature_flags: u32 = file_system.get_incompatible_feature_flags();
        assert_eq!(feature_flags, 0x00000002);

        let feature_flags: u32 = file_system.get_read_only_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000003);

        Ok(())
    }

    // TODO: add tests for get_volume_label

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: ExtFileSystem = get_file_system()?;

        let file_entry: ExtFileEntry = file_system.get_file_entry_by_identifier(12)?;

        assert_eq!(file_entry.inode_number, 12);

        let name: Option<&ByteString> = file_entry.get_name();
        assert!(name.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: ExtFileSystem = get_file_system()?;

        let ext_path: ExtPath = ExtPath::from("/emptyfile");
        let file_entry: ExtFileEntry = file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(file_entry.inode_number, 12);

        let ext_path: ExtPath = ExtPath::from("/testdir1/testfile1");
        let file_entry: ExtFileEntry = file_system.get_file_entry_by_path(&ext_path)?.unwrap();

        assert_eq!(file_entry.inode_number, 14);

        let name: &ByteString = file_entry.get_name().unwrap();
        assert_eq!(name.to_string(), "testfile1");

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: ExtFileSystem = get_file_system()?;

        let file_entry: ExtFileEntry = file_system.get_root_directory()?.unwrap();

        assert_eq!(file_entry.inode_number, 2);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/ext/ext2.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.block_size, 1024);
        assert_eq!(file_system.inode_size, 128);

        Ok(())
    }

    #[test]
    fn test_read_block_groups() -> Result<(), ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/ext/ext2.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_block_groups(&data_stream)?;

        assert_eq!(file_system.block_size, 1024);
        assert_eq!(file_system.inode_size, 128);

        Ok(())
    }
}
