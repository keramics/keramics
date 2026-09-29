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
use keramics_formats::vdi::{VdiFile, VdiImage};
use keramics_formats::{FileResolverReference, Path, PathComponent};

use crate::file_resolver::new_vfs_file_resolver;
use crate::image::VfsImageFileSystem;
use crate::traits::VfsImage;
use crate::types::VfsFileSystemReference;

/// Virtual Disk Image (VDI) storage media image file system.
pub type VdiFileSystem = VfsImageFileSystem<VdiImage, VdiFile>;

impl VfsImage for VdiImage {
    /// Path prefix.
    const PATH_PREFIX: &'static str = "/vdi";

    /// Image layer type.
    type Layer = VdiFile;

    /// Creates a new image.
    fn new() -> Self {
        VdiImage::new()
    }

    /// Retrieves the bytes per sector.
    fn get_bytes_per_sector(&self) -> u16 {
        VdiImage::get_bytes_per_sector(self)
    }

    /// Retrieves the number of layers.
    fn get_number_of_layers(&self) -> usize {
        VdiImage::get_number_of_layers(self)
    }

    /// Retrieves a layer by index.
    fn get_layer_by_index(&self, layer_index: usize) -> Result<Arc<VdiFile>, ErrorTrace> {
        VdiImage::get_layer_by_index(self, layer_index)
    }

    /// Opens the image from VFS.
    fn open_from_vfs(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let parent_path: Path = path.new_with_parent_directory();

        let file_resolver: FileResolverReference =
            match new_vfs_file_resolver(file_system, parent_path) {
                Ok(file_resolver) => file_resolver,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create VFS file resolver"
                    );
                    return Err(error);
                }
            };
        let file_name: &PathComponent = match path.file_name() {
            Some(file_name) => file_name,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve file name"
                ));
            }
        };
        let file_names: [PathComponent; 1] = [file_name.clone()];

        match self.open(&file_resolver, &file_names) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VDI image");
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

    use crate::location::VfsLocation;
    use crate::tests::get_test_data_path;
    use crate::vdi::file_entry::VdiFileEntry;

    fn get_file_system() -> Result<VdiFileSystem, ErrorTrace> {
        // TODO: create differential test image
        let mut vdi_file_system: VdiFileSystem = VdiFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        vdi_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(vdi_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let vdi_file_system: VdiFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/vdi1");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/vdi99");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("vdi1");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/vdi1/bogus1");
        let result: bool = vdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    // TODO: add tests for get_bytes_per_sector

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let vdi_file_system: VdiFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<VdiFileEntry> = vdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let vdi_file_entry: VdiFileEntry = result.unwrap();

        let name: PathComponent = vdi_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = vdi_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/vdi1");
        let result: Option<VdiFileEntry> = vdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let vdi_file_entry: VdiFileEntry = result.unwrap();

        let name: PathComponent = vdi_file_entry.get_name();
        assert_eq!(name, PathComponent::from("vdi1"));

        let file_type: VfsFileType = vdi_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<VdiFileEntry> = vdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let vdi_file_system: VdiFileSystem = get_file_system()?;

        let vdi_file_entry: VdiFileEntry = vdi_file_system.get_root_file_entry();
        assert!(matches!(vdi_file_entry, VdiFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        // TODO: create differential test image
        let mut vdi_file_system: VdiFileSystem = VdiFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        vdi_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(vdi_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_image
}
