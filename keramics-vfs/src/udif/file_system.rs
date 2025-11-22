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
use keramics_formats::udif::UdifFile;

use crate::location::VfsLocation;
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
    pub fn file_entry_exists(&self, path: &Path) -> bool {
        if path.is_relative() {
            return false;
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return false;
                }
                if path_component != "udif1" {
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
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<UdifFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                if path_component != "udif1" {
                    return Ok(None);
                }
                let media_size: u64 = match self.file.read() {
                    Ok(udif_file) => udif_file.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on UDIF file",
                            error
                        ));
                    }
                };
                Ok(Some(UdifFileEntry::Layer {
                    file: self.file.clone(),
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
    pub fn get_root_file_entry(&self) -> UdifFileEntry {
        UdifFileEntry::Root {
            file: self.file.clone(),
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

        match self.file.write() {
            Ok(mut file) => {
                match Self::open_file(&mut file, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open UDIF file");
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

    /// Opens an UDIF file.
    pub(crate) fn open_file(
        file: &mut UdifFile,
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

    fn get_file_system() -> Result<UdifFileSystem, ErrorTrace> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("udif/hfsplus_zlib.dmg").as_str());
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(udif_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/udif1");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/udif99");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("udif1");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/udif1/bogus1");
        let result: bool = udif_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: PathComponent = udif_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/udif1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let udif_file_entry: UdifFileEntry = result.unwrap();

        let name: PathComponent = udif_file_entry.get_name();
        assert_eq!(name, PathComponent::from("udif1"));

        let file_type: VfsFileType = udif_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<UdifFileEntry> = udif_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let udif_file_system: UdifFileSystem = get_file_system()?;

        let udif_file_entry: UdifFileEntry = udif_file_system.get_root_file_entry();
        assert!(matches!(udif_file_entry, UdifFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut udif_file_system: UdifFileSystem = UdifFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("udif/hfsplus_zlib.dmg").as_str());
        udif_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(udif_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_file
}
