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

use std::fs::symlink_metadata;
use std::io::ErrorKind;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};

use super::file_resolver::{FileResolver, FileResolverReference};
use super::path_component::PathComponent;

pub struct OsFileResolver {
    /// Base path.
    base_path: PathBuf,
}

impl OsFileResolver {
    /// Creates a new file resolver.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl FileResolver for OsFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let mut path_buf: PathBuf = self.base_path.clone();

        for path_component in path_components.iter() {
            match path_component {
                PathComponent::ByteString(byte_string) => {
                    let string: String = byte_string.to_string();

                    path_buf.push(string);
                }
                PathComponent::Current => path_buf.push("."),
                PathComponent::OsString(os_string) => path_buf.push(os_string),
                PathComponent::Parent => path_buf.push(".."),
                PathComponent::Root => path_buf.push(MAIN_SEPARATOR_STR),
                PathComponent::String(string) => path_buf.push(string),
                PathComponent::Ucs2String(ucs2_string) => {
                    let string: String = ucs2_string.to_string();

                    path_buf.push(string);
                }
            }
        }
        // Note that symlink_metadata() is used to prevent traversing symbolic links.
        match symlink_metadata(&path_buf) {
            Ok(_) => match open_os_data_stream(&path_buf) {
                Ok(data_stream) => Ok(Some(data_stream)),
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to open data stream with error",
                    error
                )),
            },
            Err(ref error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to determine if data stream exists",
                error
            )),
        }
    }
}

/// Opens a new operating system file resolver.
pub fn open_os_file_resolver(base_path: &PathBuf) -> Result<FileResolverReference, ErrorTrace> {
    let file_resolver: OsFileResolver = OsFileResolver::new(base_path.clone());

    Ok(FileResolverReference::new(Box::new(file_resolver)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::tests::get_test_data_path;

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: OsFileResolver = OsFileResolver::new(path_buf);

        let path_components: [PathComponent; 2] = [
            PathComponent::from("directory"),
            PathComponent::from("file.txt"),
        ];

        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&path_components)?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_open_os_file_resolver() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let _ = open_os_file_resolver(&path_buf)?;

        Ok(())
    }
}
