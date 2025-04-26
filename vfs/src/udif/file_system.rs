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

use formats::udif::UdifFile;

use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::UdifFileEntry;

/// Universal Disk Image Format (UDIF) storage media image file system.
pub struct UdifFileSystem {
    /// File.
    file: Arc<RwLock<UdifFile>>,

    /// Number of layers.
    number_of_layers: usize,
}

impl UdifFileSystem {
    pub const PATH_PREFIX: &'static str = "/udif";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            file: Arc::new(RwLock::new(UdifFile::new())),
            number_of_layers: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        let location: &String = match path {
            VfsPath::Udif { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            return Ok(true);
        } else if location == "/udif1" {
            return Ok(true);
        }
        Ok(false)
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, path: &VfsPath) -> io::Result<Option<UdifFileEntry>> {
        let location: &String = match path {
            VfsPath::Udif { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            let udif_file_entry: UdifFileEntry = self.get_root_file_entry()?;

            return Ok(Some(udif_file_entry));
        } else if location == "/udif1" {
            return Ok(Some(UdifFileEntry::Layer {
                file: self.file.clone(),
            }));
        }
        Ok(None)
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<UdifFileEntry> {
        Ok(UdifFileEntry::Root {
            file: self.file.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        path: &VfsPath,
    ) -> io::Result<()> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                ))
            }
        };
        match file_system.get_data_stream_by_path_and_name(path, None)? {
            Some(data_stream) => match self.file.write() {
                Ok(mut file) => {
                    file.read_data_stream(&data_stream)?;

                    self.number_of_layers = 1;
                }
                Err(error) => return Err(core::error_to_io_error!(error)),
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::{VfsFileType, VfsPathType};
    use crate::file_system::VfsFileSystem;

    fn get_file_system() -> io::Result<(UdifFileSystem, VfsPath)> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        Ok((udif_file_system, parent_vfs_path))
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let (udif_file_system, parent_vfs_path): (UdifFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/udif1");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/bogus1");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let (udif_file_system, parent_vfs_path): (UdifFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: Option<String> = udif_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/udif1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: Option<String> = udif_file_entry.get_name();
        assert_eq!(name, Some("udif1".to_string()));

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Udif, "/bogus1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> io::Result<()> {
        let (udif_file_system, _): (UdifFileSystem, VfsPath) = get_file_system()?;

        let udif_file_entry: UdifFileEntry = udif_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        assert_eq!(udif_file_system.number_of_layers, 1);

        Ok(())
    }
}
