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

use super::data_fork::VfsDataFork;
use super::file_entry::VfsFileEntry;

/// Virtual File System (VFS) data fork iterator.
pub struct VfsDataForksIterator<'a> {
    /// File entry.
    file_entry: &'a mut VfsFileEntry,

    /// Number of data fork.
    number_of_data_forks: usize,

    /// Extended attribute index.
    data_fork_index: usize,

    /// Value to indicate whether the iterator is initialized.
    is_initialized: bool,
}

impl<'a> VfsDataForksIterator<'a> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut VfsFileEntry) -> Self {
        Self {
            file_entry,
            number_of_data_forks: 0,
            data_fork_index: 0,
            is_initialized: false,
        }
    }
}

impl<'a> Iterator for VfsDataForksIterator<'a> {
    type Item = Result<VfsDataFork, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_initialized {
            match self.file_entry.get_number_of_data_forks() {
                Ok(number_of_data_forks) => {
                    self.number_of_data_forks = number_of_data_forks;
                }
                Err(error) => return Some(Err(error)),
            }
            self.is_initialized = true;
        }
        if self.data_fork_index >= self.number_of_data_forks {
            return None;
        }
        let item: Self::Item = self.file_entry.get_data_fork_by_index(self.data_fork_index);

        self.data_fork_index += 1;

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_formats::ntfs::NtfsFileSystem;
    use keramics_formats::Path;

    use crate::tests::get_test_data_path;

    fn get_ntfs_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let test_data_path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: keramics_core::DataStreamReference =
            open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        let path: Path = Path::from(path_string);
        match file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(VfsFileEntry::Ntfs(file_entry)),
            None => Err(keramics_core::error_trace_new!("Missing file entry")),
        }
    }

    #[test]
    fn test_new() -> Result<(), ErrorTrace> {
        let mut file_entry: VfsFileEntry = get_ntfs_file_entry("/$UpCase")?;

        let iterator: VfsDataForksIterator<'_> = VfsDataForksIterator::new(&mut file_entry);

        assert_eq!(iterator.number_of_data_forks, 0);
        assert_eq!(iterator.data_fork_index, 0);
        assert!(iterator.is_initialized == false);

        Ok(())
    }

    #[test]
    fn test_next() -> Result<(), ErrorTrace> {
        let mut file_entry: VfsFileEntry = get_ntfs_file_entry("/$UpCase")?;

        let mut iterator: VfsDataForksIterator<'_> = VfsDataForksIterator::new(&mut file_entry);

        let result: Option<Result<VfsDataFork, ErrorTrace>> = iterator.next();
        let data_fork: VfsDataFork = match result {
            Some(result) => result?,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data fork"));
            }
        };
        let _ = data_fork.get_data_stream()?;

        let result: Option<Result<VfsDataFork, ErrorTrace>> = iterator.next();
        let data_fork: VfsDataFork = match result {
            Some(result) => result?,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data fork"));
            }
        };
        let _ = data_fork.get_data_stream()?;

        let result: Option<Result<VfsDataFork, ErrorTrace>> = iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_next_without_data_forks() -> Result<(), ErrorTrace> {
        let mut file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let mut iterator: VfsDataForksIterator<'_> = VfsDataForksIterator::new(&mut file_entry);

        let result: Option<Result<VfsDataFork, ErrorTrace>> = iterator.next();
        assert!(result.is_none());

        Ok(())
    }
}
