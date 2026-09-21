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
use keramics_formats::volsnap::{VolsnapShadowStorage, VolsnapSnapshot};
use keramics_formats::{FileResolverReference, Path, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::VolsnapFileEntry;

/// Volume Shadow Snapshot (volsnap) file entry.
pub struct VolsnapFileSystem {
    /// Shadow storage.
    shadow_storage: Arc<VolsnapShadowStorage>,

    /// Number of snapshots.
    number_of_snapshots: usize,
}

impl VolsnapFileSystem {
    pub const PATH_PREFIX: &'static str = "/volsnap";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            shadow_storage: Arc::new(VolsnapShadowStorage::new()),
            number_of_snapshots: 0,
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
                let snapshot_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "volsnap") {
                        Some(snapshot_index) => snapshot_index,
                        None => return false,
                    };
                !(snapshot_index == 0 || snapshot_index > self.number_of_snapshots)
            }
            None => !path.is_empty(),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<VolsnapFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut snapshot_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "volsnap") {
                        Some(snapshot_index) => snapshot_index,
                        None => return Ok(None),
                    };
                if snapshot_index == 0 || snapshot_index > self.number_of_snapshots {
                    return Ok(None);
                }
                snapshot_index -= 1;

                let volsnap_snapshot: VolsnapSnapshot =
                    match self.shadow_storage.get_snapshot_by_index(snapshot_index) {
                        Ok(volsnap_snapshot) => volsnap_snapshot,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to retrieve snapshot: {}", snapshot_index)
                            );
                            return Err(error);
                        }
                    };
                Ok(Some(VolsnapFileEntry::Volume {
                    name_index: snapshot_index,
                    snapshot: volsnap_snapshot,
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
    pub fn get_root_file_entry(&self) -> VolsnapFileEntry {
        VolsnapFileEntry::Root {
            shadow_storage: self.shadow_storage.clone(),
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

        match Arc::get_mut(&mut self.shadow_storage) {
            Some(shadow_storage) => {
                match Self::open_shadow_storage(shadow_storage, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open shadow storage"
                        );
                        return Err(error);
                    }
                }
                self.number_of_snapshots = shadow_storage.get_number_of_snapshots();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to shadow storage"
                ));
            }
        }
        Ok(())
    }

    /// Opens a volsnap shadow storage.
    pub(crate) fn open_shadow_storage(
        shadow_storage: &mut VolsnapShadowStorage,
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
        let file_names: [PathComponent; 1] = [file_name.clone()];

        match shadow_storage.open(&file_resolver, &file_names) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open volsnap shadow storage"
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

    fn get_file_system() -> Result<VolsnapFileSystem, ErrorTrace> {
        let mut volsnap_file_system: VolsnapFileSystem = VolsnapFileSystem::new();

        let os_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("volsnap/volsnap.vhd");
        let os_vfs_location: VfsLocation = VfsLocation::from(&path_string);

        let mut vhd_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Vhd));
        match Arc::get_mut(&mut vhd_file_system) {
            Some(file_system) => file_system.open(Some(&os_file_system), &os_vfs_location)?,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to VHD file system"
                ));
            }
        };
        let vhd_vfs_location: VfsLocation =
            os_vfs_location.new_with_layer(&VfsType::Vhd, Path::from("/vhd1"));

        let mut mbr_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Mbr));
        match Arc::get_mut(&mut mbr_file_system) {
            Some(file_system) => file_system.open(Some(&vhd_file_system), &vhd_vfs_location)?,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to MBR file system"
                ));
            }
        };
        let mbr_vfs_location: VfsLocation =
            vhd_vfs_location.new_with_layer(&VfsType::Mbr, Path::from("/mbr1"));

        volsnap_file_system.open(Some(&mbr_file_system), &mbr_vfs_location)?;

        Ok(volsnap_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let volsnap_file_system: VolsnapFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/volsnap1");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/volsnap99");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("volsnap1");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/volsnap1/bogus1");
        let result: bool = volsnap_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let volsnap_file_system: VolsnapFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<VolsnapFileEntry> = volsnap_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let volsnap_file_entry: VolsnapFileEntry = result.unwrap();

        let name: PathComponent = volsnap_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = volsnap_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/volsnap1");
        let result: Option<VolsnapFileEntry> = volsnap_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let volsnap_file_entry: VolsnapFileEntry = result.unwrap();

        let name: PathComponent = volsnap_file_entry.get_name();
        assert_eq!(name, PathComponent::from("volsnap1"));

        let file_type: VfsFileType = volsnap_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<VolsnapFileEntry> = volsnap_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let volsnap_file_system: VolsnapFileSystem = get_file_system()?;

        let volsnap_file_entry: VolsnapFileEntry = volsnap_file_system.get_root_file_entry();
        assert!(matches!(volsnap_file_entry, VolsnapFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut volsnap_file_system: VolsnapFileSystem = VolsnapFileSystem::new();

        let os_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("volsnap/volsnap.vhd");
        let os_vfs_location: VfsLocation = VfsLocation::from(&path_string);

        let mut vhd_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Vhd));
        match Arc::get_mut(&mut vhd_file_system) {
            Some(file_system) => file_system.open(Some(&os_file_system), &os_vfs_location)?,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to VHD file system"
                ));
            }
        };
        let vhd_vfs_location: VfsLocation =
            os_vfs_location.new_with_layer(&VfsType::Vhd, Path::from("/vhd1"));

        let mut mbr_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Mbr));
        match Arc::get_mut(&mut mbr_file_system) {
            Some(file_system) => file_system.open(Some(&vhd_file_system), &vhd_vfs_location)?,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to MBR file system"
                ));
            }
        };
        let mbr_vfs_location: VfsLocation =
            vhd_vfs_location.new_with_layer(&VfsType::Mbr, Path::from("/mbr1"));

        volsnap_file_system.open(Some(&mbr_file_system), &mbr_vfs_location)?;
        assert_eq!(volsnap_file_system.number_of_snapshots, 2);

        Ok(())
    }

    // TODO: add tests for open_shadow_storage
}
