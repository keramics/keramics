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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::lru_cache::LruCache;
use keramics_formats::{Path, PathComponent};

use super::enums::VfsType;
use super::file_entry::VfsFileEntry;
use super::file_system::VfsFileSystem;
use super::location::VfsLocation;
use super::types::VfsFileSystemReference;

/// Virtual File System (VFS) context.
pub struct VfsContext {
    /// File systems cache.
    file_systems_cache: LruCache<VfsLocation, VfsFileSystemReference>,

    /// Operating system (OS) file system path.
    os_vfs_location: VfsLocation,
}

impl VfsContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            file_systems_cache: LruCache::new(8),
            os_vfs_location: VfsLocation::from("/"),
        }
    }

    /// Retrieves a data stream with the specified location and name.
    pub fn get_data_stream_by_location_and_name(
        &mut self,
        vfs_location: &VfsLocation,
        name: Option<&PathComponent>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let file_system: VfsFileSystemReference = match self.open_file_system(vfs_location) {
            Ok(file_system) => file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        let path: &Path = vfs_location.get_path();

        match file_system.get_data_stream_by_path_and_name(path, name) {
            Ok(data_stream) => Ok(data_stream),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                return Err(error);
            }
        }
    }

    /// Retrieves a file entry with the specified location.
    pub fn get_file_entry_by_location(
        &mut self,
        vfs_location: &VfsLocation,
    ) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        let file_system: VfsFileSystemReference = match self.open_file_system(vfs_location) {
            Ok(file_system) => file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        match file_system.get_file_entry_by_location(vfs_location) {
            Ok(file_entry) => Ok(file_entry),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        }
    }

    /// Opens a file system.
    pub fn open_file_system(
        &mut self,
        vfs_location: &VfsLocation,
    ) -> Result<VfsFileSystemReference, ErrorTrace> {
        let vfs_type: &VfsType = vfs_location.get_type();
        match vfs_type {
            VfsType::Fake => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported type: VfsType::Fake"
                ));
            }
            _ => {}
        };
        let parent_vfs_location: Option<&VfsLocation> = vfs_location.get_parent();

        let lookup_key: VfsLocation = match parent_vfs_location {
            Some(parent_vfs_location) => parent_vfs_location.clone(),
            None => self.os_vfs_location.clone(),
        };
        if !self.file_systems_cache.contains(&lookup_key) {
            let parent_file_system: Option<VfsFileSystemReference> = match parent_vfs_location {
                Some(parent_vfs_location) => match self.open_file_system(parent_vfs_location) {
                    Ok(file_system) => Some(file_system),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open parent file system"
                        );
                        return Err(error);
                    }
                },
                None => None,
            };
            let mut file_system: VfsFileSystem = VfsFileSystem::new(&vfs_type);

            match file_system.open(parent_file_system.as_ref(), &lookup_key) {
                Ok(()) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            }
            self.file_systems_cache
                .insert(lookup_key.clone(), VfsFileSystemReference::new(file_system));
        }
        match self.file_systems_cache.get(&lookup_key) {
            Some(file_system) => Ok(file_system.clone()),
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve cached file system"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_get_data_stream_by_location_and_name() -> Result<(), ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let path_string: String = get_test_data_path("directory/file.txt");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let result: Option<DataStreamReference> =
            vfs_context.get_data_stream_by_location_and_name(&vfs_location, None)?;
        assert!(result.is_some());

        let path_string: String = get_test_data_path("directory/bogus.txt");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let result: Option<DataStreamReference> =
            vfs_context.get_data_stream_by_location_and_name(&vfs_location, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_location() -> Result<(), ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let path_string: String = get_test_data_path("directory/file.txt");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let result: Option<VfsFileEntry> = vfs_context.get_file_entry_by_location(&vfs_location)?;
        assert!(result.is_some());

        let path_string: String = get_test_data_path("directory/bogus.txt");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let result: Option<VfsFileEntry> = vfs_context.get_file_entry_by_location(&vfs_location)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> Result<(), ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = VfsLocation::from("/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_location)?;

        assert!(matches!(*vfs_file_system, VfsFileSystem::Os { .. }));

        Ok(())
    }
}
