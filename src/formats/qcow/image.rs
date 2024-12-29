/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::types::{ByteString, SharedValue};
use crate::vfs::{
    VfsFileEntryReference, VfsFileSystem, VfsFileSystemReference, VfsPath, VfsPathReference,
    VfsPathType, WrapperVfsFileEntry,
};

use super::file::QcowFile;
use super::layer::QcowLayer;

/// QEMU Copy-On-Write (QCOW) storage media image.
pub struct QcowImage {
    /// Files.
    files: Vec<SharedValue<QcowFile>>,
}

impl QcowImage {
    const PATH_PREFIX: &'static str = "/qcow";

    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> io::Result<QcowLayer> {
        match self.files.get(layer_index) {
            Some(file) => {
                let mut layer: QcowLayer = QcowLayer::new();
                layer.open(&file)?;

                Ok(layer)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No layer with index: {}", layer_index),
            )),
        }
    }

    /// Retrieves the layer index with the specific location.
    fn get_layer_index_by_path(&self, location: &str) -> io::Result<usize> {
        if !location.starts_with(QcowImage::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        let layer_index: usize = match location[5..].parse::<usize>() {
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
    fn get_layer_by_path(&self, location: &str) -> io::Result<Option<QcowLayer>> {
        if location == "/" {
            return Ok(None);
        }
        let layer_index: usize = self.get_layer_index_by_path(location)?;

        let layer: QcowLayer = self.get_layer_by_index(layer_index)?;

        Ok(Some(layer))
    }
}

impl VfsFileSystem for QcowImage {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPathReference) -> io::Result<bool> {
        if path.path_type != VfsPathType::Qcow {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        if path.location == "/" {
            return Ok(true);
        }
        match self.get_layer_index_by_path(&path.location) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Retrieves the path type.
    fn get_vfs_path_type(&self) -> VfsPathType {
        VfsPathType::Qcow
    }

    /// Opens a storage media image.
    fn open(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        let directory_name: &str = match file_system.with_read_lock() {
            Ok(file_system) => file_system.get_directory_name(&path.location),
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut files: Vec<QcowFile> = Vec::new();

        let mut file: QcowFile = QcowFile::new();
        file.open(file_system, path)?;

        while file.backing_file_name.is_some() {
            let backing_file_name: &ByteString = file.backing_file_name.as_ref().unwrap();

            // TODO: add file_system.join function
            let backing_file_location: String =
                format!("{}/{}", directory_name, backing_file_name.to_string());
            let backing_file_path: VfsPathReference =
                VfsPath::new_from_path(&path, backing_file_location.as_str());

            let file_exists: bool = match file_system.with_read_lock() {
                Ok(file_system) => file_system.file_entry_exists(&backing_file_path)?,
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            if !file_exists {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Missing backing file: {}", backing_file_name.to_string()),
                ));
            }
            let mut backing_file: QcowFile = QcowFile::new();
            backing_file.open(file_system, &backing_file_path)?;

            files.push(file);

            file = backing_file;
        }
        files.push(file);

        let mut file_index: usize = 0;
        while let Some(mut file) = files.pop() {
            if file_index > 0 {
                file.set_backing_file(&mut self.files[file_index - 1])?;
            }
            self.files.push(SharedValue::new(file));

            file_index += 1;
        }
        Ok(())
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(
        &self,
        path: &VfsPathReference,
    ) -> io::Result<Option<VfsFileEntryReference>> {
        if path.path_type != VfsPathType::Qcow {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let layer: Option<QcowLayer> = match self.get_layer_by_path(&path.location) {
            Ok(layer) => layer,
            Err(_) => return Ok(None),
        };
        let mut file_entry: WrapperVfsFileEntry = WrapperVfsFileEntry::new::<QcowLayer>(layer);
        file_entry.initialize(path)?;

        Ok(Some(Box::new(file_entry)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType};

    fn get_image() -> io::Result<QcowImage> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut image: QcowImage = QcowImage::new();

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        image.open(&vfs_file_system, &vfs_path)?;

        Ok(image)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        assert_eq!(image.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "./bogus2", None);
        assert_eq!(image.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let image: QcowImage = QcowImage::new();

        let directory_name: &str = image.get_directory_name("/qcow1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let layer: QcowLayer = image.get_layer_by_index(0)?;

        assert_eq!(layer.size, 4194304);

        Ok(())
    }

    #[test]
    fn get_layer_index_by_path() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let layer_index: usize = image.get_layer_index_by_path("/qcow1")?;
        assert_eq!(layer_index, 0);

        let result = image.get_layer_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = image.get_layer_index_by_path("/qcow99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_layer_by_path() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let result: Option<QcowLayer> = image.get_layer_by_path("/")?;
        assert!(result.is_none());

        let result: Option<QcowLayer> = image.get_layer_by_path("/qcow1")?;
        assert!(result.is_some());

        let layer: QcowLayer = result.unwrap();

        assert_eq!(layer.size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_vfs_path_type() -> io::Result<()> {
        let image: QcowImage = QcowImage::new();

        let vfs_path_type: VfsPathType = image.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Qcow);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut image: QcowImage = QcowImage::new();

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        image.open(&vfs_file_system, &vfs_path)?;

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference = image.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_file() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/qcow1", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference = image.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_non_existing() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/bogus1", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> = image.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let image: QcowImage = get_image()?;

        let test_vfs_path: VfsPathReference = VfsPath::new(VfsPathType::NotSet, "/", None);

        let result = image.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
