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
use keramics_formats::sparseimage::SparseImageFile;

use crate::location::VfsLocation;
use crate::types::VfsFileSystemReference;

use super::file_entry::SparseImageFileEntry;

/// Mac OS sparse image (.sparseimage) storage media image file system.
pub struct SparseImageFileSystem {
    /// File.
    file: Arc<RwLock<SparseImageFile>>,

    /// Number of layers.
    number_of_layers: usize,
}

impl SparseImageFileSystem {
    pub const PATH_PREFIX: &'static str = "/sparseimage";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            file: Arc::new(RwLock::new(SparseImageFile::new())),
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
                if path_component != "sparseimage1" {
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
    ) -> Result<Option<SparseImageFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                if path_component != "sparseimage1" {
                    return Ok(None);
                }
                let media_size: u64 = match self.file.read() {
                    Ok(sparseimage_file) => sparseimage_file.media_size,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to obtain read lock on sparseimage file",
                            error
                        ));
                    }
                };
                Ok(Some(SparseImageFileEntry::Layer {
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
    pub fn get_root_file_entry(&self) -> SparseImageFileEntry {
        SparseImageFileEntry::Root {
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
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sparseimage file"
                        );
                        return Err(error);
                    }
                }
                self.number_of_layers = 1;
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on sparseimage file",
                    error
                ));
            }
        }
        Ok(())
    }

    /// Opens a sparseimage file.
    pub(crate) fn open_file(
        file: &mut SparseImageFile,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference =
            match file_system.get_data_stream_by_path_and_name(path, None) {
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
                    "Unable to read sparseimage file from data stream"
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

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::new_os_vfs_location;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<SparseImageFileSystem, ErrorTrace> {
        let mut sparseimage_file_system: SparseImageFileSystem = SparseImageFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("sparseimage/hfsplus.sparseimage").as_str());
        sparseimage_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(sparseimage_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let sparseimage_file_system: SparseImageFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sparseimage1");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sparseimage99");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("sparseimage1");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/sparseimage1/bogus1");
        let result: bool = sparseimage_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let sparseimage_file_system: SparseImageFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<SparseImageFileEntry> =
            sparseimage_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let sparseimage_file_entry: SparseImageFileEntry = result.unwrap();

        let name: Option<String> = sparseimage_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = sparseimage_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let path: Path = Path::from("/sparseimage1");
        let result: Option<SparseImageFileEntry> =
            sparseimage_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let sparseimage_file_entry: SparseImageFileEntry = result.unwrap();

        let name: Option<String> = sparseimage_file_entry.get_name();
        assert_eq!(name, Some(String::from("sparseimage1")));

        let file_type: VfsFileType = sparseimage_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<SparseImageFileEntry> =
            sparseimage_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let sparseimage_file_system: SparseImageFileSystem = get_file_system()?;

        let sparseimage_file_entry: SparseImageFileEntry =
            sparseimage_file_system.get_root_file_entry();
        assert!(matches!(
            sparseimage_file_entry,
            SparseImageFileEntry::Root { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut sparseimage_file_system: SparseImageFileSystem = SparseImageFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("sparseimage/hfsplus.sparseimage").as_str());
        sparseimage_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(sparseimage_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_file
}
