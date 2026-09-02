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
use keramics_formats::pdi::{PdiImage, PdiImageLayer};
use keramics_formats::{FileResolverReference, Path};

use crate::file_resolver::new_vfs_file_resolver;
use crate::image::VfsImageFileSystem;
use crate::traits::VfsImage;
use crate::types::VfsFileSystemReference;

/// Parallels Disk Image (PDI) storage media image file system.
pub type PdiFileSystem = VfsImageFileSystem<PdiImage, PdiImageLayer>;

impl VfsImage for PdiImage {
    /// Path prefix.
    const PATH_PREFIX: &'static str = "/pdi";

    /// Image layer type.
    type Layer = PdiImageLayer;

    /// Creates a new image.
    fn new() -> Self {
        PdiImage::new()
    }

    /// Retrieves the bytes per sector.
    fn get_bytes_per_sector(&self) -> u16 {
        PdiImage::get_bytes_per_sector(self)
    }

    /// Retrieves the number of layers.
    fn get_number_of_layers(&self) -> usize {
        PdiImage::get_number_of_layers(self)
    }

    /// Retrieves a layer by index.
    fn get_layer_by_index(&self, layer_index: usize) -> Result<Arc<PdiImageLayer>, ErrorTrace> {
        PdiImage::get_layer_by_index(self, layer_index)
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
        match self.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open PDI image");
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

    use crate::location::VfsLocation;
    use crate::pdi::file_entry::PdiFileEntry;
    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<PdiFileSystem, ErrorTrace> {
        let mut pdi_file_system: PdiFileSystem = PdiFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd/DiskDescriptor.xml");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        pdi_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(pdi_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let pdi_file_system: PdiFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/pdi1");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/pdi99");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("pdi1");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/pdi1/bogus1");
        let result: bool = pdi_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    // TODO: add tests for get_bytes_per_sector

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let pdi_file_system: PdiFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<PdiFileEntry> = pdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let pdi_file_entry: PdiFileEntry = result.unwrap();

        let name: PathComponent = pdi_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = pdi_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/pdi1");
        let result: Option<PdiFileEntry> = pdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let pdi_file_entry: PdiFileEntry = result.unwrap();

        let name: PathComponent = pdi_file_entry.get_name();
        assert_eq!(name, PathComponent::from("pdi1"));

        let file_type: VfsFileType = pdi_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<PdiFileEntry> = pdi_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let pdi_file_system: PdiFileSystem = get_file_system()?;

        let pdi_file_entry: PdiFileEntry = pdi_file_system.get_root_file_entry();
        assert!(matches!(pdi_file_entry, PdiFileEntry::Root { .. }));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut pdi_file_system: PdiFileSystem = PdiFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd/DiskDescriptor.xml");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        pdi_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(pdi_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_image
}
