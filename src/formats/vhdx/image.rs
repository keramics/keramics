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

use crate::types::{SharedValue, Ucs2String};
use crate::vfs::{
    VfsFileEntry, VfsFileEntryReference, VfsFileSystem, VfsFileSystemReference, VfsPath,
    VfsPathType, WrapperVfsFileEntry,
};

use super::file::VhdxFile;
use super::layer::VhdxLayer;

/// Virtual Hard Disk (VHD) storage media image.
pub struct VhdxImage {
    /// Files.
    files: Vec<SharedValue<VhdxFile>>,
}

impl VhdxImage {
    const PATH_PREFIX: &'static str = "/vhdx";

    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Retrieves the number of layers.
    pub fn get_number_of_layers(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a layer by index.
    pub fn get_layer_by_index(&self, layer_index: usize) -> io::Result<VhdxLayer> {
        match self.files.get(layer_index) {
            Some(file) => {
                let mut layer: VhdxLayer = VhdxLayer::new();
                layer.open(&file)?;

                Ok(layer)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No layer with index: {}", layer_index),
            )),
        }
    }

    // TODO: add get_layer_index_by_identifier

    /// Retrieves the layer index with the specific location.
    fn get_layer_index_by_path(&self, location: &str) -> io::Result<usize> {
        if !location.starts_with(VhdxImage::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        // TODO: add support for identifier comparison /vhdx{UUID}

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
    fn get_layer_by_path(&self, location: &str) -> io::Result<Option<VhdxLayer>> {
        if location == "/" {
            return Ok(None);
        }
        let layer_index: usize = self.get_layer_index_by_path(location)?;

        let layer: VhdxLayer = self.get_layer_by_index(layer_index)?;

        Ok(Some(layer))
    }

    /// Opens a storage media image.
    fn open_files(&mut self, file_system: &dyn VfsFileSystem, path: &VfsPath) -> io::Result<()> {
        let directory_name: &str = file_system.get_directory_name(&path.location);

        let mut files: Vec<VhdxFile> = Vec::new();

        let mut file: VhdxFile = VhdxFile::new();
        file.open(file_system, path)?;

        while file.parent_identifier.is_some() {
            let parent_filename: Ucs2String = match file.get_parent_filename() {
                Some(parent_filename) => parent_filename,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing parent filename",
                    ));
                }
            };
            // TODO: add file_system.join function
            let parent_location: String =
                format!("{}/{}", directory_name, parent_filename.to_string());
            let parent_path: VfsPath = VfsPath::new_from_path(&path, parent_location.as_str());

            if !file_system.file_entry_exists(&parent_path)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Missing parent file: {}", parent_filename.to_string()),
                ));
            }
            let mut parent_file: VhdxFile = VhdxFile::new();
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
            self.files.push(SharedValue::new(file));

            file_index += 1;
        }
        Ok(())
    }
}

impl VfsFileSystem for VhdxImage {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        if path.path_type != VfsPathType::Vhdx {
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

    /// Opens a file system.
    fn open(
        &mut self,
        parent_file_system: VfsFileSystemReference,
        path: &VfsPath,
    ) -> io::Result<()> {
        match parent_file_system.with_write_lock() {
            Ok(file_system) => self.open_files(file_system.as_ref(), path),
            Err(error) => return Err(crate::error_to_io_error!(error)),
        }
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<VfsFileEntryReference> {
        if path.path_type != VfsPathType::Vhdx {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let layer: Option<VhdxLayer> = self.get_layer_by_path(&path.location)?;

        let mut file_entry: WrapperVfsFileEntry = WrapperVfsFileEntry::new::<VhdxLayer>(layer);
        file_entry.open(path)?;

        Ok(Box::new(file_entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType};

    fn get_image() -> io::Result<VhdxImage> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut image = VhdxImage::new();

        let vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        image.open(parent_file_system, &vfs_path)?;

        Ok(image)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/vhdx1", None);
        assert_eq!(image.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "./bogus2", None);
        assert_eq!(image.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let image: VhdxImage = VhdxImage::new();

        let directory_name: &str = image.get_directory_name("/vhdx1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_layer_by_index() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let layer: VhdxLayer = image.get_layer_by_index(0)?;

        assert_eq!(layer.size, 4194304);
        assert_eq!(
            layer.identifier.to_string(),
            "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
        );
        Ok(())
    }

    #[test]
    fn get_layer_index_by_path() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let layer_index: usize = image.get_layer_index_by_path("/vhdx1")?;
        assert_eq!(layer_index, 0);

        let result = image.get_layer_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = image.get_layer_index_by_path("/vhdx99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_layer_by_path() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let result: Option<VhdxLayer> = image.get_layer_by_path("/")?;
        assert!(result.is_none());

        let result: Option<VhdxLayer> = image.get_layer_by_path("/vhdx1")?;
        assert!(result.is_some());

        let layer: VhdxLayer = result.unwrap();

        assert_eq!(layer.size, 4194304);
        assert_eq!(
            layer.identifier.to_string(),
            "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
        );
        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut image = VhdxImage::new();

        let vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        image.open(parent_file_system, &vfs_path)?;

        Ok(())
    }

    #[test]
    fn test_open_files() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut image = VhdxImage::new();

        let vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        match parent_file_system.with_write_lock() {
            Ok(file_system) => image.open_files(file_system.as_ref(), &vfs_path)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert_eq!(image.get_number_of_layers(), 2);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let os_vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference = image.open_file_entry(&test_vfs_path)?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_file() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let os_vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/vhdx1", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference = image.open_file_entry(&test_vfs_path)?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let image: VhdxImage = get_image()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::NotSet, "/", None);

        let result = image.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
