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

use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::path::Path;

use super::features::XfsFeatures;
use super::file_entry::XfsFileEntry;
use super::inode_information::XfsInodeInformation;
use super::inode_tree::XfsInodeTree;
use super::superblock::XfsSuperblock;

/// X File System (XFS) file system
pub struct XfsFileSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Features.
    features: XfsFeatures,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Inode size.
    inode_size: u16,

    /// Inode tree.
    inode_tree: Arc<XfsInodeTree>,

    /// Root directory (absolute) inode number.
    root_directory_inode_number: u64,

    /// Volume label.
    volume_label: Option<ByteString>,
}

impl XfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            character_encoding: CharacterEncoding::Utf8,
            features: XfsFeatures::new(),
            bytes_per_sector: 0,
            block_size: 0,
            inode_size: 0,
            inode_tree: Arc::new(XfsInodeTree::new()),
            root_directory_inode_number: 0,
            volume_label: None,
        }
    }

    /// Retrieves the block size.
    pub fn get_block_size(&self) -> u32 {
        self.block_size
    }

    /// Retrieves the compatible feature flags.
    pub fn get_compatible_feature_flags(&self) -> u32 {
        self.features.compatible_feature_flags
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u8 {
        self.features.format_version as u8
    }

    /// Retrieves the incompatible feature flags.
    pub fn get_incompatible_feature_flags(&self) -> u32 {
        self.features.incompatible_feature_flags
    }

    /// Retrieves the inode size.
    pub fn get_inode_size(&self) -> u16 {
        self.inode_size
    }

    /// Retrieves the read-only compatible feature flags.
    pub fn get_read_only_compatible_feature_flags(&self) -> u32 {
        self.features.read_only_compatible_feature_flags
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&ByteString> {
        self.volume_label.as_ref()
    }

    /// Retrieves the file entry for a specific identifier (inode number).
    pub fn get_file_entry_by_identifier(
        &self,
        inode_number: u64,
    ) -> Result<Option<XfsFileEntry>, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        if self.features.is_unsupported() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported file systems features"
            ));
        }
        match self
            .inode_tree
            .get_inode_by_identifier(data_stream, inode_number)
        {
            Ok(Some(inode)) => Ok(Some(XfsFileEntry::new(
                data_stream,
                &self.inode_tree,
                &self.character_encoding,
                self.features.has_directory_v2(),
                self.features.has_file_type(),
                inode_number,
                inode,
                None,
            ))),
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve inode: {}", inode_number)
                );
                Err(error)
            }
        }
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<XfsFileEntry>, ErrorTrace> {
        if path.is_empty() || path.is_relative() {
            return Ok(None);
        }
        let mut file_entry: XfsFileEntry = match self.get_root_directory() {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        for path_component in path.components[1..].iter() {
            file_entry = match file_entry.get_sub_file_entry_by_name(path_component) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve sub file entry: {}", path_component)
                    );
                    return Err(error);
                }
            };
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<Option<XfsFileEntry>, ErrorTrace> {
        if self.root_directory_inode_number == 0xffffffffffffffff {
            return Ok(None);
        }
        match self.get_file_entry_by_identifier(self.root_directory_inode_number) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file entry: {}",
                        self.root_directory_inode_number
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
        match self.read_allocation_groups(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read allocation groups");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the allocation groups.
    fn read_allocation_groups(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut superblock_offset: u64 = 0;
        let mut allocation_group_index: u32 = 0;
        let mut allocation_group_size: u64 = 0;
        let mut number_of_allocation_groups: u32 = 1;
        let mut inode_tree: XfsInodeTree = XfsInodeTree::new();

        loop {
            let mut superblock: XfsSuperblock = XfsSuperblock::new(&self.character_encoding);

            match superblock.read_at_position(data_stream, SeekFrom::Start(superblock_offset)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read allocation group: {} superblock at offset: {} (0x{:08x})",
                            allocation_group_index, superblock_offset, superblock_offset
                        )
                    );
                    return Err(error);
                }
            }
            if allocation_group_index == 0 {
                // Note that the flag values of successive superblocks can contain random data.
                if superblock.secondary_feature_flags != superblock.secondary_feature_flags_copy {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Secondary feature flags: 0x{:08x} does not match copy: 0x{:08x}",
                        superblock.secondary_feature_flags, superblock.secondary_feature_flags_copy
                    )));
                }
                self.block_size = superblock.block_size;
                self.bytes_per_sector = superblock.bytes_per_sector;
                self.inode_size = superblock.inode_size;
                self.root_directory_inode_number = superblock.root_directory_inode_number;

                self.features.initialize(&superblock);
                inode_tree.initialize(
                    &superblock,
                    self.features.has_bigtime(),
                    self.features.has_64bit_number_of_extents(),
                    superblock.root_directory_inode_number,
                );
                if !superblock.volume_label.is_empty() {
                    self.volume_label = Some(superblock.volume_label);
                }
                allocation_group_size =
                    (superblock.allocation_group_size as u64) * (self.block_size as u64);
                number_of_allocation_groups = superblock.number_of_allocation_groups;
            }
            let inode_information_offset: u64 =
                superblock_offset + (2 * (self.bytes_per_sector as u64));

            let mut inode_information: XfsInodeInformation = XfsInodeInformation::new();

            match inode_information
                .read_at_position(data_stream, SeekFrom::Start(inode_information_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read allocation group: {} inode information at offset: {} (0x{:08x})",
                            allocation_group_index,
                            inode_information_offset,
                            inode_information_offset
                        )
                    );
                    return Err(error);
                }
            }
            inode_tree
                .root_block_numbers
                .push(inode_information.inode_btree_root_block_number);

            allocation_group_index += 1;
            superblock_offset += allocation_group_size as u64;

            if allocation_group_index >= number_of_allocation_groups || allocation_group_size == 0 {
                break;
            }
        }
        self.inode_tree = Arc::new(inode_tree);

        Ok(())
    }

    /// Sets the character encoding.
    pub fn set_character_encoding(
        &mut self,
        character_encoding: &CharacterEncoding,
    ) -> Result<(), ErrorTrace> {
        self.character_encoding = character_encoding.clone();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_types::ByteString;

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
    fn test_get_block_size() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let block_size: u32 = file_system.get_block_size();
        assert_eq!(block_size, 4096);

        Ok(())
    }

    #[test]
    fn test_get_compatible_feature_flags() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let feature_flags: u32 = file_system.get_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let format_version: u8 = file_system.get_format_version();
        assert_eq!(format_version, 5);

        Ok(())
    }

    #[test]
    fn test_get_incompatible_feature_flags() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let feature_flags: u32 = file_system.get_incompatible_feature_flags();
        assert_eq!(feature_flags, 0x000000e3);

        Ok(())
    }

    #[test]
    fn test_get_inode_size() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let inode_size: u16 = file_system.get_inode_size();
        assert_eq!(inode_size, 512);

        Ok(())
    }

    #[test]
    fn test_get_read_only_compatible_feature_flags() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let feature_flags: u32 = file_system.get_read_only_compatible_feature_flags();
        assert_eq!(feature_flags, 0x0000000f);

        Ok(())
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let volume_label: Option<&ByteString> = file_system.get_volume_label();
        assert_eq!(volume_label, Some(ByteString::from("xfs_test")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let file_entry: XfsFileEntry = file_system.get_file_entry_by_identifier(16128)?.unwrap();
        assert_eq!(file_entry.inode_number, 16128);

        let name: Option<&ByteString> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry: XfsFileEntry = file_system.get_file_entry_by_identifier(16131)?.unwrap();
        assert_eq!(file_entry.inode_number, 16131);

        let name: Option<&ByteString> = file_entry.get_name();
        assert!(name.is_none());

        let result: Option<XfsFileEntry> = file_system.get_file_entry_by_identifier(99999)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let file_entry: XfsFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.inode_number, 16128);

        let path: Path = Path::from("/emptyfile");
        let file_entry: XfsFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.inode_number, 16131);

        let path: Path = Path::from("/testdir1/testfile1");
        let file_entry: XfsFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.inode_number, 16133);

        let name: Option<&ByteString> = file_entry.get_name();
        assert_eq!(name, Some(ByteString::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: XfsFileSystem = get_file_system()?;

        let file_entry: XfsFileEntry = file_system.get_root_directory()?.unwrap();

        assert_eq!(file_entry.inode_number, 16128);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file_system: XfsFileSystem = XfsFileSystem::new();

        let path_string: String = get_test_data_path("xfs/xfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.block_size, 4096);
        assert_eq!(file_system.inode_size, 512);

        Ok(())
    }

    #[test]
    fn test_set_character_encoding() -> Result<(), ErrorTrace> {
        let mut file_system: XfsFileSystem = XfsFileSystem::new();

        assert_eq!(file_system.character_encoding, CharacterEncoding::Utf8);

        file_system.set_character_encoding(&CharacterEncoding::Ascii)?;
        assert_eq!(file_system.character_encoding, CharacterEncoding::Ascii);

        Ok(())
    }
}
