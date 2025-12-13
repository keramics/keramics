/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::Path;
use keramics_formats::mbr::{MbrPartition, MbrVolumeSystem};

use crate::file_system::VfsFileSystem;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::MbrFileEntry;

/// Master Boot Record (MBR) file system.
pub struct MbrFileSystem {
    /// Volume system.
    volume_system: Arc<MbrVolumeSystem>,

    /// Number of partitions.
    number_of_partitions: usize,
}

impl MbrFileSystem {
    pub const PATH_PREFIX: &'static str = "/mbr";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            volume_system: Arc::new(MbrVolumeSystem::new()),
            number_of_partitions: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &Path) -> bool {
        if path.is_relative() {
            return false;
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return false;
                }
                let partition_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "mbr") {
                        Some(partition_index) => partition_index,
                        None => return false,
                    };
                if partition_index == 0 || partition_index > self.number_of_partitions {
                    false
                } else {
                    true
                }
            }
            None => {
                if path.is_empty() {
                    false
                } else {
                    true
                }
            }
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<MbrFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut partition_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "mbr") {
                        Some(partition_index) => partition_index,
                        None => return Ok(None),
                    };
                if partition_index == 0 || partition_index > self.number_of_partitions {
                    return Ok(None);
                }
                partition_index -= 1;

                let mbr_partition: MbrPartition =
                    match self.volume_system.get_partition_by_index(partition_index) {
                        Ok(mbr_partition) => mbr_partition,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to retrieve MBR partition: {}", partition_index)
                            );
                            return Err(error);
                        }
                    };
                let partition_size: u64 = mbr_partition.size;

                Ok(Some(MbrFileEntry::Partition {
                    index: partition_index,
                    partition: Arc::new(RwLock::new(mbr_partition)),
                    size: partition_size,
                }))
            }
            None => {
                if path.is_empty() {
                    return Ok(None);
                }
                Ok(Some(self.get_root_file_entry()))
            }
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> MbrFileEntry {
        MbrFileEntry::Root {
            volume_system: self.volume_system.clone(),
        }
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let path: &Path = vfs_location.get_path();

        match Arc::get_mut(&mut self.volume_system) {
            Some(volume_system) => {
                match Self::open_volume_system(volume_system, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open MBR volume system"
                        );
                        return Err(error);
                    }
                }
                self.number_of_partitions = volume_system.get_number_of_partitions();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to MBR volume system"
                ));
            }
        }
        Ok(())
    }

    /// Opens a MBR volume system.
    pub(crate) fn open_volume_system(
        volume_system: &mut MbrVolumeSystem,
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
            VfsFileSystem::Qcow(qcow_file_system) => {
                Ok(Some(qcow_file_system.get_bytes_per_sector()?))
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
                match volume_system.set_bytes_per_sector(bytes_per_sector) {
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
        match volume_system.read_data_stream(&data_stream) {
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
    use crate::location::new_os_vfs_location;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<MbrFileSystem, ErrorTrace> {
        let mut mbr_file_system: MbrFileSystem = MbrFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let parent_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
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
        let parent_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        mbr_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(mbr_file_system.number_of_partitions, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
