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
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::{FileResolver, PathComponent, RangeStream};

pub struct RangeFileResolver {
    /// Base path.
    base_path: PathBuf,

    /// The offset of the range.
    range_offset: u64,
}

impl RangeFileResolver {
    /// Creates a new file resolver.
    pub fn new(base_path: PathBuf, range_offset: u64) -> Self {
        Self {
            base_path,
            range_offset,
        }
    }

    /// Retrieves a path based on the base path and path components.
    fn get_path(&self, path_components: &[PathComponent]) -> PathBuf {
        let mut path_buf: PathBuf = self.base_path.clone();

        for path_component in path_components.iter() {
            match path_component {
                PathComponent::ByteString(byte_string) => {
                    path_buf.push(byte_string.to_string());
                }
                PathComponent::Current => path_buf.push("."),
                PathComponent::OsString(os_string) => path_buf.push(os_string),
                PathComponent::Parent => path_buf.push(".."),
                PathComponent::Root => path_buf.push(MAIN_SEPARATOR_STR),
                PathComponent::String(string) => path_buf.push(string),
                PathComponent::Ucs2String(ucs2_string) => {
                    path_buf.push(ucs2_string.to_string());
                }
                PathComponent::Utf16String(utf16_string) => {
                    path_buf.push(utf16_string.to_string());
                }
            }
        }
        path_buf
    }
}

impl FileResolver for RangeFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let path_buf: PathBuf = self.get_path(path_components);

        // Note that symlink_metadata() is used to prevent traversing symbolic links.
        match symlink_metadata(&path_buf) {
            Ok(_) => {
                let data_stream: DataStreamReference = match open_os_data_stream(&path_buf) {
                    Ok(data_stream) => data_stream,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
                        return Err(error);
                    }
                };
                let size: u64 = keramics_core::data_stream_get_size!(data_stream);

                Ok(Some(Arc::new(RwLock::new(RangeStream::new(
                    &data_stream,
                    self.range_offset,
                    size - self.range_offset,
                )))))
            }
            Err(ref error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to determine if data stream exists",
                error
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data");
        let file_resolver: RangeFileResolver = RangeFileResolver::new(path_buf, 0);

        let path_components: [PathComponent; 2] = [
            PathComponent::from("directory"),
            PathComponent::from("file.txt"),
        ];
        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&path_components)?;
        assert!(data_stream.is_some());

        Ok(())
    }
}
