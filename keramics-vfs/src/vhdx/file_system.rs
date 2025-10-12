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

use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_formats::vhdx::VhdxImage;
use keramics_formats::{FileResolverReference, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::VhdxFileEntry;

/// Virtual Hard Disk version 2 (VHDX) storage media image file system.
pub struct VhdxFileSystem {
    /// Storage media image.
    image: Arc<VhdxImage>,

    /// Number of layers.
    number_of_layers: usize,
}

impl VhdxFileSystem {
    pub const PATH_PREFIX: &'static str = "/vhdx";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(VhdxImage::new()),
            number_of_layers: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path_components) => {
                let number_of_components: usize = string_path_components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(false);
                }
                if string_path_components[0] != "" {
                    return Ok(false);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    return Ok(true);
                }
                match self.get_layer_index(&string_path_components[1]) {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<VhdxFileEntry>, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path_components) => {
                let number_of_components: usize = string_path_components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(None);
                }
                if string_path_components[0] != "" {
                    return Ok(None);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    let vhdx_file_entry: VhdxFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(vhdx_file_entry));
                }
                let layer_index: usize = match self.get_layer_index(&string_path_components[1]) {
                    Some(layer_index) => layer_index,
                    None => return Ok(None),
                };
                match self.image.get_layer_by_index(layer_index) {
                    Ok(vhdx_layer) => Ok(Some(VhdxFileEntry::Layer {
                        index: layer_index,
                        layer: vhdx_layer.clone(),
                    })),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve VHDX layer: {}", layer_index)
                        );
                        return Err(error);
                    }
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the layer index.
    fn get_layer_index(&self, file_name: &String) -> Option<usize> {
        if !file_name.starts_with("vhdx") {
            return None;
        }
        match file_name[4..].parse::<usize>() {
            Ok(layer_index) => {
                if layer_index > 0 && layer_index <= self.number_of_layers {
                    Some(layer_index - 1)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<VhdxFileEntry, ErrorTrace> {
        Ok(VhdxFileEntry::Root {
            image: self.image.clone(),
        })
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
        let vfs_path: &VfsPath = vfs_location.get_path();

        match Arc::get_mut(&mut self.image) {
            Some(image) => {
                match Self::open_image(image, file_system, vfs_path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                        return Err(error);
                    }
                }
                self.number_of_layers = image.get_number_of_layers();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to VHDX image"
                ));
            }
        }
        Ok(())
    }

    /// Opens a VHDX image.
    pub(crate) fn open_image(
        image: &mut VhdxImage,
        file_system: &VfsFileSystemReference,
        vfs_path: &VfsPath,
    ) -> Result<(), ErrorTrace> {
        let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
        let file_resolver: FileResolverReference =
            match new_vfs_file_resolver(file_system, parent_vfs_path) {
                Ok(file_resolver) => file_resolver,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create VFS file resolver"
                    );
                    return Err(error);
                }
            };
        let file_name: PathComponent = match vfs_path.get_file_name() {
            Some(file_name) => file_name,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve file name"
                ));
            }
        };
        match image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::new_os_vfs_location;

    fn get_file_system() -> Result<VhdxFileSystem, ErrorTrace> {
        let mut vhdx_file_system: VhdxFileSystem = VhdxFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/vhdx/ntfs-differential.vhdx");
        vhdx_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(vhdx_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let vhdx_file_system: VhdxFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/vhdx1");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/bogus1");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let vhdx_file_system: VhdxFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let vhdx_file_entry: VhdxFileEntry = result.unwrap();

        let name: Option<String> = vhdx_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/vhdx1");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let vhdx_file_entry: VhdxFileEntry = result.unwrap();

        let name: Option<String> = vhdx_file_entry.get_name();
        assert_eq!(name, Some(String::from("vhdx1")));

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Vhdx, "/bogus1");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_layer_index() -> Result<(), ErrorTrace> {
        let vhdx_file_system: VhdxFileSystem = get_file_system()?;

        let file_name: String = String::from("vhdx1");
        let layer_index: Option<usize> = vhdx_file_system.get_layer_index(&file_name);
        assert_eq!(layer_index, Some(0));

        let file_name: String = String::from("vhdx99");
        let layer_index: Option<usize> = vhdx_file_system.get_layer_index(&file_name);
        assert!(layer_index.is_none());

        let file_name: String = String::from("bogus1");
        let layer_index: Option<usize> = vhdx_file_system.get_layer_index(&file_name);
        assert!(layer_index.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let vhdx_file_system: VhdxFileSystem = get_file_system()?;

        let vhdx_file_entry: VhdxFileEntry = vhdx_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut vhdx_file_system: VhdxFileSystem = VhdxFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/vhdx/ntfs-differential.vhdx");
        vhdx_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(vhdx_file_system.number_of_layers, 2);

        Ok(())
    }

    // TODO: add tests for open_image
}
