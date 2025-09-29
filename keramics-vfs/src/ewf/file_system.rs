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

use keramics_core::FileResolverReference;
use keramics_formats::ewf::EwfImage;

use crate::file_resolver::open_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::EwfFileEntry;

/// Expert Witness Compression Format (EWF) storage media image file system.
pub struct EwfFileSystem {
    /// Image.
    image: Arc<RwLock<EwfImage>>,

    /// Number of layers.
    number_of_layers: usize,
}

impl EwfFileSystem {
    pub const PATH_PREFIX: &'static str = "/ewf";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(RwLock::new(EwfImage::new())),
            number_of_layers: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> io::Result<bool> {
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
                if string_path_components[1] == "ewf1" {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported VFS path type",
            )),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, vfs_path: &VfsPath) -> io::Result<Option<EwfFileEntry>> {
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
                    let ewf_file_entry: EwfFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(ewf_file_entry));
                }
                if string_path_components[1] == "ewf1" {
                    let ewf_file_entry: EwfFileEntry = EwfFileEntry::Layer {
                        image: self.image.clone(),
                    };
                    Ok(Some(ewf_file_entry))
                } else {
                    Ok(None)
                }
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported VFS path type",
            )),
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<EwfFileEntry> {
        Ok(EwfFileEntry::Root {
            image: self.image.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> io::Result<()> {
        let vfs_path: &VfsPath = vfs_location.get_path();
        let file_resolver: FileResolverReference = match parent_file_system {
            Some(file_system) => {
                let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
                open_vfs_file_resolver(file_system, parent_vfs_path)?
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                ));
            }
        };
        match self.image.write() {
            Ok(mut image) => {
                image.open(&file_resolver, vfs_path.get_file_name())?;

                self.number_of_layers = 1;
            }
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
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

    fn get_file_system() -> io::Result<EwfFileSystem> {
        let mut ewf_file_system: EwfFileSystem = EwfFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/ewf/ext2.E01");
        ewf_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(ewf_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/ewf1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/bougs1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let ewf_file_entry: EwfFileEntry = result.unwrap();

        let name: Option<String> = ewf_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/ewf1");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let ewf_file_entry: EwfFileEntry = result.unwrap();

        let name: Option<String> = ewf_file_entry.get_name();
        assert_eq!(name, Some("ewf1".to_string()));

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ewf, "/bougs1");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> io::Result<()> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let ewf_file_entry: EwfFileEntry = ewf_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut ewf_file_system: EwfFileSystem = EwfFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/ewf/ext2.E01");
        ewf_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(ewf_file_system.number_of_layers, 1);

        Ok(())
    }
}
