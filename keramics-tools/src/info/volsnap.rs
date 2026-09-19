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

use std::fmt;
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::volsnap::{VolsnapShadowStorage, VolsnapSnapshot};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

use crate::formatters::ByteSize;

use super::windows::FiletimeDateTimeInfo;

/// Volume Shadow Snapshot (volsnap) shadow storage information.
struct VolsnapShadowStorageInfo<'a> {
    /// Shadow storage.
    shadow_storage: &'a VolsnapShadowStorage,
}

impl<'a> VolsnapShadowStorageInfo<'a> {
    /// Creates new shadow storage information.
    fn new(shadow_storage: &'a VolsnapShadowStorage) -> Self {
        Self { shadow_storage }
    }
}

impl<'a> fmt::Display for VolsnapShadowStorageInfo<'a> {
    /// Formats shadow storage information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Volume Shadow Snapshot (volsnap) information:")?;

        writeln!(
            formatter,
            "    Volume identifier\t\t\t\t: {}",
            self.shadow_storage.get_volume_identifier()
        )?;
        writeln!(
            formatter,
            "    Storage volume identifier\t\t\t: {}",
            self.shadow_storage.get_storage_volume_identifier()
        )?;
        writeln!(
            formatter,
            "    Number of shadow copies\t\t\t: {}",
            self.shadow_storage.get_number_of_snapshots(),
        )?;
        writeln!(formatter)
    }
}

/// Volume Shadow Snapshot (volsnap) snapshot information.
struct VolsnapSnapshotInfo<'a> {
    /// Snapshot index.
    snapshot_index: usize,

    /// Snapshot.
    snapshot: &'a VolsnapSnapshot,
}

impl<'a> VolsnapSnapshotInfo<'a> {
    /// Creates new snapshot information.
    fn new(snapshot_index: usize, snapshot: &'a VolsnapSnapshot) -> Self {
        Self {
            snapshot_index,
            snapshot,
        }
    }
}

impl<'a> fmt::Display for VolsnapSnapshotInfo<'a> {
    /// Formats snapshot information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Shadow copy: {}:", self.snapshot_index + 1)?;

        if let Some(copy_identifier) = self.snapshot.get_copy_identifier() {
            writeln!(
                formatter,
                "        Copy identifier\t\t\t\t: {}",
                copy_identifier
            )?;
        }
        if let Some(copy_set_identifier) = self.snapshot.get_copy_set_identifier() {
            writeln!(
                formatter,
                "        Copy set identifier\t\t\t: {}",
                copy_set_identifier
            )?;
        }
        writeln!(
            formatter,
            "        Store identifier\t\t\t: {}",
            self.snapshot.get_store_identifier()
        )?;
        // TODO: print copy set identifier

        if let Some(creation_time) = self.snapshot.get_creation_time() {
            let date_time_info: FiletimeDateTimeInfo = FiletimeDateTimeInfo::new(creation_time);
            writeln!(
                formatter,
                "        Creation time\t\t\t\t: {}",
                date_time_info
            )?;
        }
        if let Some(size) = self.snapshot.get_size() {
            let byte_size: ByteSize = ByteSize::new(*size, 1024);
            writeln!(formatter, "        Size\t\t\t\t\t: {}", byte_size)?;
        }
        // TODO: print copy identifier

        writeln!(formatter)
    }
}

/// Information about Volume Shadow Snapshot (volsnap).
pub struct VolsnapInfo {}

impl VolsnapInfo {
    /// Opens a shadow storage.
    pub fn open_shadow_storage(path_buf: &PathBuf) -> Result<VolsnapShadowStorage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let file_name: PathComponent = match path_buf.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Unsupported file name"));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!("Missing file name"));
            }
        };
        let file_names: [PathComponent; 1] = [file_name];

        let mut volsnap_shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        match volsnap_shadow_storage.open(&file_resolver, &file_names) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open volsnap shadow storage."
                );
                return Err(error);
            }
        }
        Ok(volsnap_shadow_storage)
    }

    /// Prints information about a shadow storage.
    pub fn print_shadow_storage(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: VolsnapShadowStorage = match Self::open_shadow_storage(path_buf)
        {
            Ok(volsnap_shadow_storage) => volsnap_shadow_storage,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open shadow storage");
                return Err(error);
            }
        };
        let shadow_storage_info: VolsnapShadowStorageInfo =
            VolsnapShadowStorageInfo::new(&volsnap_shadow_storage);

        print!("{}", shadow_storage_info);

        for (snapshot_index, result) in volsnap_shadow_storage.snapshots().enumerate() {
            let volsnap_snapshot: VolsnapSnapshot = match result {
                Ok(volsnap_snapshot) => volsnap_snapshot,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve snapshot: {}", snapshot_index)
                    );
                    return Err(error);
                }
            };
            let snapshot_info: VolsnapSnapshotInfo =
                VolsnapSnapshotInfo::new(snapshot_index, &volsnap_snapshot);

            print!("{}", snapshot_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{MAIN_SEPARATOR_STR, PathBuf};
    use std::sync::{Arc, RwLock};

    use keramics_core::{DataStreamReference, open_os_data_stream};
    use keramics_formats::vhd::VhdFile;
    use keramics_formats::{FileResolver, RangeStream};

    use crate::assert_lines_eq;

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

        let path_buf: PathBuf = PathBuf::from("../test_data/volsnap");
        let file_resolver: FileResolverReference =
            FileResolverReference::new(Box::new(TestFileResolver::new(path_buf)));

        let file_names: [PathComponent; 1] = [PathComponent::from("volsnap.vhd")];
        shadow_storage.open(&file_resolver, &file_names)?;

        Ok(shadow_storage)
    }

    #[test]
    fn test_shadow_storage_information_fmt() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: VolsnapShadowStorage = get_shadow_storage()?;

        let test_struct: VolsnapShadowStorageInfo =
            VolsnapShadowStorageInfo::new(&volsnap_shadow_storage);

        let expected_string: &str = concat!(
            "Volume Shadow Snapshot (volsnap) information:\n",
            "    Volume identifier\t\t\t\t: 9f178190-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "    Storage volume identifier\t\t\t: 9f178190-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "    Number of shadow copies\t\t\t: 2\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_snapshot_information_fmt() -> Result<(), ErrorTrace> {
        let volsnap_shadow_storage: VolsnapShadowStorage = get_shadow_storage()?;
        let volsnap_snapshot: VolsnapSnapshot = volsnap_shadow_storage.get_snapshot_by_index(0)?;

        let test_struct: VolsnapSnapshotInfo = VolsnapSnapshotInfo::new(0, &volsnap_snapshot);

        let expected_string: &str = concat!(
            "    Shadow copy: 1:\n",
            "        Copy identifier\t\t\t\t: 54ab4fc9-3eae-4ded-8e85-6f1f864251dc\n",
            "        Copy set identifier\t\t\t: 6755c5a0-eb62-41e9-91d6-490246c61375\n",
            "        Store identifier\t\t\t: 9f17819b-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "        Creation time\t\t\t\t: 2026-09-15T12:35:23.5737520+00:00\n",
            "        Size\t\t\t\t\t: 126.9 MiB (133103616 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_shadow_storage
    // TODO: add tests for print_shadow_storage
}
