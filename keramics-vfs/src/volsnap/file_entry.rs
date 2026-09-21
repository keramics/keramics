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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::PathComponent;
use keramics_formats::volsnap::{VolsnapShadowStorage, VolsnapSnapshot};
use keramics_types::Uuid;

use crate::enums::VfsFileType;

/// Volume Shadow Snapshot (volsnap) file entry.
pub enum VolsnapFileEntry {
    /// Root file entry.
    Root {
        /// Shadow storage.
        shadow_storage: Arc<VolsnapShadowStorage>,
    },

    /// Volume file entry.
    Volume {
        /// File name index.
        name_index: usize,

        /// Snapshot.
        snapshot: VolsnapSnapshot,
    },
}

impl VolsnapFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            VolsnapFileEntry::Root { .. } => Ok(None),
            VolsnapFileEntry::Volume { snapshot, .. } => Ok(snapshot.get_data_stream()),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VolsnapFileEntry::Root { .. } => VfsFileType::Directory,
            VolsnapFileEntry::Volume { .. } => VfsFileType::File,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<&Uuid> {
        match self {
            VolsnapFileEntry::Root { .. } => None,
            VolsnapFileEntry::Volume { snapshot, .. } => Some(snapshot.get_store_identifier()),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            VolsnapFileEntry::Root { .. } => PathComponent::Root,
            VolsnapFileEntry::Volume { name_index, .. } => {
                PathComponent::from(format!("volsnap{}", name_index + 1))
            }
        }
    }

    /// Retrieves the volume number.
    pub fn get_volume_number(&self) -> Option<usize> {
        match self {
            VolsnapFileEntry::Root { .. } => None,
            VolsnapFileEntry::Volume { snapshot, .. } => match snapshot.get_store_index() {
                Some(store_index) => Some(*store_index + 1),
                None => None,
            },
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VolsnapFileEntry::Root { .. } => 0,
            VolsnapFileEntry::Volume { snapshot, .. } => match snapshot.get_size() {
                Some(size) => *size,
                None => 0,
            },
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            VolsnapFileEntry::Root { shadow_storage } => shadow_storage.get_number_of_snapshots(),
            VolsnapFileEntry::Volume { .. } => 0,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<VolsnapFileEntry, ErrorTrace> {
        match self {
            VolsnapFileEntry::Root { shadow_storage } => {
                match shadow_storage.get_snapshot_by_index(sub_file_entry_index) {
                    Ok(volsnap_snapshot) => Ok(VolsnapFileEntry::Volume {
                        name_index: sub_file_entry_index,
                        snapshot: volsnap_snapshot,
                    }),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve snapshot: {}", sub_file_entry_index)
                        );
                        Err(error)
                    }
                }
            }
            VolsnapFileEntry::Volume { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            VolsnapFileEntry::Root { .. } => true,
            VolsnapFileEntry::Volume { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{MAIN_SEPARATOR_STR, PathBuf};
    use std::sync::RwLock;

    use keramics_core::{DataStreamReference, open_os_data_stream};
    use keramics_formats::vhd::VhdFile;
    use keramics_formats::{FileResolver, FileResolverReference, PathComponent, RangeStream};

    use crate::tests::get_test_data_path;

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

    fn get_shadow_storage() -> Result<Arc<VolsnapShadowStorage>, ErrorTrace> {
        let mut shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        let path_string: String = get_test_data_path("volsnap");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

        let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];
        shadow_storage.open(&file_resolver, &file_names)?;

        Ok(Arc::new(shadow_storage))
    }

    fn get_root_file_entry(volsnap_shadow_storage: &Arc<VolsnapShadowStorage>) -> VolsnapFileEntry {
        VolsnapFileEntry::Root {
            shadow_storage: volsnap_shadow_storage.clone(),
        }
    }

    fn get_volume_file_entry(
        volsnap_shadow_storage: &Arc<VolsnapShadowStorage>,
    ) -> Result<VolsnapFileEntry, ErrorTrace> {
        let volsnap_snapshot: VolsnapSnapshot = volsnap_shadow_storage.get_snapshot_by_index(0)?;

        Ok(VolsnapFileEntry::Volume {
            name_index: 0,
            snapshot: volsnap_snapshot,
        })
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let result: Option<&Uuid> = file_entry.get_identifier();
        assert!(result.is_none());

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let identifier: &Uuid = file_entry.get_identifier().unwrap();
        assert_eq!(
            identifier.to_string(),
            "9f17819b-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("volsnap1"));

        Ok(())
    }

    #[test]
    fn test_get_volume_number() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, None);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, Some(1));
        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 133103616);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 2);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);

        let sub_file_entry: VolsnapFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("volsnap1"));

        let result: Result<VolsnapFileEntry, ErrorTrace> =
            file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: Arc<VolsnapShadowStorage> = get_shadow_storage()?;

        let file_entry: VolsnapFileEntry = get_root_file_entry(&volsnap_shadow_storage);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: VolsnapFileEntry = get_volume_file_entry(&volsnap_shadow_storage)?;
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
