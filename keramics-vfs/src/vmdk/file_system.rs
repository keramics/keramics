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

use keramics_core::ErrorTrace;
use keramics_formats::vmdk::{VmdkImage, VmdkImageLayer};
use keramics_formats::{FileResolverReference, Path, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::VmdkFileEntry;

/// VMware Virtual Disk (VMDK) storage media image file system.
pub struct VmdkFileSystem {
    /// Image.
    image: Arc<VmdkImage>,

    /// Number of layers.
    number_of_layers: usize,
}

impl VmdkFileSystem {
    pub const PATH_PREFIX: &'static str = "/vmdk";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(VmdkImage::new()),
            number_of_layers: 0,
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
                if path_component != "vmdk1" {
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

    /// Retrieves the bytes per sector.
    pub(crate) fn get_bytes_per_sector(&self) -> Result<u32, ErrorTrace> {
        Ok(self.image.bytes_per_sector as u32)
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<VmdkFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut layer_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, "vmdk") {
                        Some(layer_index) => layer_index,
                        None => return Ok(None),
                    };
                if layer_index == 0 || layer_index > self.number_of_layers {
                    return Ok(None);
                }
                layer_index -= 1;

                let image_layer: Arc<RwLock<VmdkImageLayer>> =
                    match self.image.get_layer_by_index(layer_index) {
                        Ok(image_layer) => image_layer,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to retrieve image layer: {}", layer_index)
                            );
                            return Err(error);
                        }
                    };
                let media_size: u64 = match image_layer.read() {
                    Ok(vmdk_file) => vmdk_file.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on image layer",
                            error
                        ));
                    }
                };
                Ok(Some(VmdkFileEntry::Layer {
                    index: layer_index,
                    layer: image_layer.clone(),
                    size: media_size,
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
    pub fn get_root_file_entry(&self) -> VmdkFileEntry {
        VmdkFileEntry::Root {
            image: self.image.clone(),
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

        match Arc::get_mut(&mut self.image) {
            Some(image) => {
                match Self::open_image(image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                        return Err(error);
                    }
                }
                self.number_of_layers = image.get_number_of_layers();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to VMDK image"
                ));
            }
        }
        Ok(())
    }

    /// Opens an VMDK image.
    pub(crate) fn open_image(
        image: &mut VmdkImage,
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
        match image.open(&file_resolver, file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
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

    fn get_file_system() -> Result<VmdkFileSystem, ErrorTrace> {
        let mut vmdk_file_system: VmdkFileSystem = VmdkFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let parent_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vmdk_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(vmdk_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let vmdk_file_system: VmdkFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/vmdk1");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/vmdk99");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("vmdk1");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/vmdk1/bogus1");
        let result: bool = vmdk_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    // TODO: add tests for get_bytes_per_sector

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let vmdk_file_system: VmdkFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<VmdkFileEntry> = vmdk_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let vmdk_file_entry: VmdkFileEntry = result.unwrap();

        let name: PathComponent = vmdk_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = vmdk_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/vmdk1");
        let result: Option<VmdkFileEntry> = vmdk_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let vmdk_file_entry: VmdkFileEntry = result.unwrap();

        let name: PathComponent = vmdk_file_entry.get_name();
        assert_eq!(name, PathComponent::from("vmdk1"));

        let file_type: VfsFileType = vmdk_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus");
        let result: Option<VmdkFileEntry> = vmdk_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let vmdk_file_system: VmdkFileSystem = get_file_system()?;

        let vmdk_file_entry: VmdkFileEntry = vmdk_file_system.get_root_file_entry();
        assert!(matches!(vmdk_file_entry, VmdkFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut vmdk_file_system: VmdkFileSystem = VmdkFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let parent_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vmdk_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(vmdk_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_image
}
