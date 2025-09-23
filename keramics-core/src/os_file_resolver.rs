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

use std::io;
use std::path::MAIN_SEPARATOR_STR;

use super::data_stream::DataStreamReference;
use super::file_resolver::{FileResolver, FileResolverReference};
use super::os_data_stream::open_os_data_stream;

pub struct OsFileResolver {
    /// Base path.
    base_path: String,
}

impl OsFileResolver {
    /// Creates a new file resolver.
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }
}

impl FileResolver for OsFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream<'a>(
        &'a self,
        path_components: &mut Vec<&'a str>,
    ) -> io::Result<Option<DataStreamReference>> {
        let mut components: Vec<&str> = vec![self.base_path.as_str()];
        components.append(path_components);

        let path: String = components.join(MAIN_SEPARATOR_STR);
        let data_stream: DataStreamReference = open_os_data_stream(path.as_str())?;

        Ok(Some(data_stream))
    }
}

/// Opens a new operating system file resolver.
pub fn open_os_file_resolver(base_path: &str) -> io::Result<FileResolverReference> {
    let file_resolver: OsFileResolver = OsFileResolver::new(base_path);
    Ok(FileResolverReference::new(Box::new(file_resolver)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_data_stream() -> io::Result<()> {
        let file_resolver: OsFileResolver = OsFileResolver::new("../test_data/");

        let mut path_components: Vec<&str> = vec!["file.txt"];

        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&mut path_components)?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_open_os_file_resolver() -> io::Result<()> {
        let _ = open_os_file_resolver("../test_data/")?;

        Ok(())
    }
}
