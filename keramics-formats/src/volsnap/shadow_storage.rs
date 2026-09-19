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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::snapshot::VolsnapSnapshot;
use super::snapshots::VolsnapSnapshotsIterator;
use super::volume::VolsnapVolume;

/// Volume Shadow Snapshot (volsnap) shadow storage.
pub struct VolsnapShadowStorage {
    /// Snapshot (or source) volume.
    snapshot_volume: VolsnapVolume,

    /// Storage volume.
    storage_volume: Option<VolsnapVolume>,

    /// Bytes per sector.
    bytes_per_sector: u16,
}

impl VolsnapShadowStorage {
    /// Creates a new shadow storage.
    pub fn new() -> Self {
        Self {
            snapshot_volume: VolsnapVolume::new(),
            storage_volume: None,
            bytes_per_sector: 0,
        }
    }

    /// Retrieves the storage volume identifier.
    pub fn get_storage_volume_identifier(&self) -> &Uuid {
        match &self.storage_volume {
            Some(storage_volume) => &storage_volume.volume_identifier,
            None => &self.snapshot_volume.storage_volume_identifier,
        }
    }

    /// Retrieves the (snapshot) volume identifier.
    pub fn get_volume_identifier(&self) -> &Uuid {
        &self.snapshot_volume.volume_identifier
    }

    /// Retrieves the number of snapshots.
    pub fn get_number_of_snapshots(&self) -> usize {
        self.snapshot_volume.shadow_copies.len()
    }

    /// Retrieves a snapshot by index.
    pub fn get_snapshot_by_index(
        &self,
        snapshot_index: usize,
    ) -> Result<VolsnapSnapshot, ErrorTrace> {
        match self
            .snapshot_volume
            .shadow_copies
            .get_key_value_by_index(snapshot_index)
        {
            Some((identifier, shadow_copy)) => match self.snapshot_volume.data_stream.as_ref() {
                Some(data_stream) => {
                    let mut snapshot: VolsnapSnapshot = VolsnapSnapshot::new(
                        data_stream,
                        self.bytes_per_sector,
                        identifier,
                        shadow_copy,
                    );
                    match snapshot.open() {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to open snapshot");
                            return Err(error);
                        }
                    }
                    Ok(snapshot)
                }
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
        VolsnapSnapshotsIterator::new(self, self.snapshot_volume.shadow_copies.len())
    }

    /// Opens a shadow storage.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_names: &[PathComponent],
    ) -> Result<(), ErrorTrace> {
        let file_name: &PathComponent = match file_names.get(0) {
            Some(path_component) => path_component,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing snapshot volume file name"
                ));
            }
        };
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open file: {}", file_name)
                );
                return Err(error);
            }
        };
        match self.snapshot_volume.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read snapshot volume");
                return Err(error);
            }
        }
        // TODO: read NTFS boot record to determine bytes per sector?
        self.bytes_per_sector = 512;

        if self.snapshot_volume.volume_identifier != self.snapshot_volume.storage_volume_identifier
        {
            // TODO: look for the storage volume in the remaining file names
            // TODO: read the stores from the storage volume
            todo!();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{MAIN_SEPARATOR_STR, PathBuf};
    use std::sync::{Arc, RwLock};

    use keramics_core::open_os_data_stream;

    use crate::RangeStream;
    use crate::file_resolver::FileResolver;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;

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

    fn get_shadow_storage() -> Result<VolsnapShadowStorage, ErrorTrace> {
        let mut shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        let path_string: String = get_test_data_path("volsnap");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

        let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];

        shadow_storage.open(&file_resolver, &file_names)?;

        Ok(shadow_storage)
    }

    #[test]
    fn test_get_storage_volume_identifier() -> Result<(), ErrorTrace> {
        let shadow_storage: VolsnapShadowStorage = get_shadow_storage()?;

        let identifier: &Uuid = shadow_storage.get_storage_volume_identifier();
        assert_eq!(
            identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }

    #[test]
    fn test_get_volume_identifier() -> Result<(), ErrorTrace> {
        let shadow_storage: VolsnapShadowStorage = get_shadow_storage()?;

        let identifier: &Uuid = shadow_storage.get_volume_identifier();
        assert_eq!(
            identifier.to_string(),
            "9f178190-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_snapshots() -> Result<(), ErrorTrace> {
        let shadow_storage: VolsnapShadowStorage = get_shadow_storage()?;

        let number_of_snapshots: usize = shadow_storage.get_number_of_snapshots();
        assert_eq!(number_of_snapshots, 2);

        Ok(())
    }

    // TODO: add tests for get_snapshot_by_index
    // TODO: add tests for snapshots

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        let path_string: String = get_test_data_path("volsnap");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

        let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];

        shadow_storage.open(&file_resolver, &file_names)?;

        // TODO

        Ok(())
    }
}
