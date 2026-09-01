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
use keramics_formats::cdsaencr::CdsaEncrCredential;
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::{FileResolverReference, Path, PathComponent};

use crate::credential::VfsCredential;
use crate::credential_store::VfsCredentialStore;
use crate::file_resolver::new_vfs_file_resolver;
use crate::location::VfsLocation;
use crate::types::VfsFileSystemReference;

use super::file_entry::SparseBundleFileEntry;

/// Mac OS sparse image (.sparsebundle) storage media image file system.
pub struct SparseBundleFileSystem {
    /// File.
    image: Arc<SparseBundleImage>,

    /// Number of layers.
    number_of_layers: usize,
}

impl SparseBundleFileSystem {
    pub const PATH_PREFIX: &'static str = "/sparsebundle";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            image: Arc::new(SparseBundleImage::new()),
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
                if path_component != "sparsebundle1" {
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
    ) -> Result<Option<SparseBundleFileEntry>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                if path_component != "sparsebundle1" {
                    return Ok(None);
                }
                Ok(Some(SparseBundleFileEntry::Layer {
                    image: self.image.clone(),
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
    pub fn get_root_file_entry(&self) -> SparseBundleFileEntry {
        SparseBundleFileEntry::Root {
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
                match Self::open_image(image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sparsebundle image"
                        );
                        return Err(error);
                    }
                }
                self.number_of_layers = 1;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to sparsebundle image"
                ));
            }
        }
        Ok(())
    }

    /// Opens a sparsebundle image.
    pub(crate) fn open_image(
        image: &mut SparseBundleImage,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let file_resolver: FileResolverReference =
            match new_vfs_file_resolver(file_system, path.clone()) {
                Ok(file_resolver) => file_resolver,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create VFS file resolver"
                    );
                    return Err(error);
                }
            };
        let file_name: PathComponent = PathComponent::from("Info.plist");
        match image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open sparsebundle image");
                return Err(error);
            }
        }
        if image.is_locked() {
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<CdsaEncrCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(CdsaEncrCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match image.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Failed to unlock sparsebundle image"
                    );
                    return Err(error);
                }
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

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<SparseBundleFileSystem, ErrorTrace> {
        let mut sparsebundle_file_system: SparseBundleFileSystem = SparseBundleFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        sparsebundle_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(sparsebundle_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let sparsebundle_file_system: SparseBundleFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sparsebundle1");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, true);

        let path: Path = Path::from("/sparsebundle99");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("sparsebundle1");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/bogus1");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        let path: Path = Path::from("/sparsebundle1/bogus1");
        let result: bool = sparsebundle_file_system.file_entry_exists(&path);
        assert_eq!(result, false);

        Ok(())
    }

    // TODO: add tests for get_bytes_per_sector

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let sparsebundle_file_system: SparseBundleFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<SparseBundleFileEntry> =
            sparsebundle_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let sparsebundle_file_entry: SparseBundleFileEntry = result.unwrap();

        let name: PathComponent = sparsebundle_file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_type: VfsFileType = sparsebundle_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/sparsebundle1");
        let result: Option<SparseBundleFileEntry> =
            sparsebundle_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let sparsebundle_file_entry: SparseBundleFileEntry = result.unwrap();

        let name: PathComponent = sparsebundle_file_entry.get_name();
        assert_eq!(name, PathComponent::from("sparsebundle1"));

        let file_type: VfsFileType = sparsebundle_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/bogus1");
        let result: Option<SparseBundleFileEntry> =
            sparsebundle_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let sparsebundle_file_system: SparseBundleFileSystem = get_file_system()?;

        let sparsebundle_file_entry: SparseBundleFileEntry =
            sparsebundle_file_system.get_root_file_entry();
        assert!(matches!(
            sparsebundle_file_entry,
            SparseBundleFileEntry::Root { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut sparsebundle_file_system: SparseBundleFileSystem = SparseBundleFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let parent_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        sparsebundle_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(sparsebundle_file_system.number_of_layers, 1);

        Ok(())
    }

    // TODO: add tests for open_image
}
