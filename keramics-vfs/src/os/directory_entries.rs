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

use std::ffi::OsString;
use std::fs::{ReadDir, read_dir};
use std::path::PathBuf;

use keramics_core::ErrorTrace;

/// Operating system directory entries.
pub struct OsDirectoryEntries {
    /// Entries.
    entries: Vec<OsString>,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl OsDirectoryEntries {
    /// Creates new directory entries.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(&self, entry_index: usize) -> Option<&OsString> {
        self.entries.get(entry_index)
    }

    /// Retrieves the number of entries.
    pub fn get_number_of_entries(&self) -> usize {
        self.entries.len()
    }

    /// Determines if the directory entries were read.
    pub fn is_read(&self) -> bool {
        return self.is_read;
    }

    /// Reads the directory entries.
    pub fn read(&mut self, path: &PathBuf) -> Result<(), ErrorTrace> {
        let directory_iterator: ReadDir = match read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to read directory entries",
                    error
                ));
            }
        };
        for result in directory_iterator {
            match result {
                Ok(directory_entry) => {
                    self.entries.push(directory_entry.file_name());
                }
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to read directory entry",
                        error
                    ));
                }
            }
        }
        self.is_read = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_get_entry_by_index() -> Result<(), ErrorTrace> {
        let mut directory_entries: OsDirectoryEntries = OsDirectoryEntries::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("directory").as_str());
        directory_entries.read(&path_buf)?;

        let entry: Option<&OsString> = directory_entries.get_entry_by_index(0);
        assert!(entry.is_some());

        let entry: Option<&OsString> = directory_entries.get_entry_by_index(99);
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_entries() -> Result<(), ErrorTrace> {
        let mut directory_entries: OsDirectoryEntries = OsDirectoryEntries::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("directory").as_str());
        directory_entries.read(&path_buf)?;

        assert_eq!(directory_entries.get_number_of_entries(), 2);

        Ok(())
    }

    #[test]
    fn test_read() -> Result<(), ErrorTrace> {
        let mut directory_entries: OsDirectoryEntries = OsDirectoryEntries::new();

        assert_eq!(directory_entries.entries.len(), 0);
        assert_eq!(directory_entries.is_read, false);

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("directory").as_str());
        directory_entries.read(&path_buf)?;

        assert_eq!(directory_entries.entries.len(), 2);
        assert_eq!(directory_entries.is_read, true);

        Ok(())
    }
}
