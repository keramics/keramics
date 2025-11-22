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

use std::fs::symlink_metadata;
use std::io::ErrorKind;
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::Path;

use super::file_entry::OsFileEntry;

/// Operating system file system.
pub struct OsFileSystem {}

impl OsFileSystem {
    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(path: &Path) -> Result<bool, ErrorTrace> {
        let path_buf: PathBuf = match path.to_path_buf() {
            Ok(path_buf) => path_buf,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to determine path buffer from path",
                    error
                ));
            }
        };
        // Note that symlink_metadata() is used to prevent traversing symbolic links.
        match symlink_metadata(&path_buf) {
            Ok(_) => Ok(true),
            Err(ref error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to determine if file entry exists",
                error
            )),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(path: &Path) -> Result<Option<OsFileEntry>, ErrorTrace> {
        let path_buf: PathBuf = match path.to_path_buf() {
            Ok(path_buf) => path_buf,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to determine path buffer from path",
                    error
                ));
            }
        };
        let mut os_file_entry: OsFileEntry = OsFileEntry::new();

        match os_file_entry.open(&path_buf) {
            Ok(false) => Ok(None),
            Ok(true) => Ok(Some(os_file_entry)),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file entry");
                Err(error)
            }
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry() -> Result<OsFileEntry, ErrorTrace> {
        let mut os_file_entry: OsFileEntry = OsFileEntry::new();

        let path_buf: PathBuf = PathBuf::from("/");
        match os_file_entry.open(&path_buf) {
            Ok(true) => Ok(os_file_entry),
            Ok(false) => Err(keramics_core::error_trace_new!("Missing file entry")),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to open OS root directory",
                error
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
