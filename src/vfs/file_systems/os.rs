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
use std::path::{Path, MAIN_SEPARATOR, MAIN_SEPARATOR_STR};

use crate::vfs::enums::VfsPathType;
use crate::vfs::file_entries::OsVfsFileEntry;
use crate::vfs::path::VfsPath;
use crate::vfs::traits::{VfsFileEntry, VfsFileSystem};
use crate::vfs::types::{VfsFileEntryReference, VfsPathReference};

/// Operating system file system.
pub struct OsVfsFileSystem {}

impl OsVfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {}
    }
}

impl VfsFileSystem for OsVfsFileSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        if path.path_type != VfsPathType::Os {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let os_path: &Path = Path::new(&path.location);

        os_path.try_exists()
    }

    /// Retrieves the directory name of the specified location.
    fn get_directory_name<'a>(&self, location: &'a str) -> &'a str {
        let directory_name: &str = match location.rsplit_once(MAIN_SEPARATOR) {
            Some(path_components) => path_components.0,
            None => "",
        };
        if directory_name == "" {
            MAIN_SEPARATOR_STR
        } else {
            directory_name
        }
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<VfsFileEntryReference> {
        if path.path_type != VfsPathType::Os {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let mut file_entry: OsVfsFileEntry = OsVfsFileEntry::new();

        file_entry.open(path)?;

        Ok(Box::new(file_entry))
    }

    /// Opens a file system.
    fn open_with_resolver(&mut self, path: &VfsPath) -> io::Result<()> {
        if path.path_type != VfsPathType::Os {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let parent_path: Option<VfsPathReference> = path.get_parent();

        if parent_path.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Parent set in path",
            ));
        }
        if path.location != "/" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Location in path is not /",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsFileType;

    fn get_file_system() -> io::Result<OsVfsFileSystem> {
        let mut vfs_file_system: OsVfsFileSystem = OsVfsFileSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        vfs_file_system.open_with_resolver(&vfs_path)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let vfs_file_system: OsVfsFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let vfs_file_system: OsVfsFileSystem = OsVfsFileSystem::new();

        let directory_name: &str = vfs_file_system.get_directory_name("./test_data/file.txt");
        assert_eq!(directory_name, "./test_data");

        Ok(())
    }

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let vfs_file_system: OsVfsFileSystem = get_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?;
        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let vfs_file_system: OsVfsFileSystem = get_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::NotSet, "/", None);
        let result = vfs_file_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_open_with_resolver() -> io::Result<()> {
        let mut vfs_file_system: OsVfsFileSystem = OsVfsFileSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        vfs_file_system.open_with_resolver(&vfs_path)?;

        Ok(())
    }
}
