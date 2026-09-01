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
use keramics_formats::Path;
use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};

use crate::partition::VfsPartitionFileSystem;
use crate::traits::VfsPartitionSystem;
use crate::types::VfsFileSystemReference;

/// Apple Partition Map (APM) file system.
pub type ApmFileSystem = VfsPartitionFileSystem<ApmPartition, ApmVolumeSystem>;

impl VfsPartitionSystem for ApmVolumeSystem {
    const PATH_PREFIX: &'static str = "/apm";

    /// Creates a new partition (volume) system.
    fn new() -> Self {
        ApmVolumeSystem::new()
    }

    /// Opens the partition system from VFS.
    fn open_from_vfs(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = match file_system.get_data_stream_by_path(path) {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                return Err(error);
            }
        };
        match self.read_data_stream(&data_stream) {
            Ok(()) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read APM volume system from data stream"
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_formats::PathComponent;

    use crate::apm::ApmFileEntry;
    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::VfsLocation;
    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<ApmFileSystem, ErrorTrace> {
        let mut apm_file_system: ApmFileSystem = ApmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        apm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(apm_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/apm1");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/apm99");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("apm1");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/apm1/bogus1");
        let result: bool = apm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<ApmFileEntry> = apm_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let apm_file_entry: ApmFileEntry = result.unwrap();

        let name: PathComponent = apm_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = apm_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/apm1");
        let apm_file_entry: ApmFileEntry = apm_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = apm_file_entry.get_name();
        assert_eq!(name, PathComponent::from("apm1"));

        let file_type: VfsFileType = apm_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<ApmFileEntry> = apm_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let apm_file_entry: ApmFileEntry = apm_file_system.get_root_file_entry();
        assert!(matches!(apm_file_entry, ApmFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut apm_file_system: ApmFileSystem = ApmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        apm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(apm_file_system.number_of_partitions, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
