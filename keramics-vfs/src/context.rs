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

use std::collections::HashMap;
use std::io;
use std::sync::Weak;

use keramics_core::DataStreamReference;

use super::enums::VfsType;
use super::file_entry::VfsFileEntry;
use super::file_system::VfsFileSystem;
use super::location::{new_os_vfs_location, VfsLocation};
use super::path::VfsPath;
use super::types::VfsFileSystemReference;

/// Virtual File System (VFS) context.
pub struct VfsContext {
    /// File systems.
    file_systems: HashMap<VfsLocation, Weak<VfsFileSystem>>,

    /// Operating system (OS) file system path.
    os_vfs_location: VfsLocation,
}

impl VfsContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            file_systems: HashMap::new(),
            os_vfs_location: new_os_vfs_location("/"),
        }
    }

    /// Retrieves a data stream with the specified path and name.
    pub fn get_data_stream_by_path_and_name(
        &mut self,
        vfs_location: &VfsLocation,
        name: Option<&str>,
    ) -> io::Result<Option<DataStreamReference>> {
        let file_system: VfsFileSystemReference = self.open_file_system(vfs_location)?;

        let vfs_path: &VfsPath = vfs_location.get_path();
        file_system.get_data_stream_by_path_and_name(vfs_path, name)
    }

    /// Retrieves a file entry with the specified path.
    pub fn get_file_entry_by_path(
        &mut self,
        vfs_location: &VfsLocation,
    ) -> io::Result<Option<VfsFileEntry>> {
        let file_system: VfsFileSystemReference = self.open_file_system(vfs_location)?;

        let vfs_path: &VfsPath = vfs_location.get_path();
        file_system.get_file_entry_by_path(vfs_path)
    }

    /// Opens a file system.
    pub fn open_file_system(
        &mut self,
        vfs_location: &VfsLocation,
    ) -> io::Result<VfsFileSystemReference> {
        let vfs_type: &VfsType = vfs_location.get_type();
        match vfs_type {
            VfsType::Fake => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported type: VfsType::Fake",
                ));
            }
            _ => {}
        };
        let parent_vfs_location: Option<&VfsLocation> = vfs_location.get_parent();

        let lookup_key: &VfsLocation = match parent_vfs_location {
            Some(parent_vfs_location) => parent_vfs_location,
            None => &self.os_vfs_location,
        };
        let cached_file_system: Option<VfsFileSystemReference> =
            match self.file_systems.get(lookup_key) {
                Some(file_system) => file_system.upgrade(),
                None => None,
            };
        match cached_file_system {
            Some(file_system) => Ok(file_system),
            None => {
                let parent_file_system: Option<VfsFileSystemReference> = match parent_vfs_location {
                    Some(parent_vfs_location) => Some(self.open_file_system(parent_vfs_location)?),
                    None => None,
                };
                let file_system_path: VfsLocation = match parent_vfs_location {
                    Some(parent_vfs_location) => parent_vfs_location.clone(),
                    None => self.os_vfs_location.clone(),
                };
                let mut file_system: VfsFileSystem = VfsFileSystem::new(&vfs_type);
                file_system.open(parent_file_system.as_ref(), &file_system_path)?;

                let cached_file_system: VfsFileSystemReference =
                    VfsFileSystemReference::new(file_system);

                self.file_systems.insert(
                    file_system_path,
                    VfsFileSystemReference::downgrade(&cached_file_system),
                );

                Ok(cached_file_system)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::location::new_os_vfs_location;

    #[test]
    fn test_get_data_stream_by_path_and_name() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");
        let result: Option<DataStreamReference> =
            vfs_context.get_data_stream_by_path_and_name(&vfs_location, None)?;
        assert!(result.is_some());

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/bogus.txt");
        let result: Option<DataStreamReference> =
            vfs_context.get_data_stream_by_path_and_name(&vfs_location, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");
        let result: Option<VfsFileEntry> = vfs_context.get_file_entry_by_path(&vfs_location)?;
        assert!(result.is_some());

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/bogus.txt");
        let result: Option<VfsFileEntry> = vfs_context.get_file_entry_by_path(&vfs_location)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = new_os_vfs_location("/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_location)?;

        assert!(matches!(*vfs_file_system, VfsFileSystem::Os { .. }));

        Ok(())
    }
}
