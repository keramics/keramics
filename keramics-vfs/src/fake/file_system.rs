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

use std::collections::HashMap;
use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_formats::{Path, PathComponent};

use super::file_entry::FakeFileEntry;

/// Fake (or virtual) file system.
pub struct FakeFileSystem {
    /// File entries per path.
    paths: HashMap<Path, Arc<FakeFileEntry>>,
}

impl FakeFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        let root_path: Path = Path::from("/");
        let root_file_entry: FakeFileEntry = FakeFileEntry::new_root();

        Self {
            paths: HashMap::from([(root_path, Arc::new(root_file_entry))]),
        }
    }

    /// Adds a new file entry.
    pub fn add_file_entry(
        &mut self,
        path: Path,
        file_entry: FakeFileEntry,
    ) -> Result<(), ErrorTrace> {
        let file_name: &PathComponent = file_entry.get_name();

        let file_entry_path: Path = match file_name {
            PathComponent::Root => path,
            _ => {
                let path_components: [PathComponent; 1] = [file_name.clone()];

                path.new_with_join_path_components(&path_components)
            }
        };
        match self.paths.insert(file_entry_path, Arc::new(file_entry)) {
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
    pub fn file_entry_exists(&self, path: &Path) -> Result<bool, ErrorTrace> {
        Ok(self.paths.contains_key(path))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<Arc<FakeFileEntry>>, ErrorTrace> {
        match self.paths.get(path) {
            Some(file_entry) => Ok(Some(file_entry.clone())),
            None => Ok(None),
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<Arc<FakeFileEntry>, ErrorTrace> {
        let path: Path = Path::from("/");

        match self.paths.get(&path) {
            Some(file_entry) => Ok(file_entry.clone()),
            None => Err(keramics_core::error_trace_new!("Missing root file entry")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsFileType;

    fn get_file_system() -> Result<FakeFileSystem, ErrorTrace> {
        let mut fake_file_system: FakeFileSystem = FakeFileSystem::new();

        let path: Path = Path::from("/");
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_directory("fake");
        fake_file_system.add_file_entry(path, fake_file_entry)?;

        let path: Path = Path::from("/fake");
        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file("file.txt", &test_data);
        fake_file_system.add_file_entry(path, fake_file_entry)?;

        Ok(fake_file_system)
    }

    #[test]
    fn test_add_file_entry() -> Result<(), ErrorTrace> {
        let mut fake_file_system: FakeFileSystem = FakeFileSystem::new();

        let path: Path = Path::from("/");
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_directory("fake");
        fake_file_system.add_file_entry(path, fake_file_entry)?;

        let path: Path = Path::from("/fake");
        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file("file.txt", &test_data);
        fake_file_system.add_file_entry(path, fake_file_entry)?;

        Ok(())
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let fake_file_system: FakeFileSystem = get_file_system()?;

        let path: Path = Path::from("/fake/file.txt");
        let result: bool = fake_file_system.file_entry_exists(&path)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/fake/bogus.txt");
        let result: bool = fake_file_system.file_entry_exists(&path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let fake_file_system: FakeFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let result: Option<Arc<FakeFileEntry>> = fake_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let fake_file_entry: Arc<FakeFileEntry> = result.unwrap();

        let name: &PathComponent = fake_file_entry.get_name();
        assert_eq!(name, &PathComponent::Root);

        let file_type: VfsFileType = fake_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let path: Path = Path::from("/fake/file.txt");
        let result: Option<Arc<FakeFileEntry>> = fake_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_some());

        let fake_file_entry: Arc<FakeFileEntry> = result.unwrap();

        let name: &PathComponent = fake_file_entry.get_name();
        assert_eq!(name, &PathComponent::from("file.txt"));

        let file_type: VfsFileType = fake_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        let path: Path = Path::from("/fake/bogus.txt");
        let result: Option<Arc<FakeFileEntry>> = fake_file_system.get_file_entry_by_path(&path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let fake_file_system: FakeFileSystem = get_file_system()?;

        let fake_file_entry: Arc<FakeFileEntry> = fake_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = fake_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }
}
