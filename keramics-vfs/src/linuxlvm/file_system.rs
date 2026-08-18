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

use keramics_core::ErrorTrace;
use keramics_formats::linuxlvm::{
    LinuxLvmDataFileDescriptor, LinuxLvmVolume, LinuxLvmVolumeSystem,
};
use keramics_formats::{FileResolverReference, Path, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::LinuxLvmFileEntry;

/// Linux Logical Volume Manager (LVM) file entry.
pub struct LinuxLvmFileSystem {
    /// Volume system.
    volume_system: Arc<LinuxLvmVolumeSystem>,

    /// Number of volumes.
    number_of_volumes: usize,
}

impl LinuxLvmFileSystem {
    pub const PATH_PREFIX: &'static str = "/lvm";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            volume_system: Arc::new(LinuxLvmVolumeSystem::new()),
            number_of_volumes: 0,
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
                let volume_index: usize = match VfsPath::get_numeric_suffix(path_component, "lvm") {
                    Some(volume_index) => volume_index,
                    None => return false,
                };
                if volume_index == 0 || volume_index > self.number_of_volumes {
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
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<LinuxLvmFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut volume_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "lvm") {
                        Some(volume_index) => volume_index,
                        None => return Ok(None),
                    };
                if volume_index == 0 || volume_index > self.number_of_volumes {
                    return Ok(None);
                }
                volume_index -= 1;

                let lvm_volume: LinuxLvmVolume =
                    match self.volume_system.get_volume_by_index(volume_index) {
                        Ok(lvm_volume) => lvm_volume,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to retrieve volume: {}", volume_index)
                            );
                            return Err(error);
                        }
                    };
                Ok(Some(LinuxLvmFileEntry::Volume {
                    name_index: volume_index,
                    volume: lvm_volume,
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
    pub fn get_root_file_entry(&self) -> LinuxLvmFileEntry {
        LinuxLvmFileEntry::Root {
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
                            "Unable to open volume system"
                        );
                        return Err(error);
                    }
                }
                self.number_of_volumes = volume_system.get_number_of_volumes();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to volume system"
                ));
            }
        }
        Ok(())
    }

    /// Opens a Linux LVM volume system.
    pub(crate) fn open_volume_system(
        volume_system: &mut LinuxLvmVolumeSystem,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let parent_path: Path = path.new_with_parent_directory();

        let file_resolver: FileResolverReference =
            match new_vfs_file_resolver(file_system, parent_path) {
                Ok(file_resolver) => file_resolver,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create VFS file resolver"
                    );
                    return Err(error);
                }
            };
        let file_name: &PathComponent = match path.file_name() {
            Some(file_name) => file_name,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve file name"
                ));
            }
        };
        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(file_name.clone(), 0)];

        match volume_system.open(&file_resolver, &data_file_descriptors) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open Linux LVM volume system"
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

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<LinuxLvmFileSystem, ErrorTrace> {
        let mut lvm_file_system: LinuxLvmFileSystem = LinuxLvmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("linuxlvm/lvm2.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        lvm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(lvm_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let lvm_file_system: LinuxLvmFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/lvm1");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/lvm99");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("lvm1");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/lvm1/bogus1");
        let result: bool = lvm_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let lvm_file_system: LinuxLvmFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<LinuxLvmFileEntry> = lvm_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let lvm_file_entry: LinuxLvmFileEntry = result.unwrap();

        let name: PathComponent = lvm_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = lvm_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/lvm1");
        let result: Option<LinuxLvmFileEntry> = lvm_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let lvm_file_entry: LinuxLvmFileEntry = result.unwrap();

        let name: PathComponent = lvm_file_entry.get_name();
        assert_eq!(name, PathComponent::from("lvm1"));

        let file_type: VfsFileType = lvm_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<LinuxLvmFileEntry> = lvm_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let lvm_file_system: LinuxLvmFileSystem = get_file_system()?;

        let lvm_file_entry: LinuxLvmFileEntry = lvm_file_system.get_root_file_entry();
        assert!(matches!(lvm_file_entry, LinuxLvmFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut lvm_file_system: LinuxLvmFileSystem = LinuxLvmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("linuxlvm/lvm2.raw");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        lvm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(lvm_file_system.number_of_volumes, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
