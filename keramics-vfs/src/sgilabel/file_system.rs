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
use keramics_formats::sgilabel::{SgiDiskLabelPartition, SgiDiskLabelVolumeSystem};

use crate::partition::VfsPartitionFileSystem;
use crate::traits::VfsPartitionSystem;
use crate::types::VfsFileSystemReference;

/// SGI disklabel (sgilabel) file system.
pub type SgiDiskLabelFileSystem =
    VfsPartitionFileSystem<SgiDiskLabelPartition, SgiDiskLabelVolumeSystem>;

impl VfsPartitionSystem for SgiDiskLabelVolumeSystem {
    const PATH_PREFIX: &'static str = "/sgilabel";

    /// Creates a new partition (volume) system.
    fn new() -> Self {
        SgiDiskLabelVolumeSystem::new()
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
                    "Unable to read sgilabel volume system from data stream"
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

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::VfsLocation;
    use crate::sgilabel::SgiDiskLabelFileEntry;
    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<SgiDiskLabelFileSystem, ErrorTrace> {
        let mut sgilabel_file_system: SgiDiskLabelFileSystem = SgiDiskLabelFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("sgilabel/sgilabel.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        sgilabel_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(sgilabel_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let sgilabel_file_system: SgiDiskLabelFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sgilabel1");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sgilabel99");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("sgilabel1");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/sgilabel1/bogus1");
        let result: bool = sgilabel_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let sgilabel_file_system: SgiDiskLabelFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<SgiDiskLabelFileEntry> =
            sgilabel_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let sgilabel_file_entry: SgiDiskLabelFileEntry = result.unwrap();

        let name: PathComponent = sgilabel_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = sgilabel_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/sgilabel1");
        let sgilabel_file_entry: SgiDiskLabelFileEntry =
            sgilabel_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = sgilabel_file_entry.get_name();
        assert_eq!(name, PathComponent::from("sgilabel1"));

        let file_type: VfsFileType = sgilabel_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<SgiDiskLabelFileEntry> =
            sgilabel_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let sgilabel_file_system: SgiDiskLabelFileSystem = get_file_system()?;

        let sgilabel_file_entry: SgiDiskLabelFileEntry = sgilabel_file_system.get_root_file_entry();
        assert!(matches!(
            sgilabel_file_entry,
            SgiDiskLabelFileEntry::Root { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut sgilabel_file_system: SgiDiskLabelFileSystem = SgiDiskLabelFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("sgilabel/sgilabel.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        sgilabel_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(sgilabel_file_system.number_of_partitions, 1);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
