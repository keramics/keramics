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
use keramics_types::Uuid;

use super::checkpoint_map::ApfsCheckpointMap;
use super::checkpoint_map_entry::ApfsCheckpointMapEntry;
use super::container_superblock::ApfsContainerSuperblock;
use super::object_header::ApfsObjectHeader;
use super::object_map::ApfsObjectMap;
use super::object_map_tree::ApfsObjectMapTree;
use super::object_map_value::ApfsObjectMapValue;
use super::volume::ApfsVolume;
use super::volume_superblock::ApfsVolumeSuperblock;
use super::volumes::ApfsVolumesIterator;

/// Apple File System (APFS) container.
pub struct ApfsContainer {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Identifier.
    identifier: Uuid,

    /// Block size.
    block_size: u32,

    /// Features flags.
    feature_flags: u64,

    /// Read-only compatible feature flags.
    read_only_compatible_feature_flags: u64,

    /// Incompatible feature flags.
    incompatible_feature_flags: u64,

    /// Transaction identifier.
    transaction_identifier: u64,

    /// Volume object identifiers.
    volume_object_identifiers: Vec<u64>,

    /// Object map B-tree.
    object_map_tree: Arc<ApfsObjectMapTree>,
}

impl ApfsContainer {
    /// Creates a container.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            identifier: Uuid::new(),
            block_size: 0,
            feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
            transaction_identifier: 0,
            volume_object_identifiers: Vec::new(),
            object_map_tree: Arc::new(ApfsObjectMapTree::new()),
        }
    }

    /// Retrieves the block size.
    pub fn get_block_size(&self) -> u32 {
        self.block_size
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

    /// Retrieves the read-only compatible feature flags.
    pub fn get_read_only_compatible_feature_flags(&self) -> u64 {
        self.read_only_compatible_feature_flags
    }

    /// Retrieves the number of volumes.
    pub fn get_number_of_volumes(&self) -> usize {
        self.volume_object_identifiers.len()
    }

    /// Retrieves a volume by index.
    pub fn get_volume_by_index(&self, volume_index: usize) -> Result<ApfsVolume, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let object_identifier: u64 = match self.volume_object_identifiers.get(volume_index) {
            Some(object_identifier) => *object_identifier,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No volume with index: {}",
                    volume_index
                )));
            }
        };
        let object_map_value: ApfsObjectMapValue =
            match self.object_map_tree.get_value_by_object_identifier(
                &data_stream,
                object_identifier,
                self.transaction_identifier,
            ) {
                Ok(Some(object_map_value)) => object_map_value,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing object map value of volume object: {}",
                        object_identifier
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve object map value of volume object: {}",
                            object_identifier
                        )
                    );
                    return Err(error);
                }
            };
        let offset: u64 = object_map_value.physical_address * (self.block_size as u64);

        let mut superblock: ApfsVolumeSuperblock = ApfsVolumeSuperblock::new();

        match superblock.read_at_position(&data_stream, SeekFrom::Start(offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read volume superblock at offset: {} (0x{:08x}))",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        let mut volume: ApfsVolume = ApfsVolume::new(superblock);

        match volume.open(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open volume: {}", volume_index)
                );
                return Err(error);
            }
        }
        Ok(volume)
    }

    /// Retrieves a volumes iterator.
    pub fn volumes(&self) -> ApfsVolumesIterator<'_> {
        ApfsVolumesIterator::new(self, self.volume_object_identifiers.len())
    }

    /// Reads the container from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut superblock: ApfsContainerSuperblock = ApfsContainerSuperblock::new();

        match superblock.read_at_position(&data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read superblock at offset: 0 (0x00000000)"
                );
                return Err(error);
            }
        }
        if superblock.incompatible_feature_flags & 0x0000000000000001 != 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported format version 1"
            ));
        }
        if superblock.incompatible_feature_flags & 0x0000000000000100 != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported Fusion drive"));
        }
        if superblock.block_size != 4096 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported block size: {}",
                superblock.block_size
            )));
        }
        match self.read_checkpoint_descriptor_area(&data_stream, &mut superblock) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read checkpoint descriptor area"
                );
                return Err(error);
            }
        }
        self.identifier = superblock.container_identifier;
        self.block_size = superblock.block_size;
        self.feature_flags = superblock.feature_flags;
        self.read_only_compatible_feature_flags = superblock.read_only_compatible_feature_flags;
        self.incompatible_feature_flags = superblock.incompatible_feature_flags;
        self.transaction_identifier = superblock.object_header.transaction_identifier;
        self.volume_object_identifiers = superblock.volume_object_identifiers;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the checkpoint descriptor area.
    fn read_checkpoint_descriptor_area(
        &mut self,
        data_stream: &DataStreamReference,
        superblock: &mut ApfsContainerSuperblock,
    ) -> Result<(), ErrorTrace> {
        let block_size: u64 = superblock.block_size as u64;
        let mut offset: u64 = superblock.checkpoint_descriptor_area.block_number * block_size;
        let end_offset: u64 =
            offset + (superblock.checkpoint_descriptor_area.number_of_blocks * block_size);
        let mut checkpoint_map_offset: u64 = 0;
        let mut checkpoint_map_transaction_identifier: u64 = 0;

        while offset < end_offset {
            let mut object_header: ApfsObjectHeader = ApfsObjectHeader::new();

            match object_header.read_at_position(&data_stream, SeekFrom::Start(offset)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read object header at offset: {} (0x{:08x}))",
                            offset, offset
                        )
                    );
                    return Err(error);
                }
            }
            match object_header.object_type {
                0x4000000c => {
                    if object_header.transaction_identifier > checkpoint_map_transaction_identifier
                    {
                        checkpoint_map_offset = offset;
                        checkpoint_map_transaction_identifier =
                            object_header.transaction_identifier;
                    }
                }
                0x80000001 => {
                    if object_header.transaction_identifier
                        > superblock.object_header.transaction_identifier
                    {
                        match superblock.read_at_position(&data_stream, SeekFrom::Start(offset)) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to read superblock at offset: {} (0x{:08x}))",
                                        offset, offset
                                    )
                                );
                                return Err(error);
                            }
                        }
                    }
                }
                _ => {}
            }
            offset += block_size;
        }
        if checkpoint_map_offset == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unable to locate checkpoint map"
            ));
        }
        if superblock.object_map_block_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid superblock - missing object map block number"
            ));
        }
        let mut checkpoint_map_entries: Vec<ApfsCheckpointMapEntry> = Vec::new();

        while checkpoint_map_offset < end_offset {
            let mut checkpoint_map: ApfsCheckpointMap = ApfsCheckpointMap::new();

            match checkpoint_map
                .read_at_position(&data_stream, SeekFrom::Start(checkpoint_map_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read checkpoint map at offset: {} (0x{:08x}))",
                            checkpoint_map_offset, checkpoint_map_offset
                        )
                    );
                    return Err(error);
                }
            }
            if checkpoint_map.object_header.transaction_identifier
                != checkpoint_map_transaction_identifier
            {
                return Err(keramics_core::error_trace_new!(
                    "Invalid checkpoint map chain - transaction identifier does not match"
                ));
            }
            checkpoint_map_entries.append(&mut checkpoint_map.entries);

            if checkpoint_map.flags & 0x00000001 != 0 {
                break;
            }
            checkpoint_map_offset += block_size;
        }
        let object_map_offset: u64 = superblock.object_map_block_number * block_size;

        let mut object_map: ApfsObjectMap = ApfsObjectMap::new();

        match object_map.read_at_position(&data_stream, SeekFrom::Start(object_map_offset)) {
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
        match Arc::get_mut(&mut self.object_map_tree) {
            Some(object_map_tree) => {
                match object_map_tree
                    .initialize(superblock.block_size, object_map.btree_block_number)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize object map tree"
                        );
                        return Err(error);
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to object map tree"
                ));
            }
        }
        // TODO: determine snapshots based on objects map.
        // TODO: read optional key bag.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        assert_eq!(
            container.identifier.to_string(),
            "34d0674d-da87-4991-a3de-27eb13011c3e"
        );
        assert_eq!(container.block_size, 4096);
        assert_eq!(container.feature_flags, 0x00000000);
        assert_eq!(container.read_only_compatible_feature_flags, 0x00000000);
        assert_eq!(container.incompatible_feature_flags, 0x00000002);

        Ok(())
    }
}
