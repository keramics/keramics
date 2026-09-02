/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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
use keramics_formats::Path;

use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::traits::{VfsImage, VfsImageLayer};
use crate::types::VfsFileSystemReference;

use super::file_entry::VfsImageFileEntry;

/// Virtual File System (VFS) image based file system.
pub struct VfsImageFileSystem<I: VfsImage<Layer = L>, L: VfsImageLayer> {
    /// Storage media image.
    image: Arc<I>,

    /// Number of layers.
    pub(crate) number_of_layers: usize,
}

impl<I: VfsImage<Layer = L>, L: VfsImageLayer> VfsImageFileSystem<I, L> {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(I::new()),
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
                let layer_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, L::NAME_PREFIX) {
                        Some(layer_index) => layer_index,
                        None => return false,
                    };
                if layer_index == 0 || layer_index > self.number_of_layers {
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
        Ok(self.image.get_bytes_per_sector() as u32)
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<VfsImageFileEntry<I, L>>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut layer_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, L::NAME_PREFIX) {
                        Some(layer_index) => layer_index,
                        None => return Ok(None),
                    };
                if layer_index == 0 || layer_index > self.number_of_layers {
                    return Ok(None);
                }
                layer_index -= 1;

                let image_layer: Arc<L> = match self.image.get_layer_by_index(layer_index) {
                    Ok(image_layer) => image_layer,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve image layer: {}", layer_index)
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsImageFileEntry::Layer {
                    name_index: layer_index,
                    layer: image_layer,
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
    pub fn get_root_file_entry(&self) -> VfsImageFileEntry<I, L> {
        VfsImageFileEntry::Root {
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
                match image.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open image");
                        return Err(error);
                    }
                }
                self.number_of_layers = image.get_number_of_layers();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to image"
                ));
            }
        }
        Ok(())
    }
}
