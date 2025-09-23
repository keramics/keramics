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

use super::data_stream::DataStreamReference;
use super::file_resolver::FileResolver;

pub struct FakeFileResolver {}

impl FakeFileResolver {
    /// Creates a new file resolver.
    pub fn new() -> Self {
        Self {}
    }
}

impl FileResolver for FakeFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream<'a>(
        &'a self,
        _path_components: &mut Vec<&'a str>,
    ) -> io::Result<Option<DataStreamReference>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_data_stream() -> io::Result<()> {
        let file_resolver: FakeFileResolver = FakeFileResolver::new();

        let mut path_components: Vec<&str> = vec!["file.txt"];

        let data_stream: Option<DataStreamReference> =
            file_resolver.get_data_stream(&mut path_components)?;
        assert!(data_stream.is_none());

        Ok(())
    }
}
