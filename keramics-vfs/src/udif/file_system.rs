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
use keramics_formats::udif::UdifFile;

use crate::location::VfsLocation;
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
                if string_path_components[1] == "udif1" {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<UdifFileEntry>, ErrorTrace> {
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
                    let udif_file_entry: UdifFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(udif_file_entry));
                }
                if string_path_components[1] == "udif1" {
                    let udif_file_entry: UdifFileEntry = UdifFileEntry::Layer {
                        file: self.file.clone(),
                    };
                    Ok(Some(udif_file_entry))
                } else {
                    Ok(None)
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<UdifFileEntry, ErrorTrace> {
        Ok(UdifFileEntry::Root {
            file: self.file.clone(),
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

        let result: Option<DataStreamReference> =
            match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    vfs_path.to_string()
                )));
            }
        };
        match self.file.write() {
            Ok(mut file) => {
                match file.read_data_stream(&data_stream) {
                    Ok(()) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read UDIF file from data stream"
                        );
                        return Err(error);
                    }
                }
                self.number_of_layers = 1;
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on UDIF file",
                    error
                ));
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

    fn get_file_system() -> Result<UdifFileSystem, ErrorTrace> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/udif/hfsplus_zlib.dmg");
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(udif_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/udif1");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/bogus1");
        let result: bool = udif_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: Option<String> = udif_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/udif1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: Option<String> = udif_file_entry.get_name();
        assert_eq!(name, Some("udif1".to_string()));

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Udif, "/bogus1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let udif_file_entry: UdifFileEntry = udif_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/udif/hfsplus_zlib.dmg");
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(udif_file_system.number_of_layers, 1);

        Ok(())
    }
}
