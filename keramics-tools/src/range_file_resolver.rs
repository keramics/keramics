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

    /// Data stream.
    data_stream: Option<DataStreamReference>,
}

impl RangeFileResolver {
    /// Creates a new file resolver.
    pub fn new(
        base_path: PathBuf,
        range_offset: u64,
        data_stream: Option<&DataStreamReference>,
    ) -> Self {
        Self {
            base_path,
            range_offset,
            data_stream: data_stream.cloned(),
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
                let data_stream: DataStreamReference = match self.data_stream.as_ref() {
                    Some(data_stream) => data_stream.clone(),
                    None => match open_os_data_stream(&path_buf) {
                        Ok(data_stream) => data_stream,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to open data stream"
                            );
                            return Err(error);
                        }
                    },
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

    use std::ffi::OsString;
    use std::path::PathBuf;

    use keramics_core::open_fake_data_stream;
    use keramics_types::{ByteString, Ucs2String, Utf16String};

    #[test]
    fn test_get_path() {
        let base_path: PathBuf = PathBuf::from("../test_data");
        let file_resolver: RangeFileResolver = RangeFileResolver::new(base_path, 0, None);

        let components: [PathComponent; 8] = [
            PathComponent::Root,
            PathComponent::ByteString(ByteString::from("a")),
            PathComponent::Current,
            PathComponent::OsString(OsString::from("b")),
            PathComponent::Parent,
            PathComponent::String(String::from("c")),
            PathComponent::Ucs2String(Ucs2String::from("d")),
            PathComponent::Utf16String(Utf16String::from("e")),
        ];
        let path: PathBuf = file_resolver.get_path(&components);
        assert!(path.ends_with("e"));
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data");
        let file_resolver: RangeFileResolver = RangeFileResolver::new(path_buf.clone(), 0, None);

        let path_components: [PathComponent; 2] = [
            PathComponent::from("directory"),
            PathComponent::from("file.txt"),
        ];
        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&path_components)?;
        assert!(data_stream.is_some());

        // Test with a nonexistent path.
        let missing_components: [PathComponent; 1] = [PathComponent::from("does_not_exist.bin")];
        let result: Option<DataStreamReference> =
            file_resolver.get_data_stream(&missing_components)?;
        assert!(result.is_none());

        // Test with an invalid path containing a NUL byte.
        let invalid_components: [PathComponent; 1] = [PathComponent::from("bad\0path")];
        let result: Result<Option<DataStreamReference>, ErrorTrace> =
            file_resolver.get_data_stream(&invalid_components);
        assert!(result.is_err());

        // Test with a provided data stream and non-zero range offset.
        let fake_stream: DataStreamReference = open_fake_data_stream(b"0123456789abcdef");
        let range_resolver: RangeFileResolver =
            RangeFileResolver::new(path_buf, 4, Some(&fake_stream));
        let range_stream: DataStreamReference =
            range_resolver.get_data_stream(&path_components)?.unwrap();
        let mut data: [u8; 4] = [0; 4];
        let read_count: usize = range_stream.write().unwrap().read(&mut data)?;
        assert_eq!(read_count, 4);
        assert_eq!(&data, b"4567");

        Ok(())
    }
}
