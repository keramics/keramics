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

use keramics_core::DataStreamReference;
use keramics_datetime::DateTime;
use keramics_types::Uuid;

use super::block_reader::VolsnapBlockReader;
use super::block_stream::VolsnapBlockStream;
use super::volume::VolsnapVolume;

/// Volume Shadow Snapshot (volsnap) snapshot.
pub struct VolsnapSnapshot {
    /// Snapshot (or source) volume.
    snapshot_volume: Arc<VolsnapVolume>,

    /// Storage volume.
    storage_volume: Option<Arc<VolsnapVolume>>,

    /// Store identifier.
    store_identifier: Uuid,
}

impl VolsnapSnapshot {
    /// Creates a new partition.
    pub(super) fn new(
        snapshot_volume: &Arc<VolsnapVolume>,
        storage_volume: Option<&Arc<VolsnapVolume>>,
        store_identifier: &Uuid,
    ) -> Self {
        Self {
            snapshot_volume: snapshot_volume.clone(),
            storage_volume: storage_volume.cloned(),
            store_identifier: store_identifier.clone(),
        }
    }

    /// Retrieves the copy identifier.
    pub fn get_copy_identifier(&self) -> Option<&Uuid> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.store_metadata_read {
                    Some(&shadow_copy.copy_identifier)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the copy set identifier.
    pub fn get_copy_set_identifier(&self) -> Option<&Uuid> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.store_metadata_read {
                    Some(&shadow_copy.copy_set_identifier)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.type2_entry_read {
                    Some(&shadow_copy.creation_time)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.store_metadata_read {
                    Some(Arc::new(RwLock::new(VolsnapBlockStream::new(
                        VolsnapBlockReader::new(
                            &self.snapshot_volume,
                            self.storage_volume.as_ref(),
                            16384,
                            shadow_copy,
                        ),
                    ))))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> Option<&u64> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.type2_entry_read {
                    Some(&shadow_copy.size)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the store identifier.
    pub fn get_store_identifier(&self) -> &Uuid {
        &self.store_identifier
    }

    /// Retrieves the store index.
    pub fn get_store_index(&self) -> Option<&usize> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_key(&self.store_identifier)
        {
            Some(shadow_copy) => {
                if shadow_copy.type3_entry_read {
                    Some(&shadow_copy.store_index)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{MAIN_SEPARATOR_STR, PathBuf};
    use std::sync::RwLock;

    use keramics_core::{ErrorTrace, open_os_data_stream};
    use keramics_datetime::Filetime;

    use crate::file_resolver::{FileResolver, FileResolverReference};
    use crate::path_component::PathComponent;
    use crate::range_stream::RangeStream;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;
    use crate::volsnap::shadow_storage::VolsnapShadowStorage;

    struct TestFileResolver {
        /// Base path.
        base_path: PathBuf,
    }

    impl TestFileResolver {
        /// Creates a new file resolver.
        pub fn new(base_path: PathBuf) -> Self {
            Self { base_path }
        }

        /// Retrieves a path based on the base path and path components.
        fn get_path(&self, path_components: &[PathComponent]) -> PathBuf {
            let mut path_buf: PathBuf = self.base_path.clone();

            for path_component in path_components.iter() {
                match path_component {
                    PathComponent::ByteString(byte_string) => {
                        path_buf.push(byte_string.to_string());
                    }
                    PathComponent::Current => path_buf.push("."),
                    PathComponent::OsString(os_string) => path_buf.push(os_string),
                    PathComponent::Parent => path_buf.push(".."),
                    PathComponent::Root => path_buf.push(MAIN_SEPARATOR_STR),
                    PathComponent::String(string) => path_buf.push(string),
                    PathComponent::Ucs2String(ucs2_string) => {
                        path_buf.push(ucs2_string.to_string());
                    }
                    PathComponent::Utf16String(utf16_string) => {
                        path_buf.push(utf16_string.to_string());
                    }
                }
            }
            path_buf
        }
    }

    impl FileResolver for TestFileResolver {
        /// Retrieves a data stream with the specified path.
        fn get_data_stream(
            &self,
            path_components: &[PathComponent],
        ) -> Result<Option<DataStreamReference>, ErrorTrace> {
            let path_buf: PathBuf = self.get_path(path_components);

            let os_data_stream: DataStreamReference = match open_os_data_stream(&path_buf) {
                Ok(data_stream) => data_stream,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open file: {}", path_buf.display())
                    );
                    return Err(error);
                }
            };
            let mut vhd_file: VhdFile = VhdFile::new();

            match vhd_file.read_data_stream(&os_data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open VHD file");
                    return Err(error);
                }
            }
            let vhd_data_stream: DataStreamReference = match vhd_file.get_data_stream() {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!("Missing data stream"));
                }
            };
            Ok(Some(Arc::new(RwLock::new(RangeStream::new(
                &vhd_data_stream,
                65536,
                133103616,
            )))))
        }
    }

    fn get_snapshot() -> Result<VolsnapSnapshot, ErrorTrace> {
        let mut shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        let path_string: String = get_test_data_path("volsnap");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

        let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];

        shadow_storage.open(&file_resolver, &file_names)?;

        let identifier: Uuid = Uuid::from_string("9f17819b-b0f9-11f1-90dc-7ced8d4e4e79")?;

        Ok(VolsnapSnapshot::new(
            &shadow_storage.snapshot_volume,
            None,
            &identifier,
        ))
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
