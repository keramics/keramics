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

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;

use crate::vfs::{VfsFileSystem, VfsPath};

use super::file::VhdFile;
use super::layer::VhdLayer;

/// Virtual Hard Disk (VHD) storage media image.
pub struct VhdImage {
    /// Files.
    files: Vec<Rc<RefCell<VhdFile>>>,
}

impl VhdImage {
    pub const PATH_PREFIX: &'static str = "/vhd";

    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> io::Result<VhdLayer> {
        match self.files.get(layer_index) {
            Some(file) => match file.try_borrow() {
                Ok(vhd_file) => Ok(VhdLayer::new(
                    file,
                    &vhd_file.identifier,
                    vhd_file.media_size,
                )),
                Err(error) => return Err(crate::error_to_io_error!(error)),
            },
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No layer with index: {}", layer_index),
            )),
        }
    }

    // TODO: add get_layer_index_by_identifier

    /// Retrieves the layer index with the specific location.
    pub(crate) fn get_layer_index_by_path(&self, location: &str) -> io::Result<usize> {
        if !location.starts_with(VhdImage::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        // TODO: add support for identifier comparison /vhd{UUID}

        let layer_index: usize = match location[4..].parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported path: {}", location),
                ))
            }
        };
        if layer_index == 0 || layer_index > self.files.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        Ok(layer_index - 1)
    }

    /// Retrieves the layer with the specific location.
    pub(crate) fn get_layer_by_path(&self, location: &str) -> io::Result<Option<VhdLayer>> {
        if location == "/" {
            return Ok(None);
        }
        let layer_index: usize = self.get_layer_index_by_path(location)?;

        let layer: VhdLayer = self.get_layer_by_index(layer_index)?;

        Ok(Some(layer))
    }

    /// Opens a storage media image.
    pub fn open(&mut self, file_system: &Arc<VfsFileSystem>, path: &VfsPath) -> io::Result<()> {
        let directory_path: VfsPath = path.parent_directory();

        let mut files: Vec<VhdFile> = Vec::new();

        let mut file: VhdFile = VhdFile::new();
        file.open(file_system, path)?;

        while file.parent_identifier.is_some() {
            let parent_filename: String = match file.get_parent_filename() {
                Some(file_name) => file_name.to_string(),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing parent filename",
                    ));
                }
            };
            let parent_path: VfsPath =
                directory_path.append_components(&mut vec![parent_filename.as_str()]);
            if !file_system.file_entry_exists(&parent_path)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Missing parent file: {}", parent_filename),
                ));
            }
            let mut parent_file: VhdFile = VhdFile::new();
            parent_file.open(file_system, &parent_path)?;

            files.push(file);

            file = parent_file;
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                file.set_parent(&mut self.files[file_index - 1])?;
            }
            self.files.push(Rc::new(RefCell::new(file)));

            file_index += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::VfsContext;

    fn get_image() -> io::Result<VhdImage> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut image: VhdImage = VhdImage::new();

        image.open(&vfs_file_system, &vfs_path)?;

        Ok(image)
    }

    #[test]
    fn test_get_layer_by_index() -> io::Result<()> {
        let image: VhdImage = get_image()?;

        let layer: VhdLayer = image.get_layer_by_index(0)?;

        assert_eq!(layer.size, 4194304);
        assert_eq!(
            layer.identifier.to_string(),
            "e7ea9200-8493-954e-a816-9572339be931"
        );
        Ok(())
    }

    #[test]
    fn get_layer_index_by_path() -> io::Result<()> {
        let image: VhdImage = get_image()?;

        let layer_index: usize = image.get_layer_index_by_path("/vhd1")?;
        assert_eq!(layer_index, 0);

        let result = image.get_layer_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = image.get_layer_index_by_path("/vhd99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_layer_by_path() -> io::Result<()> {
        let image: VhdImage = get_image()?;

        let result: Option<VhdLayer> = image.get_layer_by_path("/")?;
        assert!(result.is_none());

        let result: Option<VhdLayer> = image.get_layer_by_path("/vhd1")?;
        assert!(result.is_some());

        let layer: VhdLayer = result.unwrap();

        assert_eq!(layer.size, 4194304);
        assert_eq!(
            layer.identifier.to_string(),
            "e7ea9200-8493-954e-a816-9572339be931"
        );
        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut image: VhdImage = VhdImage::new();

        image.open(&vfs_file_system, &vfs_path)?;

        Ok(())
    }
}
