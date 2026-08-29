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
use keramics_types::{ByteString, Uuid};

use super::block_range::ApfsBlockRange;
use super::file_system::ApfsFileSystem;
use super::key_bag::ApfsKeyBag;
use super::object_map::ApfsObjectMap;
use super::object_map_tree::ApfsObjectMapTree;
use super::object_map_value::ApfsObjectMapValue;
use super::volume_superblock::ApfsVolumeSuperblock;

/// Apple File System (APFS) volume.
pub struct ApfsVolume {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Volume index.
    volume_index: usize,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Container key bag.
    container_key_bag: Option<Arc<ApfsKeyBag>>,

    /// Object map B-tree.
    object_map_tree: Arc<ApfsObjectMapTree>,

    /// Identifier.
    identifier: Uuid,

    /// Transaction identifier.
    transaction_identifier: u64,

    /// Features flags.
    feature_flags: u64,

    /// Read-only compatible feature flags.
    read_only_compatible_feature_flags: u64,

    /// Incompatible feature flags.
    incompatible_feature_flags: u64,

    /// Volume label.
    volume_label: ByteString,

    /// Size.
    size: u64,

    /// File system root object identifier.
    file_system_root_object_identifier: u64,

    /// Value to indicate the volume is locked.
    is_locked: bool,
}

impl ApfsVolume {
    /// Creates a volume.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        volume_index: usize,
        bytes_per_sector: u16,
        block_size: u32,
        container_key_bag: Option<&Arc<ApfsKeyBag>>,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            volume_index,
            bytes_per_sector,
            block_size,
            container_key_bag: container_key_bag.cloned(),
            object_map_tree: Arc::new(ApfsObjectMapTree::new()),
            identifier: Uuid::new(),
            transaction_identifier: 0,
            feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            volume_label: ByteString::new(),
            size: 0,
            file_system_root_object_identifier: 0,
            is_locked: false,
        }
    }

    /// Retrieves the feature flags.
    pub fn get_feature_flags(&self) -> u64 {
        self.feature_flags
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the incompatible feature flags.
    pub fn get_incompatible_feature_flags(&self) -> u64 {
        self.incompatible_feature_flags
    }

    /// Retrieves the file system.
    pub fn get_file_system(&self) -> Result<ApfsFileSystem, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Volume is locked"));
        }
        let object_map_value: ApfsObjectMapValue =
            match self.object_map_tree.get_value_by_identifier(
                &self.data_stream,
                self.file_system_root_object_identifier,
                self.transaction_identifier,
            ) {
                Ok(Some(object_map_value)) => object_map_value,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing object map value of file system object: {}",
                        self.file_system_root_object_identifier
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve object map value of file system object: {}",
                            self.file_system_root_object_identifier
                        )
                    );
                    return Err(error);
                }
            };
        let use_case_folding: bool = self.incompatible_feature_flags & 0x00000000000000001 != 0;

        let mut file_system: ApfsFileSystem =
            ApfsFileSystem::new(self.block_size, &self.object_map_tree, use_case_folding);

        match file_system.open(
            &self.data_stream,
            object_map_value.physical_address,
            self.transaction_identifier,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        }
        Ok(file_system)
    }

    /// Retrieves the read-only compatible feature flags.
    pub fn get_read_only_compatible_feature_flags(&self) -> u64 {
        self.read_only_compatible_feature_flags
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the volume index.
    pub fn get_volume_index(&self) -> usize {
        self.volume_index
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&ByteString> {
        if self.volume_label.is_empty() {
            None
        } else {
            Some(&self.volume_label)
        }
    }

    /// Determines if the volume is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Opens a volume.
    pub(super) fn open(&mut self, superblock_block_number: u64) -> Result<(), ErrorTrace> {
        let superblock_offset: u64 = superblock_block_number * (self.block_size as u64);

        let mut superblock: ApfsVolumeSuperblock = ApfsVolumeSuperblock::new();

        match superblock.read_at_position(&self.data_stream, SeekFrom::Start(superblock_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read volume superblock at offset: {} (0x{:08x}))",
                        superblock_offset, superblock_offset
                    )
                );
                return Err(error);
            }
        }
        if superblock.object_map_block_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid superblock - missing object map block number"
            ));
        }
        if superblock.file_system_root_object_identifier == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid superblock - missing file system root object identifier"
            ));
        }
        let volume_identifier: Uuid = Uuid::from_be_bytes(&superblock.volume_identifier);

        let object_map_offset: u64 = superblock.object_map_block_number * (self.block_size as u64);

        let mut object_map: ApfsObjectMap = ApfsObjectMap::new();

        match object_map.read_at_position(&self.data_stream, SeekFrom::Start(object_map_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read object map at offset: {} (0x{:08x}))",
                        object_map_offset, object_map_offset
                    )
                );
                return Err(error);
            }
        }
        if object_map.btree_block_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid object map - missing B-tree block number"
            ));
        }
        match Arc::get_mut(&mut self.object_map_tree) {
            Some(object_map_tree) => {
                object_map_tree.initialize(self.block_size, object_map.btree_block_number);
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to object map tree"
                ));
            }
        }
        if superblock.volume_flags & 0x0000000000000001 == 0 {
            let container_key_bag: &Arc<ApfsKeyBag> = match &self.container_key_bag {
                Some(container_key_bag) => container_key_bag,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Volume is encrypted and no container key bag was found"
                    ));
                }
            };
            // If the container key bag is locked the volume is also locked.
            self.is_locked = container_key_bag.is_locked;

            match container_key_bag.get_entry(&volume_identifier, 3) {
                Some(entry_data) => {
                    keramics_core::debug_trace_data_and_structure!(
                        "ApfsVolumeKeyBagBlockRange",
                        0,
                        &entry_data,
                        entry_data.len(),
                        ApfsBlockRange::debug_read_data(&entry_data)
                    );
                    let mut block_range: ApfsBlockRange = ApfsBlockRange::new();

                    match block_range.read_data(entry_data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read volume key bag block range"
                            );
                            return Err(error);
                        }
                    }
                    if block_range.block_number == 0 || block_range.number_of_blocks == 0 {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid volume key bag block range"
                        ));
                    }
                    let key_bag_offset: u64 = block_range.block_number * (self.block_size as u64);
                    let key_bag_size: u64 = block_range.number_of_blocks * (self.block_size as u64);

                    let mut key_bag: ApfsKeyBag = ApfsKeyBag::new(
                        self.bytes_per_sector,
                        &superblock.volume_identifier,
                        &superblock.volume_identifier,
                    );
                    match key_bag.read_at_position(
                        &self.data_stream,
                        key_bag_size,
                        SeekFrom::Start(key_bag_offset),
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read key bag at offset: {} (0x{:08x}))",
                                    key_bag_offset, key_bag_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                    if key_bag.object_header.object_type != 0x72656373 {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported volume key bag object type"
                        ));
                    }
                    // The volume has a key bag and therefore is locked.
                    self.is_locked = true;
                }
                None => {}
            }
        }
        // TODO: add snapshot support

        self.identifier = volume_identifier;
        self.transaction_identifier = superblock.object_header.transaction_identifier;
        self.feature_flags = superblock.feature_flags;
        self.read_only_compatible_feature_flags = superblock.read_only_compatible_feature_flags;
        self.incompatible_feature_flags = superblock.incompatible_feature_flags;
        self.volume_label = superblock.volume_label;
        self.size = superblock.number_of_allocated_blocks * (self.block_size as u64);
        self.file_system_root_object_identifier = superblock.file_system_root_object_identifier;

        Ok(())
    }

    // TODO: add unlock
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::apfs::ApfsContainer;

    use crate::tests::get_test_data_path;

    fn get_volume() -> Result<ApfsVolume, ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        container.get_volume_by_index(0)
    }

    #[test]
    fn test_get_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_feature_flags();
        assert_eq!(feature_flags, 0x00000002);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let identifier: &Uuid = volume.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "33d13da9-f1c8-4d2a-b9c7-71ab9dbe5fe2"
        );
        Ok(())
    }

    #[test]
    fn test_get_incompatible_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_incompatible_feature_flags();
        assert_eq!(feature_flags, 0x00000001);

        Ok(())
    }

    #[test]
    fn test_get_read_only_compatible_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_read_only_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let size: u64 = volume.get_size();
        assert_eq!(size, 77824);

        Ok(())
    }

    #[test]
    fn test_get_volume_index() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let volume_index: usize = volume.get_volume_index();
        assert_eq!(volume_index, 0);

        Ok(())
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let volume_label: Option<&ByteString> = volume.get_volume_label();
        assert_eq!(volume_label, Some(ByteString::from("apfs_test")).as_ref());

        Ok(())
    }

    // TODO: add tests for open
}
