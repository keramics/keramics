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

use std::io;
use std::sync::{Arc, RwLock};

use core::FileResolverReference;
use formats::vhdx::{VhdxImage, VhdxLayer};

use crate::file_resolver::open_vfs_file_resolver;
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

    /// Creates a new file entry.
    pub fn new() -> Self {
        Self {
            image: Arc::new(VhdxImage::new()),
            number_of_layers: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        let location: &String = match path {
            VfsPath::Vhdx { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            return Ok(true);
        }
        match self.get_layer_index_by_path(&location) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, path: &VfsPath) -> io::Result<Option<VhdxFileEntry>> {
        let location: &String = match path {
            VfsPath::Vhdx { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            let vhdx_file_entry: VhdxFileEntry = self.get_root_file_entry()?;

            return Ok(Some(vhdx_file_entry));
        }
        match self.get_layer_index_by_path(location) {
            Ok(layer_index) => {
                let vhdx_layer: VhdxLayer = self.image.get_layer_by_index(layer_index)?;

                Ok(Some(VhdxFileEntry::Layer {
                    index: layer_index,
                    layer: Arc::new(RwLock::new(vhdx_layer)),
                }))
            }
            Err(_) => Ok(None),
        }
    }

    /// Retrieves the layer index with the specific location.
    // TODO: return None instead of Err
    fn get_layer_index_by_path(&self, location: &String) -> io::Result<usize> {
        if !location.starts_with(VhdxFileSystem::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        };
        let layer_index: usize = match location[5..].parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported path: {}", location),
                ))
            }
        };
        if layer_index == 0 || layer_index > self.number_of_layers {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        Ok(layer_index - 1)
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<VhdxFileEntry> {
        Ok(VhdxFileEntry::Root {
            image: self.image.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        path: &VfsPath,
    ) -> io::Result<()> {
        let file_resolver: FileResolverReference = match parent_file_system {
            Some(file_system) => open_vfs_file_resolver(file_system, path.parent_directory())?,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                ))
            }
        };
        match Arc::get_mut(&mut self.image) {
            Some(image) => {
                image.open(&file_resolver, path.get_file_name())?;

                self.number_of_layers = image.get_number_of_layers();
            }
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing image")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::{VfsFileType, VfsPathType};
    use crate::file_system::VfsFileSystem;

    fn get_file_system() -> io::Result<(VhdxFileSystem, VfsPath)> {
        let mut vhdx_file_system: VhdxFileSystem = VhdxFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        vhdx_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        Ok((vhdx_file_system, parent_vfs_path))
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let (vhdx_file_system, parent_vfs_path): (VhdxFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx1");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/bogus1");
        let result: bool = vhdx_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let (vhdx_file_system, parent_vfs_path): (VhdxFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx1");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let vhdx_file_entry: VhdxFileEntry = result.unwrap();

        let name: Option<String> = vhdx_file_entry.get_name();
        assert_eq!(name, Some("vhdx1".to_string()));

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let vhdx_file_entry: VhdxFileEntry = result.unwrap();

        let name: Option<String> = vhdx_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Vhdx, "/bogus1");
        let result: Option<VhdxFileEntry> = vhdx_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn get_layer_index_by_path() -> io::Result<()> {
        let (vhdx_file_system, _): (VhdxFileSystem, VfsPath) = get_file_system()?;

        let path: String = "/vhdx1".to_string();
        let layer_index: usize = vhdx_file_system.get_layer_index_by_path(&path)?;
        assert_eq!(layer_index, 0);

        let path: String = "/".to_string();
        let result = vhdx_file_system.get_layer_index_by_path(&path);
        assert!(result.is_err());

        let path: String = "/vhdx99".to_string();
        let result = vhdx_file_system.get_layer_index_by_path(&path);
        assert!(result.is_err());

        let path: String = "/bogus1".to_string();
        let result = vhdx_file_system.get_layer_index_by_path(&path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> io::Result<()> {
        let (vhdx_file_system, _): (VhdxFileSystem, VfsPath) = get_file_system()?;

        let vhdx_file_entry: VhdxFileEntry = vhdx_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = vhdx_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vhdx_file_system: VhdxFileSystem = VhdxFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        vhdx_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        assert_eq!(vhdx_file_system.number_of_layers, 2);

        Ok(())
    }
}
