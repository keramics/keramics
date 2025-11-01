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
use keramics_formats::ewf::EwfImage;
use keramics_formats::{FileResolverReference, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
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
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path) => {
                let number_of_components: usize = string_path.components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(false);
                }
                if string_path.components[0] != "" {
                    return Ok(false);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    return Ok(true);
                }
                if string_path.components[1] == "ewf1" {
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
    ) -> Result<Option<EwfFileEntry>, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path) => {
                let number_of_components: usize = string_path.components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(None);
                }
                if string_path.components[0] != "" {
                    return Ok(None);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    let ewf_file_entry: EwfFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(ewf_file_entry));
                }
                if string_path.components[1] == "ewf1" {
                    let ewf_file_entry: EwfFileEntry = EwfFileEntry::Layer {
                        image: self.image.clone(),
                    };
                    Ok(Some(ewf_file_entry))
                } else {
                    Ok(None)
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<EwfFileEntry, ErrorTrace> {
        Ok(EwfFileEntry::Root {
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

        match self.image.write() {
            Ok(mut image) => {
                match Self::open_image(&mut image, file_system, vfs_path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                        return Err(error);
                    }
                }
                self.number_of_layers = 1;
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on EWF image",
                    error
                ));
            }
        }
        Ok(())
    }

    /// Opens an EWF image.
    pub(crate) fn open_image(
        image: &mut EwfImage,
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
                keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
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

    fn get_file_system() -> Result<EwfFileSystem, ErrorTrace> {
        let mut ewf_file_system: EwfFileSystem = EwfFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("ewf/ext2.E01").as_str());
        ewf_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(ewf_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/bougs1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1/bogus1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "bogus1");
        let result: bool = ewf_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "/");
        let result: Result<bool, ErrorTrace> = ewf_file_system.file_entry_exists(&vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let ewf_file_entry: EwfFileEntry = result.unwrap();

        let name: Option<String> = ewf_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let ewf_file_entry: EwfFileEntry = result.unwrap();

        let name: Option<String> = ewf_file_entry.get_name();
        assert_eq!(name, Some(String::from("ewf1")));

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/bougs1");
        let result: Option<EwfFileEntry> = ewf_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let ewf_file_system: EwfFileSystem = get_file_system()?;

        let ewf_file_entry: EwfFileEntry = ewf_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = ewf_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut ewf_file_system: EwfFileSystem = EwfFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("ewf/ext2.E01").as_str());
        ewf_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(ewf_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_image
}
