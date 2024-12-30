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

use crate::vfs::file_entries::FakeVfsFileEntry;
use crate::vfs::types::VfsFileEntryReference;

/// Fake (or virtual) file system.
pub struct FakeFileSystem {
    /// Paths.
    paths: HashMap<String, Rc<FakeVfsFileEntry>>,
}

impl FakeFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Adds a new file entry.
    pub fn add_file_entry(&mut self, path: &str, file_entry: FakeVfsFileEntry) -> io::Result<()> {
        match self.paths.insert(path.to_string(), Rc::new(file_entry)) {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unable to add file entry given path is already set",
                ));
            }
            None => {}
        }
        Ok(())
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &str) -> io::Result<bool> {
        Ok(self.paths.contains_key(path))
    }

    /// Opens a file entry with the specified path.
    pub fn open_file_entry(&self, path: &str) -> io::Result<Option<VfsFileEntryReference>> {
        match self.paths.get(path) {
            Some(file_entry) => Ok(Some(Box::new(file_entry.clone()))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsFileType;

    fn get_file_system() -> io::Result<FakeFileSystem> {
        let mut test_file_system: FakeFileSystem = FakeFileSystem::new();

        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let vfs_file_entry: FakeVfsFileEntry = FakeVfsFileEntry::new_file(&test_data);
        test_file_system.add_file_entry("/fake/file.txt", vfs_file_entry)?;

        Ok(test_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let test_file_system: FakeFileSystem = get_file_system()?;

        assert_eq!(test_file_system.file_entry_exists("/fake/file.txt")?, true);
        assert_eq!(
            test_file_system.file_entry_exists("/fake/bogus.txt")?,
            false
        );

        Ok(())
    }

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let test_file_system: FakeFileSystem = get_file_system()?;

        let vfs_file_entry: VfsFileEntryReference =
            test_file_system.open_file_entry("/fake/file.txt")?.unwrap();
        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }
}
