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
use super::volume_header::VolsnapVolumeHeader;

/// Volume Shadow Snapshot (volsnap) volume.
pub struct VolsnapVolume {
    /// Data stream.
    pub(super) data_stream: Option<DataStreamReference>,

    /// Volume identifier.
    pub(super) volume_identifier: Uuid,

    /// Storage volume identifier.
    pub(super) storage_volume_identifier: Uuid,

    /// Shadow copies.
    pub(super) shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy>,
}

impl VolsnapVolume {
    /// Creates a new volume.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            volume_identifier: Uuid::new(),
            storage_volume_identifier: Uuid::new(),
            shadow_copies: IndexedHashMap::new(),
        }
    }

    /// Reads the volume from a data stream.
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
                match shadow_copy
                    .read_store_metadata(data_stream, shadow_copy.store_metadata_offset)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read store metadata",
                        );
                        return Err(error);
                    }
                }
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

    fn get_volume() -> Result<VolsnapVolume, ErrorTrace> {
        let mut volume: VolsnapVolume = VolsnapVolume::new();

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
        volume.read_data_stream(&data_stream)?;

        Ok(volume)
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume: VolsnapVolume = VolsnapVolume::new();

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
        volume.read_data_stream(&data_stream)?;

        assert_eq!(
            volume.volume_identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        assert_eq!(
            volume.storage_volume_identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }
}
