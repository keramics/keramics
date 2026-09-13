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

use keramics_core::ErrorTrace;
use keramics_formats::{FileEntryIterator, Path};

use crate::file_entry::VfsFileEntry;
use crate::file_system::VfsFileSystem;

/// Virtual File System (VFS) finder state.
struct VfsFinderState {
    /// File entry.
    file_entry: VfsFileEntry,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,

    /// Value to indicate the state has been initialized.
    is_initialized: bool,
}

impl VfsFinderState {
    /// Creates a new finder state.
    fn new(file_entry: VfsFileEntry) -> Self {
        Self {
            file_entry,
            number_of_sub_file_entries: 0,
            sub_file_entry_index: 0,
            is_initialized: false,
        }
    }
}

/// Virtual File System (VFS) finder.
pub struct VfsFinder<'a> {
    /// File system.
    file_system: &'a VfsFileSystem,

    /// Path.
    path: Path,

    /// Finder states.
    states: Vec<VfsFinderState>,

    /// Value to indicate the finder has started searching.
    search_started: bool,

    /// Value to indicate the finder encountered an error.
    error_encountered: bool,
}

// TODO: add support for filters (FindSpecs).

impl<'a> VfsFinder<'a> {
    /// Creates a new finder.
    pub fn new(file_system: &'a VfsFileSystem) -> Self {
        Self {
            file_system,
            path: Path::from("/"),
            states: Vec::new(),
            search_started: false,
            error_encountered: false,
        }
    }

    /// Retrieves the current path.
    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

impl<'a> Iterator for VfsFinder<'a> {
    type Item = Result<(VfsFileEntry, Path), ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.search_started {
            self.search_started = true;
            match self.file_system.get_root_file_entry() {
                Ok(result) => match result {
                    Some(file_entry) => {
                        self.states.push(VfsFinderState::new(file_entry));
                    }
                    None => return None,
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve root file entry"
                    );
                    return Some(Err(error));
                }
            };
        }
        if self.error_encountered {
            self.path.components.pop();
            self.error_encountered = false;
        }
        while let Some(mut state) = self.states.pop() {
            if !state.is_initialized {
                match state.file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => {
                        state.number_of_sub_file_entries = number_of_sub_file_entries
                    }
                    Err(mut error) => {
                        self.error_encountered = true;

                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of sub file entries"
                        );
                        return Some(Err(error));
                    }
                }
                state.is_initialized = true;
            }
            if state.sub_file_entry_index >= state.number_of_sub_file_entries {
                let path: Path = self.path.clone();

                self.path.components.pop();

                return Some(Ok((state.file_entry, path)));
            }
            let result: Result<VfsFileEntry, ErrorTrace> = state
                .file_entry
                .get_sub_file_entry_by_index(state.sub_file_entry_index);
            let sub_file_entry_index: usize = state.sub_file_entry_index;

            state.sub_file_entry_index += 1;
            self.states.push(state);

            match result {
                Ok(file_entry) => {
                    match file_entry.get_name() {
                        Some(name) => self.path.push(name),
                        None => {
                            return Some(Err(keramics_core::error_trace_new!(
                                "Missing name for file entry"
                            )));
                        }
                    }
                    self.states.push(VfsFinderState::new(file_entry));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve sub file entry: {}",
                            sub_file_entry_index
                        )
                    );
                    return Some(Err(error));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_formats::ntfs::NtfsFileSystem;

    use crate::enums::VfsFileType;
    use crate::tests::get_test_data_path;

    fn get_ntfs_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let test_data_path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: keramics_core::DataStreamReference =
            open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(VfsFileSystem::Ntfs(file_system))
    }

    #[test]
    fn test_new() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let vfs_finder: VfsFinder<'_> = VfsFinder::new(&vfs_file_system);

        assert_eq!(vfs_finder.get_path(), &Path::from("/"));
        assert!(vfs_finder.states.is_empty());
        assert!(vfs_finder.search_started == false);
        assert!(vfs_finder.error_encountered == false);

        Ok(())
    }

    #[test]
    fn test_get_path() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let vfs_finder: VfsFinder<'_> = VfsFinder::new(&vfs_file_system);

        let path: &Path = vfs_finder.get_path();
        assert_eq!(path, &Path::from("/"));

        Ok(())
    }

    #[test]
    fn test_next() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;
        let mut vfs_finder: VfsFinder<'_> = VfsFinder::new(&vfs_file_system);

        let mut number_of_file_entries: usize = 0;
        let mut file_entry_paths: Vec<String> = Vec::new();

        while let Some(result) = vfs_finder.next() {
            let (file_entry, file_entry_path): (VfsFileEntry, Path) = result?;

            number_of_file_entries += 1;

            match file_entry.get_name() {
                Some(name) => match file_entry_path.file_name() {
                    Some(file_name) => {
                        assert_eq!(
                            name.to_string().as_str().to_lowercase(),
                            file_name.to_string().as_str().to_lowercase()
                        );
                    }
                    None => {}
                },
                None => {}
            }

            file_entry_paths.push(file_entry_path.to_string());
        }

        let expected_paths: [String; 5] = [
            "/testdir1".to_string(),
            "/testdir1/testfile1".to_string(),
            "/emptyfile".to_string(),
            "/$Extend".to_string(),
            "/".to_string(),
        ];

        assert!(number_of_file_entries > 0);
        for expected_path in expected_paths.iter() {
            assert!(
                file_entry_paths.contains(expected_path),
                "missing path: {}",
                expected_path
            );
        }

        Ok(())
    }

    #[test]
    fn test_next_with_file_types() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;
        let mut vfs_finder: VfsFinder<'_> = VfsFinder::new(&vfs_file_system);

        let directories: [String; 3] = [
            "/".to_string(),
            "/testdir1".to_string(),
            "/$Extend".to_string(),
        ];
        let files: [String; 4] = [
            "/$MFT".to_string(),
            "/$Boot".to_string(),
            "/emptyfile".to_string(),
            "/testdir1/testfile1".to_string(),
        ];

        while let Some(result) = vfs_finder.next() {
            let (file_entry, path): (VfsFileEntry, Path) = result?;

            let file_type: VfsFileType = file_entry.get_file_type();
            let path_string: String = path.to_string();
            if directories.contains(&path_string) {
                assert_eq!(file_type, VfsFileType::Directory);
            } else if files.contains(&path_string) {
                assert_eq!(file_type, VfsFileType::File);
            }
        }

        Ok(())
    }
}
