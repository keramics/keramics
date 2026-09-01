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
use keramics_formats::mbr::{MbrPartition, MbrVolumeSystem};

use crate::file_system::VfsFileSystem;
use crate::partition::VfsPartitionFileSystem;
use crate::traits::VfsPartitionSystem;
use crate::types::VfsFileSystemReference;

/// Master Boot Record (MBR) file system.
pub type MbrFileSystem = VfsPartitionFileSystem<MbrPartition, MbrVolumeSystem>;

impl VfsPartitionSystem for MbrVolumeSystem {
    const PATH_PREFIX: &'static str = "/mbr";

    /// Creates a new partition (volume) system.
    fn new() -> Self {
        MbrVolumeSystem::new()
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
        let result: Result<Option<u32>, ErrorTrace> = match file_system.as_ref() {
            VfsFileSystem::Ewf(ewf_file_system) => {
                Ok(Some(ewf_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Pdi(pdi_file_system) => {
                Ok(Some(pdi_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                Ok(Some(qcow_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::SparseBundle(sparsebundle_file_system) => {
                Ok(Some(sparsebundle_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                Ok(Some(sparseimage_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Udif(udif_file_system) => {
                Ok(Some(udif_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                Ok(Some(vhd_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                Ok(Some(vhdx_file_system.get_bytes_per_sector()?))
            }
            VfsFileSystem::Vmdk(vmdk_file_system) => {
                Ok(Some(vmdk_file_system.get_bytes_per_sector()?))
            }
            _ => Ok(None),
        };
        match result {
            Ok(Some(bytes_per_sector)) => {
                if bytes_per_sector > u16::MAX as u32 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid bytes per sector value out of bounds"
                    ));
                }
                match self.set_bytes_per_sector(bytes_per_sector as u16) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to set bytes per sector"
                        );
                        return Err(error);
                    }
                }
            }
            Ok(None) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve bytes per sector from parent file system"
                );
                return Err(error);
            }
        }
        match self.read_data_stream(&data_stream) {
            Ok(()) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read MBR volume system from data stream"
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
    use crate::mbr::MbrFileEntry;
    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<MbrFileSystem, ErrorTrace> {
        let mut mbr_file_system: MbrFileSystem = MbrFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        mbr_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(mbr_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/mbr1");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/mbr99");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("mbr1");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/mbr1/bogus1");
        let result: bool = mbr_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let mbr_file_entry: MbrFileEntry = result.unwrap();

        let name: PathComponent = mbr_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = mbr_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/mbr1");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let mbr_file_entry: MbrFileEntry = result.unwrap();

        let name: PathComponent = mbr_file_entry.get_name();
        assert_eq!(name, PathComponent::from("mbr1"));

        let file_type: VfsFileType = mbr_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let mbr_file_entry: MbrFileEntry = mbr_file_system.get_root_file_entry();
        assert!(matches!(mbr_file_entry, MbrFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut mbr_file_system: MbrFileSystem = MbrFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        mbr_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(mbr_file_system.number_of_partitions, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
