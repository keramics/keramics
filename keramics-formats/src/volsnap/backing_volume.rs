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

use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::indexed_hash_map::IndexedHashMap;

use super::catalog_block::VolsnapCatalogBlock;
use super::shadow_copy::VolsnapShadowCopy;
use super::snapshot::VolsnapSnapshot;
use super::snapshots::VolsnapSnapshotsIterator;
use super::store_block::VolsnapStoreBlock;
use super::store_metadata::VolsnapStoreMetadata;
use super::volume_header::VolsnapVolumeHeader;

/// Volume Shadow Snapshot (volsnap) backing volume.
pub struct VolsnapBackingVolume {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Volume identifier.
    volume_identifier: Uuid,

    /// Storage volume identifier.
    storage_volume_identifier: Uuid,

    /// Shadow copies.
    pub(super) shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy>,
}

impl VolsnapBackingVolume {
    /// Creates a new backing volume.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            volume_identifier: Uuid::new(),
            storage_volume_identifier: Uuid::new(),
            shadow_copies: IndexedHashMap::new(),
        }
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        if self.volume_identifier != self.storage_volume_identifier {
            None
        } else {
            todo!()
        }
    }

    /// Retrieves the storage volume identifier.
    pub fn get_storage_volume_identifier(&self) -> &Uuid {
        &self.storage_volume_identifier
    }

    /// Retrieves the volume identifier.
    pub fn get_volume_identifier(&self) -> &Uuid {
        &self.volume_identifier
    }

    /// Retrieves the number of snapshots.
    pub fn get_number_of_snapshots(&self) -> usize {
        self.shadow_copies.len()
    }

    /// Retrieves a snapshot by index.
    pub fn get_snapshot_by_index(
        &self,
        snapshot_index: usize,
    ) -> Result<VolsnapSnapshot, ErrorTrace> {
        match self.shadow_copies.get_key_value_by_index(snapshot_index) {
            Some((identifier, shadow_copy)) => match self.data_stream.as_ref() {
                Some(data_stream) => Ok(VolsnapSnapshot::new(data_stream, identifier, shadow_copy)),
                None => Err(keramics_core::error_trace_new!("Missing data stream")),
            },
            None => Err(keramics_core::error_trace_new!(format!(
                "No snapshot with index: {}",
                snapshot_index
            ))),
        }
    }

    /// Retrieves a snapshots iterator.
    pub fn snapshots(&self) -> VolsnapSnapshotsIterator<'_> {
        VolsnapSnapshotsIterator::new(self, self.shadow_copies.len())
    }

    /// Reads the backing volume from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut volume_header: VolsnapVolumeHeader = VolsnapVolumeHeader::new();

        match volume_header.read_at_position(data_stream, SeekFrom::Start(0x00001e00)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read volume header at offset: 7680 (0x00001e00)",
                );
                return Err(error);
            }
        }
        if volume_header.relative_block_offset != 0x00001e00 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported volume header - relative block offset value out of bounds",
            ));
        }
        if volume_header.current_block_offset != 0x00001e00 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported volume header - current block offset value out of bounds",
            ));
        }
        if volume_header.next_block_offset != 0x00000000 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported volume header - next block offset value out of bounds",
            ));
        }
        self.volume_identifier = volume_header.volume_identifier;
        self.storage_volume_identifier = volume_header.storage_volume_identifier;

        let mut catalog_block_offset: u64 = volume_header.catalog_offset;
        let mut read_catalog_blocks: HashSet<u64> = HashSet::new();

        loop {
            if read_catalog_blocks.contains(&catalog_block_offset) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Catalog block at offset: {} (0x{:08x}) already read",
                    catalog_block_offset, catalog_block_offset
                )));
            }
            let mut catalog_block: VolsnapCatalogBlock = VolsnapCatalogBlock::new();

            match catalog_block.read_at_position(
                data_stream,
                SeekFrom::Start(catalog_block_offset),
                &mut self.shadow_copies,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read catalog block at offset: {} (0x{:08x})",
                            catalog_block_offset, catalog_block_offset
                        ),
                    );
                    return Err(error);
                }
            }
            read_catalog_blocks.insert(catalog_block_offset);

            if catalog_block.current_block_offset != catalog_block_offset {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported catalog block - current block offset value out of bounds",
                ));
            }
            if catalog_block.next_block_offset == 0 {
                break;
            }
            if !catalog_block.next_block_offset.is_multiple_of(16384) {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported catalog block - next block offset value not a multiple of 16384",
                ));
            }
            catalog_block_offset = catalog_block.next_block_offset;
        }
        for (_, shadow_copy) in self.shadow_copies.iter_mut() {
            if shadow_copy.type3_entry_read && shadow_copy.store_metadata_offset != 0 {
                let mut store_block: VolsnapStoreBlock = VolsnapStoreBlock::new();

                match store_block.read_at_position(
                    data_stream,
                    SeekFrom::Start(shadow_copy.store_metadata_offset),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read store metadata block at offset: {} (0x{:08x})",
                                shadow_copy.store_metadata_offset,
                                shadow_copy.store_metadata_offset
                            ),
                        );
                        return Err(error);
                    }
                }
                if store_block.block_type != 4 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported store metadata block - unsupported block type",
                    ));
                }
                if store_block.next_block_offset != 0 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported store metadata block - unsupported next block offset",
                    ));
                }
                let data_end_offset: usize = 128 + (store_block.store_metadata_size as usize);

                if store_block.store_metadata_size < 64 || data_end_offset > store_block.data.len()
                {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported store metadata block - invalid store metadata size value out of bounds",
                    ));
                }
                keramics_core::debug_trace_data_and_structure!(
                    "VolsnapStoreMetadata",
                    shadow_copy.store_metadata_offset + 128,
                    &store_block.data[128..data_end_offset],
                    store_block.store_metadata_size,
                    VolsnapStoreMetadata::debug_read_data(&store_block.data[128..])
                );
                let mut store_metadata: VolsnapStoreMetadata = VolsnapStoreMetadata::new();

                match store_metadata.read_data(&store_block.data[128..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read store metadata"
                        );
                        return Err(error);
                    }
                }
                shadow_copy.copy_identifier = store_metadata.copy_identifier;
                shadow_copy.copy_set_identifier = store_metadata.copy_set_identifier;
                shadow_copy.attribute_flags = store_metadata.attribute_flags;
                shadow_copy.store_metadata_read = true;
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::open_os_data_stream;

    use crate::RangeStream;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;

    fn get_backing_volume() -> Result<VolsnapBackingVolume, ErrorTrace> {
        let mut backing_volume: VolsnapBackingVolume = VolsnapBackingVolume::new();

        let path_string: String = get_test_data_path("volsnap/volsnap.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            133103616,
        )));
        backing_volume.read_data_stream(&data_stream)?;

        Ok(backing_volume)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_storage_volume_identifier() -> Result<(), ErrorTrace> {
        let backing_volume: VolsnapBackingVolume = get_backing_volume()?;

        let identifier: &Uuid = backing_volume.get_storage_volume_identifier();
        assert_eq!(
            identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }

    #[test]
    fn test_get_volume_identifier() -> Result<(), ErrorTrace> {
        let backing_volume: VolsnapBackingVolume = get_backing_volume()?;

        let identifier: &Uuid = backing_volume.get_volume_identifier();
        assert_eq!(
            identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_snapshots() -> Result<(), ErrorTrace> {
        let backing_volume: VolsnapBackingVolume = get_backing_volume()?;

        let number_of_snapshots: usize = backing_volume.get_number_of_snapshots();
        assert_eq!(number_of_snapshots, 2);

        Ok(())
    }

    // TODO: add tests for get_snapshot_by_index
    // TODO: add tests for snapshots

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let mut backing_volume: VolsnapBackingVolume = VolsnapBackingVolume::new();

        let path_string: String = get_test_data_path("volsnap/volsnap.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            133103616,
        )));
        backing_volume.read_data_stream(&data_stream)?;

        // assert_eq!(backing_volume.is_locked, true);

        Ok(())
    }
}
