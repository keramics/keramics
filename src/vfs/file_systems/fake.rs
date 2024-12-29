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

use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use crate::vfs::enums::VfsPathType;
use crate::vfs::file_entries::FakeVfsFileEntry;
use crate::vfs::traits::VfsFileSystem;
use crate::vfs::types::{VfsFileEntryReference, VfsFileSystemReference, VfsPathReference};

/// Fake (or virtual) file system.
pub struct FakeVfsFileSystem {
    /// Paths.
    paths: HashMap<String, Rc<FakeVfsFileEntry>>,

    /// Value to indicate the file system has been opened.
    is_open: bool,
}

impl FakeVfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            is_open: false,
        }
    }

    /// Adds a new file entry.
    pub fn add_file_entry(
        &mut self,
        location: &str,
        file_entry: FakeVfsFileEntry,
    ) -> io::Result<()> {
        if self.is_open {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unable to add file entry after file system has been opened",
            ));
        }
        match self.paths.insert(location.to_string(), Rc::new(file_entry)) {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unable to add file entry given location is already set",
                ));
            }
            None => {}
        }
        Ok(())
    }
}

impl VfsFileSystem for FakeVfsFileSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPathReference) -> io::Result<bool> {
        if path.path_type != VfsPathType::Fake {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        Ok(self.paths.contains_key(&path.location))
    }

    /// Retrieves the path type.
    fn get_vfs_path_type(&self) -> VfsPathType {
        VfsPathType::Fake
    }

    /// Opens a file system.
    fn open(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        if file_system.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported file system",
            ));
        }
        if path.path_type != VfsPathType::Fake {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        self.is_open = true;

        Ok(())
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(
        &self,
        path: &VfsPathReference,
    ) -> io::Result<Option<VfsFileEntryReference>> {
        if path.path_type != VfsPathType::Fake {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        match self.paths.get(&path.location) {
            Some(file_entry) => Ok(Some(Box::new(file_entry.clone()))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::SharedValue;
    use crate::vfs::enums::VfsFileType;
    use crate::vfs::path::VfsPath;

    fn get_file_system() -> io::Result<FakeVfsFileSystem> {
        let vfs_file_system: VfsFileSystemReference = SharedValue::none();

        let mut test_file_system: FakeVfsFileSystem = FakeVfsFileSystem::new();

        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let vfs_file_entry: FakeVfsFileEntry = FakeVfsFileEntry::new_file(&test_data);
        test_file_system.add_file_entry("/fake/file.txt", vfs_file_entry)?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Fake, "/", None);
        test_file_system.open(&vfs_file_system, &vfs_path)?;

        Ok(test_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let test_file_system: FakeVfsFileSystem = get_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Fake, "/fake/file.txt", None);
        assert_eq!(test_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Fake, "/fake/bogus.txt", None);
        assert_eq!(test_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let test_file_system: FakeVfsFileSystem = FakeVfsFileSystem::new();

        let directory_name: &str = test_file_system.get_directory_name("/fake/file.txt");
        assert_eq!(directory_name, "/fake");

        Ok(())
    }

    #[test]
    fn test_get_vfs_path_type() -> io::Result<()> {
        let test_file_system: FakeVfsFileSystem = FakeVfsFileSystem::new();

        let vfs_path_type: VfsPathType = test_file_system.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Fake);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let vfs_file_system: VfsFileSystemReference = SharedValue::none();

        let mut test_file_system: FakeVfsFileSystem = FakeVfsFileSystem::new();

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Fake, "/", None);
        test_file_system.open(&vfs_file_system, &vfs_path)?;

        Ok(())
    }

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let test_file_system: FakeVfsFileSystem = get_file_system()?;

        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Fake, "/fake/file.txt", None);
        let vfs_file_entry: VfsFileEntryReference =
            test_file_system.open_file_entry(&test_vfs_path)?.unwrap();
        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let test_file_system: FakeVfsFileSystem = get_file_system()?;

        let test_vfs_path: VfsPathReference = VfsPath::new(VfsPathType::NotSet, "/", None);
        let result = test_file_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
