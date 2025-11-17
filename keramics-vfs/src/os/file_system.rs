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

use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::{Path, PathComponent};

use super::file_entry::OsFileEntry;

/// Operating system file system.
pub struct OsFileSystem {}

impl OsFileSystem {
    fn get_path_buf(path: &Path) -> Result<PathBuf, ErrorTrace> {
        let mut path_buf: PathBuf = PathBuf::new();

        for path_component in path.components.iter() {
            match path_component {
                PathComponent::ByteString(_) => todo!(),
                PathComponent::OsString(os_string) => path_buf.push(os_string),
                PathComponent::String(string) => path_buf.push(string),
                PathComponent::Ucs2String(ucs2_string) => todo!(),
            }
        }
        Ok(path_buf)
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(path: &Path) -> Result<bool, ErrorTrace> {
        let path_buf: PathBuf = Self::get_path_buf(path)?;

        match path_buf.try_exists() {
            Ok(result) => Ok(result),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to determine if file entry exists",
                error
            )),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(path: &Path) -> Result<Option<OsFileEntry>, ErrorTrace> {
        let path_buf: PathBuf = Self::get_path_buf(path)?;

        match path_buf.try_exists() {
            Ok(false) => Ok(None),
            Ok(true) => {
                let mut os_file_entry: OsFileEntry = OsFileEntry::new();

                match os_file_entry.open(&path_buf) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to open OS file entry",
                            error
                        ));
                    }
                }
                Ok(Some(os_file_entry))
            }
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to determine if OS file entry exists",
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
