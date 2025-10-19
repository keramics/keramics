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
use std::sync::Arc;

use keramics_core::ErrorTrace;

use crate::path::VfsPath;

use super::file_entry::FakeFileEntry;

/// Fake (or virtual) file system.
pub struct FakeFileSystem {
    /// Paths.
    paths: HashMap<String, Arc<FakeFileEntry>>,
}

impl FakeFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Adds a new file entry.
    pub fn add_file_entry(
        &mut self,
        path: &str,
        file_entry: FakeFileEntry,
    ) -> Result<(), ErrorTrace> {
        match self.paths.insert(path.to_string(), Arc::new(file_entry)) {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to add file entry given path is already set"
                ));
            }
            None => {}
        }
        Ok(())
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        // TODO: use string path components as key
        let path: String = vfs_path.to_string();

        Ok(self.paths.contains_key(&path))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<Arc<FakeFileEntry>>, ErrorTrace> {
        // TODO: use string path components as key
        let path: String = vfs_path.to_string();

        let result: Option<Arc<FakeFileEntry>> = match self.paths.get(&path) {
            Some(file_entry) => Some(file_entry.clone()),
            None => None,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::VfsType;

    fn get_file_system() -> Result<FakeFileSystem, ErrorTrace> {
        let mut fake_file_system: FakeFileSystem = FakeFileSystem::new();

        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&test_data);
        fake_file_system.add_file_entry("/fake/file.txt", fake_file_entry)?;

        Ok(fake_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let fake_file_system: FakeFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/fake/file.txt");
        let result: bool = fake_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/fake/bogus.txt");
        let result: bool = fake_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }
}
