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
use keramics_formats::qcow::{QcowImage, QcowImageLayer};
use keramics_formats::{FileResolverReference, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::QcowFileEntry;

/// QEMU Copy-On-Write (QCOW) storage media image file system.
pub struct QcowFileSystem {
    /// Storage media image.
    image: Arc<QcowImage>,

    /// Number of layers.
    number_of_layers: usize,
}

impl QcowFileSystem {
    pub const PATH_PREFIX: &'static str = "/qcow";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(QcowImage::new()),
            number_of_layers: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        match vfs_path {
            VfsPath::Path(path) => {
                if path.is_relative() {
                    return Ok(false);
                }
                match path.get_component_by_index(1) {
                    Some(path_component) => {
                        if path.get_number_of_components() > 2 {
                            return Ok(false);
                        }
                        let layer_index: usize =
                            match VfsPath::get_numeric_suffix(path_component, "qcow") {
                                Some(layer_index) => layer_index,
                                None => return Ok(false),
                            };
                        if layer_index == 0 || layer_index > self.number_of_layers {
                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    }
                    None => {
                        if path.is_empty() {
                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    }
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<QcowFileEntry>, ErrorTrace> {
        match vfs_path {
            VfsPath::Path(path) => {
                if path.is_relative() {
                    return Ok(None);
                }
                match path.get_component_by_index(1) {
                    Some(path_component) => {
                        if path.get_number_of_components() > 2 {
                            return Ok(None);
                        }
                        let mut layer_index: usize =
                            match VfsPath::get_numeric_suffix(path_component, "qcow") {
                                Some(layer_index) => layer_index,
                                None => return Ok(None),
                            };
                        if layer_index == 0 || layer_index > self.number_of_layers {
                            return Ok(None);
                        }
                        layer_index -= 1;

                        let qcow_layer: QcowImageLayer =
                            match self.image.get_layer_by_index(layer_index) {
                                Ok(qcow_layer) => qcow_layer,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!("Unable to retrieve QCOW layer: {}", layer_index)
                                    );
                                    return Err(error);
                                }
                            };
                        let media_size: u64 = match qcow_layer.read() {
                            Ok(qcow_file) => qcow_file.media_size,
                            Err(error) => {
                                return Err(keramics_core::error_trace_new_with_error!(
                                    "Unable to obtain read lock on QCOW layer",
                                    error
                                ));
                            }
                        };
                        Ok(Some(QcowFileEntry::Layer {
                            index: layer_index,
                            layer: qcow_layer.clone(),
                            size: media_size,
                        }))
                    }
                    None => {
                        if path.is_empty() {
                            return Ok(None);
                        }
                        Ok(Some(QcowFileEntry::Root {
                            image: self.image.clone(),
                        }))
                    }
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<QcowFileEntry, ErrorTrace> {
        Ok(QcowFileEntry::Root {
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
                        keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                        return Err(error);
                    }
                }
                self.number_of_layers = image.get_number_of_layers();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to QCOW image"
                ));
            }
        }
        Ok(())
    }

    /// Opens a QCOW image.
    pub(crate) fn open_image(
        image: &mut QcowImage,
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
                keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
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

    fn get_file_system() -> Result<QcowFileSystem, ErrorTrace> {
        let mut qcow_file_system: QcowFileSystem = QcowFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("qcow/ext2.qcow2").as_str());
        qcow_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(qcow_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let qcow_file_system: QcowFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/");
        let result: bool = qcow_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/qcow1");
        let result: bool = qcow_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/bogus1");
        let result: bool = qcow_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/qcow1/bogus1");
        let result: bool = qcow_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "bogus1");
        let result: bool = qcow_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "/");
        let result: Result<bool, ErrorTrace> = qcow_file_system.file_entry_exists(&vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let qcow_file_system: QcowFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/");
        let result: Option<QcowFileEntry> = qcow_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let qcow_file_entry: QcowFileEntry = result.unwrap();

        let name: Option<String> = qcow_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = qcow_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/qcow1");
        let result: Option<QcowFileEntry> = qcow_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let qcow_file_entry: QcowFileEntry = result.unwrap();

        let name: Option<String> = qcow_file_entry.get_name();
        assert_eq!(name, Some(String::from("qcow1")));

        let file_type: VfsFileType = qcow_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Qcow, "/bogus1");
        let result: Option<QcowFileEntry> = qcow_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let qcow_file_system: QcowFileSystem = get_file_system()?;

        let qcow_file_entry: QcowFileEntry = qcow_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = qcow_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut qcow_file_system: QcowFileSystem = QcowFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("qcow/ext2.qcow2").as_str());
        qcow_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(qcow_file_system.number_of_layers, 1);

        Ok(())
    }
}
