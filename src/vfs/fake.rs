/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use crate::datetime::DateTime;
use crate::types::SharedValue;
use crate::vfs::data_streams::new_fake_data_stream;

use super::enums::VfsFileType;
use super::types::VfsDataStreamReference;

/// Fake (or virtual) file entry.
pub struct FakeFileEntry {
    /// Location.
    location: String,

    /// File type.
    file_type: VfsFileType,

    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Access time.
    access_time: Option<DateTime>,

    /// Change time.
    change_time: Option<DateTime>,

    /// Creation time.
    creation_time: Option<DateTime>,

    /// Modification time.
    modification_time: Option<DateTime>,
}

impl FakeFileEntry {
    /// Creates a new file entry.
    pub fn new() -> Self {
        // TODO: test timestamps with current time
        Self {
            location: String::new(),
            data_stream: SharedValue::none(),
            file_type: VfsFileType::NotSet,
            access_time: None,
            change_time: None,
            creation_time: None,
            modification_time: None,
        }
    }

    /// Creates a new file entry.
    pub fn new_file(data: &[u8]) -> Self {
        // TODO: test timestamps with current time
        Self {
            location: String::new(),
            data_stream: new_fake_data_stream(data.to_vec()),
            file_type: VfsFileType::File,
            access_time: None,
            change_time: None,
            creation_time: None,
            modification_time: None,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        self.access_time.as_ref()
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        self.change_time.as_ref()
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.creation_time.as_ref()
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        self.modification_time.as_ref()
    }

    /// Retrieves the file type.
    pub fn get_vfs_file_type(&self) -> VfsFileType {
        self.file_type.clone()
    }

    /// Opens a data stream with the specified name.
    pub fn open_data_stream(
        &self,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        if self.file_type != VfsFileType::File || name.is_some() || self.data_stream.is_none() {
            return Ok(None);
        }
        Ok(Some(self.data_stream.clone()))
    }
}

/// Fake (or virtual) file system.
pub struct FakeFileSystem {
    /// Paths.
    paths: HashMap<String, Rc<FakeFileEntry>>,
}

impl FakeFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Adds a new file entry.
    pub fn add_file_entry(&mut self, path: &str, file_entry: FakeFileEntry) -> io::Result<()> {
        match self.paths.insert(path.to_string(), Rc::new(file_entry)) {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unable to add file entry given path is already set",
                ));
            }
            None => {}
        }
        Ok(())
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &str) -> io::Result<bool> {
        Ok(self.paths.contains_key(path))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &str) -> io::Result<Option<Rc<FakeFileEntry>>> {
        let result: Option<Rc<FakeFileEntry>> = match self.paths.get(path) {
            Some(file_entry) => Some(file_entry.clone()),
            None => None,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsFileType;

    fn get_file_system() -> io::Result<FakeFileSystem> {
        let mut test_file_system: FakeFileSystem = FakeFileSystem::new();

        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&test_data);
        test_file_system.add_file_entry("/fake/file.txt", fake_file_entry)?;

        Ok(test_file_system)
    }

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ];
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let test_file_system: FakeFileSystem = get_file_system()?;

        assert_eq!(test_file_system.file_entry_exists("/fake/file.txt")?, true);
        assert_eq!(
            test_file_system.file_entry_exists("/fake/bogus.txt")?,
            false
        );

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let test_file_system: FakeFileSystem = get_file_system()?;

        let fake_file_entry: Rc<FakeFileEntry> = test_file_system
            .get_file_entry_by_path("/fake/file.txt")?
            .unwrap();
        assert!(fake_file_entry.file_type == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_data_stream() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&test_data);

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        let result: Option<VfsDataStreamReference> = fake_file_entry.open_data_stream(None)?;

        let vfs_data_stream: VfsDataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Missing data stream"),
                ))
            }
        };
        let mut test_data: Vec<u8> = vec![];
        let read_count: usize = match vfs_data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_to_end(&mut test_data)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert_eq!(read_count, 202);
        assert_eq!(test_data, expected_data.as_bytes());

        Ok(())
    }
}
