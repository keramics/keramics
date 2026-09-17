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

use keramics_core::DataStreamReference;
use keramics_datetime::DateTime;
use keramics_types::Uuid;

use super::shadow_copy::VolsnapShadowCopy;

/// Volume Shadow Snapshot (volsnap) snapshot.
pub struct VolsnapSnapshot {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Store identifier.
    store_identifier: Uuid,

    /// Copy identifier.
    copy_identifier: Option<Uuid>,

    /// Copy set identifier.
    copy_set_identifier: Option<Uuid>,

    /// Creation time.
    creation_time: Option<DateTime>,

    /// Store metadata offset.
    store_metadata_offset: Option<u64>,

    /// Store block list offset.
    store_block_list_offset: Option<u64>,

    /// Store block range list offset.
    store_block_range_list_offset: Option<u64>,

    /// Store bitmap offset.
    store_bitmap_offset: Option<u64>,

    /// Store previous bitmap offset.
    store_previous_bitmap_offset: Option<u64>,

    /// Size.
    size: Option<u64>,

    /// Attribute flags.
    attribute_flags: Option<u32>,
}

impl VolsnapSnapshot {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        store_identifier: &Uuid,
        shadow_copy: &VolsnapShadowCopy,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            store_identifier: store_identifier.clone(),
            copy_identifier: if shadow_copy.store_metadata_read {
                Some(shadow_copy.copy_identifier.clone())
            } else {
                None
            },
            copy_set_identifier: if shadow_copy.store_metadata_read {
                Some(shadow_copy.copy_set_identifier.clone())
            } else {
                None
            },
            creation_time: if shadow_copy.type2_entry_read {
                Some(shadow_copy.creation_time.clone())
            } else {
                None
            },
            store_metadata_offset: if shadow_copy.type3_entry_read {
                Some(shadow_copy.store_metadata_offset)
            } else {
                None
            },
            store_block_list_offset: if shadow_copy.type3_entry_read {
                Some(shadow_copy.store_block_list_offset)
            } else {
                None
            },
            store_block_range_list_offset: if shadow_copy.type3_entry_read {
                Some(shadow_copy.store_block_range_list_offset)
            } else {
                None
            },
            store_bitmap_offset: if shadow_copy.type3_entry_read {
                Some(shadow_copy.store_bitmap_offset)
            } else {
                None
            },
            store_previous_bitmap_offset: if shadow_copy.type3_entry_read {
                Some(shadow_copy.store_previous_bitmap_offset)
            } else {
                None
            },
            size: if shadow_copy.type2_entry_read {
                Some(shadow_copy.size)
            } else {
                None
            },
            attribute_flags: None,
        }
    }

    /// Retrieves the copy identifier.
    pub fn get_copy_identifier(&self) -> Option<&Uuid> {
        self.copy_identifier.as_ref()
    }

    /// Retrieves the copy set identifier.
    pub fn get_copy_set_identifier(&self) -> Option<&Uuid> {
        self.copy_set_identifier.as_ref()
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.creation_time.as_ref()
    }

    // TODO: add get_data_stream

    /// Retrieves the size.
    pub fn get_size(&self) -> Option<&u64> {
        self.size.as_ref()
    }

    /// Retrieves the store identifier.
    pub fn get_store_identifier(&self) -> &Uuid {
        &self.store_identifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::{ErrorTrace, open_os_data_stream};
    use keramics_datetime::Filetime;

    use crate::RangeStream;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;
    use crate::volsnap::backing_volume::VolsnapBackingVolume;

    fn get_snapshot() -> Result<VolsnapSnapshot, ErrorTrace> {
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

        let identifier: Uuid = Uuid::from_string("9f17819b-b0f9-11f1-90dc-7ced8d4e4e79")?;
        let shadow_copy: &VolsnapShadowCopy = backing_volume
            .shadow_copies
            .get_value_by_key(&identifier)
            .unwrap();

        Ok(VolsnapSnapshot::new(&data_stream, &identifier, shadow_copy))
    }

    #[test]
    fn test_get_copy_identifier() -> Result<(), ErrorTrace> {
        let snapshot: VolsnapSnapshot = get_snapshot()?;

        let identifier: &Uuid = snapshot.get_copy_identifier().unwrap();
        assert_eq!(
            identifier.to_string(),
            "54ab4fc9-3eae-4ded-8e85-6f1f864251dc"
        );
        Ok(())
    }

    #[test]
    fn test_get_copy_set_identifier() -> Result<(), ErrorTrace> {
        let snapshot: VolsnapSnapshot = get_snapshot()?;

        let identifier: &Uuid = snapshot.get_copy_set_identifier().unwrap();
        assert_eq!(
            identifier.to_string(),
            "6755c5a0-eb62-41e9-91d6-490246c61375"
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let snapshot: VolsnapSnapshot = get_snapshot()?;

        assert_eq!(
            snapshot.get_creation_time(),
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1dd450ead9f43b0
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let snapshot: VolsnapSnapshot = get_snapshot()?;

        let size: Option<&u64> = snapshot.get_size();
        assert_eq!(size, Some(133103616).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_store_identifier() -> Result<(), ErrorTrace> {
        let snapshot: VolsnapSnapshot = get_snapshot()?;

        let identifier: &Uuid = snapshot.get_store_identifier();
        assert_eq!(
            identifier.to_string(),
            "9f17819b-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }
}
