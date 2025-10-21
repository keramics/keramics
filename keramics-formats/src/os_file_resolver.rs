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
        Self {
            base_path: base_path,
        }
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
                PathComponent::String(string) => {
                    path_buf.push(string);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported path component"
                    ));
                }
            }
        }
        let data_stream: DataStreamReference = match open_os_data_stream(&path_buf) {
            Ok(data_stream) => data_stream,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to open data stream with error",
                    error
                ));
            }
        };
        Ok(Some(data_stream))
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
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("").as_str());
        let file_resolver: OsFileResolver = OsFileResolver::new(path_buf);

        let path_components: [PathComponent; 1] = [PathComponent::from("file.txt")];

        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&path_components)?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_open_os_file_resolver() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("").as_str());
        let _ = open_os_file_resolver(&path_buf)?;

        Ok(())
    }
}
